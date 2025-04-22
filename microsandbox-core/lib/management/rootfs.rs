//! Root filesystem management for Microsandbox sandboxes.
//!
//! This module provides functionality for managing root filesystems used by Microsandbox sandboxes.
//! It handles the creation, extraction, and merging of filesystem layers following OCI (Open
//! Container Initiative) specifications.

use std::{
    borrow::Cow,
    collections::HashMap,
    fs::Permissions,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use async_recursion::async_recursion;
use tokio::fs;

use crate::{config::PathPair, vm::VIRTIOFS_TAG_PREFIX, MicrosandboxResult};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// The opaque directory marker file name used in OCI layers.
pub const OPAQUE_WHITEOUT_MARKER: &str = ".wh..wh..opq";

/// The prefix for whiteout files in OCI layers.
pub const WHITEOUT_PREFIX: &str = ".wh.";

//--------------------------------------------------------------------------------------------------
// Structs
//--------------------------------------------------------------------------------------------------

/// RAII guard that temporarily changes file permissions and restores them when dropped
struct PermissionGuard {
    path: PathBuf,
    original_mode: u32,
}

impl PermissionGuard {
    /// Creates a new guard that temporarily adds the given mode bits to the file permissions
    fn new(path: impl AsRef<Path>, mode_to_add: u32) -> MicrosandboxResult<Self> {
        let path = path.as_ref().to_path_buf();
        let metadata = std::fs::metadata(&path)?;
        let original_mode = metadata.permissions().mode();

        // Update permissions
        let mut perms = metadata.permissions();
        perms.set_mode(original_mode | mode_to_add);
        std::fs::set_permissions(&path, perms)?;

        Ok(Self {
            path,
            original_mode,
        })
    }
}

impl Drop for PermissionGuard {
    fn drop(&mut self) {
        // Attempt to restore original permissions, ignore errors during drop
        if let Ok(mut perms) = std::fs::metadata(&self.path).and_then(|m| Ok(m.permissions())) {
            perms.set_mode(self.original_mode);
            let _ = fs::set_permissions(&self.path, perms);
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Updates a rootfs by adding sandbox script files to a `/.sandbox_scripts` directory.
///
/// This function:
/// 1. Creates a `.sandbox_scripts` directory under the rootfs if it doesn't exist
/// 2. For each script in the provided HashMap, creates a file with the given name
/// 3. Adds a shebang line using the provided shell path
/// 4. Makes the script files executable (rwxr-x---)
/// 5. Creates a `shell` script containing just the shell path
///
/// ## Arguments
///
/// * `root_path` - Path to the root of the filesystem to patch
/// * `scripts` - HashMap containing script names and their contents
/// * `shell_path` - Path to the shell binary within the rootfs (e.g. "/bin/sh")
pub async fn patch_with_sandbox_scripts(
    scripts_dir: &Path,
    scripts: Cow<'_, HashMap<String, String>>,
    shell_path: impl AsRef<Path>,
) -> MicrosandboxResult<()> {
    // Remove the scripts directory if it exists
    if scripts_dir.exists() {
        fs::remove_dir_all(&scripts_dir).await?;
    }

    // Create the directory if it doesn't exist
    fs::create_dir_all(&scripts_dir).await?;

    // Get shell path as string for shebang
    let shell_path = shell_path.as_ref().to_string_lossy();
    for (script_name, script_content) in scripts.iter() {
        // Create script file path
        let script_path = scripts_dir.join(script_name);

        // Write shebang and content
        let full_content = format!("#!{}\n{}\n", shell_path, script_content);
        fs::write(&script_path, full_content).await?;

        // Make executable for user and group (rwxr-x---)
        fs::set_permissions(&script_path, Permissions::from_mode(0o750)).await?;
    }

    // Create shell script containing just the shell path
    let shell_script_path = scripts_dir.join("shell");
    fs::write(&shell_script_path, shell_path.to_string()).await?;
    fs::set_permissions(&shell_script_path, Permissions::from_mode(0o750)).await?;

    Ok(())
}

/// Updates the /etc/fstab file in the guest rootfs to mount the mapped directories.
/// Creates the file if it doesn't exist.
///
/// This method:
/// 1. Creates or updates the /etc/fstab file in the guest rootfs
/// 2. Adds entries for each mapped directory using virtio-fs
/// 3. Creates the mount points in the guest rootfs
/// 4. Sets appropriate permissions on the fstab file
///
/// ## Format
/// Each mapped directory is mounted using virtiofs with the following format:
/// ```text
/// virtiofs_N  /guest/path  virtiofs  defaults  0  0
/// ```
/// where N is the index of the mapped directory.
///
/// ## Arguments
/// * `root_path` - Path to the guest rootfs
/// * `mapped_dirs` - List of host:guest directory mappings to mount
///
/// ## Errors
/// Returns an error if:
/// - Cannot create directories in the rootfs
/// - Cannot read or write the fstab file
/// - Cannot set permissions on the fstab file
pub async fn patch_with_virtiofs_mounts(
    root_path: &Path,
    mapped_dirs: &[PathPair],
) -> MicrosandboxResult<()> {
    let fstab_path = root_path.join("etc/fstab");

    // Create parent directories if they don't exist
    if let Some(parent) = fstab_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Read existing fstab content if it exists
    let mut fstab_content = if fstab_path.exists() {
        fs::read_to_string(&fstab_path).await?
    } else {
        String::new()
    };

    // Add header comment if file is empty
    if fstab_content.is_empty() {
        fstab_content.push_str(
            "# /etc/fstab: static file system information.\n\
                 # <file system>\t<mount point>\t<type>\t<options>\t<dump>\t<pass>\n",
        );
    }

    // Add entries for mapped directories
    for (idx, dir) in mapped_dirs.iter().enumerate() {
        let tag = format!("{}_{}", VIRTIOFS_TAG_PREFIX, idx);
        tracing::debug!("adding virtiofs mount for {}", tag);
        let guest_path = dir.get_guest();

        // Add entry for this mapped directory
        fstab_content.push_str(&format!(
            "{}\t{}\tvirtiofs\tdefaults\t0\t0\n",
            tag, guest_path
        ));

        // Create the mount point directory in the guest rootfs
        // Convert guest path to a relative path by removing leading slash
        let guest_path_str = guest_path.as_str();
        let relative_path = guest_path_str.strip_prefix('/').unwrap_or(guest_path_str);
        let mount_point = root_path.join(relative_path);
        fs::create_dir_all(mount_point).await?;
    }

    // Write updated fstab content
    fs::write(&fstab_path, fstab_content).await?;

    // Set proper permissions (644 - rw-r--r--)
    let perms = fs::metadata(&fstab_path).await?.permissions();
    let mut new_perms = perms;
    new_perms.set_mode(0o644);
    fs::set_permissions(&fstab_path, new_perms).await?;

    Ok(())
}

/// Updates the /etc/hosts file in the guest rootfs to add hostname mappings.
/// Creates the file if it doesn't exist.
///
/// This method:
/// 1. Creates or updates the /etc/hosts file in the guest rootfs
/// 2. Adds entries for each IP address and hostname pair
/// 3. Sets appropriate permissions on the hosts file
///
/// ## Format
/// Each hostname mapping follows the standard hosts file format:
/// ```text
/// 192.168.1.100  hostname1
/// 192.168.1.101  hostname2
/// ```
///
/// ## Arguments
/// * `root_path` - Path to the guest rootfs
/// * `hostname_mappings` - List of (IPv4 address, hostname) pairs to add
///
/// ## Errors
/// Returns an error if:
/// - Cannot create directories in the rootfs
/// - Cannot read or write the hosts file
/// - Cannot set permissions on the hosts file
async fn _patch_with_hostnames(
    root_path: &Path,
    hostname_mappings: &[(std::net::Ipv4Addr, String)],
) -> MicrosandboxResult<()> {
    let hosts_path = root_path.join("etc/hosts");

    // Create parent directories if they don't exist
    if let Some(parent) = hosts_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Read existing hosts content if it exists
    let mut hosts_content = if hosts_path.exists() {
        fs::read_to_string(&hosts_path).await?
    } else {
        String::new()
    };

    // Add header comment if file is empty
    if hosts_content.is_empty() {
        hosts_content.push_str(
            "# /etc/hosts: static table lookup for hostnames.\n\
             # <ip-address>\t<hostname>\n\n\
             127.0.0.1\tlocalhost\n\
             ::1\tlocalhost ip6-localhost ip6-loopback\n",
        );
    }

    // Add entries for hostname mappings
    for (ip_addr, hostname) in hostname_mappings {
        // Check if this mapping already exists
        let entry = format!("{}\t{}", ip_addr, hostname);
        if !hosts_content.contains(&entry) {
            hosts_content.push_str(&format!("{}\n", entry));
        }
    }

    // Write updated hosts content
    fs::write(&hosts_path, hosts_content).await?;

    // Set proper permissions (644 - rw-r--r--)
    let perms = fs::metadata(&hosts_path).await?.permissions();
    let mut new_perms = perms;
    new_perms.set_mode(0o644);
    fs::set_permissions(&hosts_path, new_perms).await?;

    Ok(())
}

/// Updates the /etc/resolv.conf file in the guest rootfs to add default DNS servers if none exist.
/// Creates the file if it doesn't exist.
///
/// This function:
/// 1. Checks all root paths for existing /etc/resolv.conf files
/// 2. If any nameserver entries exist in any layer, does nothing
/// 3. If no nameservers exist in any layer, adds default ones (1.1.1.1 and 8.8.8.8) to the top layer
/// 4. Sets appropriate permissions on the resolv.conf file
///
/// ## Format
/// The resolv.conf file follows the standard format:
/// ```text
/// nameserver 1.1.1.1
/// nameserver 8.8.8.8
/// ```
///
/// ## Arguments
/// * `root_paths` - List of root paths to check, ordered from bottom to top layer
///                  For overlayfs, this should be [lower_layers..., patch_dir]
///                  For native rootfs, this should be [root_path]
///
/// ## Errors
/// Returns an error if:
/// - Cannot create directories in the rootfs
/// - Cannot read or write the resolv.conf file
/// - Cannot set permissions on the resolv.conf file
pub async fn patch_with_default_dns_settings(root_paths: &[PathBuf]) -> MicrosandboxResult<()> {
    if root_paths.is_empty() {
        return Ok(());
    }

    // Check all layers for existing nameserver entries
    let mut has_nameserver = false;
    for root_path in root_paths {
        let resolv_path = root_path.join("etc/resolv.conf");
        if resolv_path.exists() {
            let content = fs::read_to_string(&resolv_path).await?;
            if content
                .lines()
                .any(|line| line.trim_start().starts_with("nameserver "))
            {
                has_nameserver = true;
                break;
            }
        }
    }

    // If no nameservers found in any layer, add defaults to the top layer
    if !has_nameserver {
        // Get the top layer (last in the list)
        let top_layer = root_paths.last().unwrap();
        let resolv_path = top_layer.join("etc/resolv.conf");

        // Create parent directories if they don't exist
        if let Some(parent) = resolv_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Create new resolv.conf with default nameservers
        let mut resolv_content = String::from("# /etc/resolv.conf: DNS resolver configuration\n");
        resolv_content.push_str("nameserver 1.1.1.1\n");
        resolv_content.push_str("nameserver 8.8.8.8\n");

        // Write the file
        fs::write(&resolv_path, resolv_content).await?;

        // Set proper permissions (644 - rw-r--r--)
        let perms = fs::metadata(&resolv_path).await?.permissions();
        let mut new_perms = perms;
        new_perms.set_mode(0o644);
        fs::set_permissions(&resolv_path, new_perms).await?;
    }

    Ok(())
}

/// Recursively copies a directory from source to destination, preserving file permissions.
///
/// This function:
/// 1. Creates the destination directory if it doesn't exist
/// 2. Recursively copies all files and subdirectories from source to destination
/// 3. Preserves the original file permissions for all copied files and directories
/// 4. Uses PermissionGuard to handle directories that may have restrictive permissions
///
/// ## Arguments
/// * `src_dir` - Path to the source directory
/// * `dst_dir` - Path to the destination directory
///
/// ## Errors
/// Returns an error if:
/// - Source directory doesn't exist or isn't readable
/// - Cannot create the destination directory
/// - Cannot copy files or subdirectories
/// - Cannot set permissions on copied files or directories
#[async_recursion(?Send)]
pub async fn copy_dir_recursive(src_dir: &Path, dst_dir: &Path) -> MicrosandboxResult<()> {
    // Ensure source directory exists
    // Ensure source directory exists
    if !src_dir.exists() {
        return Err(crate::MicrosandboxError::PathNotFound(format!(
            "source directory does not exist: {}",
            src_dir.display()
        )));
    }

    // Create destination directory if it doesn't exist
    if !dst_dir.exists() {
        fs::create_dir_all(dst_dir).await?;
    }

    // Copy the permissions from source to destination directory
    let src_metadata = fs::metadata(src_dir).await?;
    let src_perms = src_metadata.permissions();
    fs::set_permissions(dst_dir, src_perms.clone()).await?;

    // Create a permission guard to ensure we have read access to the source directory
    // This temporarily adds read and execute permissions if needed
    let _src_guard = PermissionGuard::new(src_dir, 0o500)?;

    // Read directory entries
    let mut entries = fs::read_dir(src_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst_dir.join(entry.file_name());

        let file_type = entry.file_type().await?;

        if file_type.is_dir() {
            // Recursively copy subdirectory
            copy_dir_recursive(&src_path, &dst_path).await?;
        } else if file_type.is_file() {
            // Copy file with permissions
            copy_file_with_permissions(&src_path, &dst_path).await?;
        } else if file_type.is_symlink() {
            // Copy symlink
            let target = fs::read_link(&src_path).await?;
            fs::symlink(target, &dst_path).await?;
        }
    }

    Ok(())
}

/// Copies a single file from source to destination, preserving file permissions.
///
/// This function:
/// 1. Creates the destination file
/// 2. Copies the file contents
/// 3. Preserves the original file permissions
///
/// ## Arguments
/// * `src_file` - Path to the source file
/// * `dst_file` - Path to the destination file
///
/// ## Errors
/// Returns an error if:
/// - Source file doesn't exist or isn't readable
/// - Cannot create or write to the destination file
/// - Cannot copy file contents
/// - Cannot set permissions on the destination file
async fn copy_file_with_permissions(
    src_file: impl AsRef<Path>,
    dst_file: impl AsRef<Path>,
) -> MicrosandboxResult<()> {
    let src_file = src_file.as_ref();
    let dst_file = dst_file.as_ref();

    // Create a permission guard to ensure we have read access to the source file
    let _src_guard = PermissionGuard::new(src_file, 0o400)?;

    // Copy the file contents
    fs::copy(src_file, dst_file).await?;

    // Copy the permissions from source to destination file
    let src_metadata = fs::metadata(src_file).await?;
    let src_perms = src_metadata.permissions();
    fs::set_permissions(dst_file, src_perms).await?;

    Ok(())
}

/// Sets the user.containers.override_stat xattr on the rootfs directory.
///
/// This function:
/// 1. Sets the extended attribute user.containers.override_stat to "0:0:0555"
/// 2. This overrides the UID:GID:MODE of the rootfs directory when accessed inside the container
///
/// ## Arguments
/// * `root_path` - Path to the rootfs directory to modify
///
/// ## Errors
/// Returns an error if:
/// - Cannot set the extended attribute
pub async fn patch_with_stat_override(root_path: &Path) -> MicrosandboxResult<()> {
    // The xattr name to set
    let xattr_name = "user.containers.override_stat";

    // The value in the format "uid:gid:mode" (0:0:0555 means root:root with r-xr-xr-x permissions)
    let xattr_value = "0:0:0555";

    // Convert path to CString for xattr crate
    let path_str = root_path.to_str().ok_or_else(|| {
        crate::MicrosandboxError::InvalidArgument(format!(
            "Could not convert path to string: {}",
            root_path.display()
        ))
    })?;

    // Set the xattr
    match xattr::set(path_str, xattr_name, xattr_value.as_bytes()) {
        Ok(_) => {
            tracing::debug!(
                "Set xattr {} = {} on {}",
                xattr_name,
                xattr_value,
                root_path.display()
            );
            Ok(())
        }
        Err(err) => Err(crate::MicrosandboxError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to set xattr on {}: {}", root_path.display(), err),
        ))),
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::TempDir;

    use crate::MicrosandboxError;

    use super::*;

    #[tokio::test]
    async fn test_patch_rootfs_with_virtiofs_mounts() -> anyhow::Result<()> {
        // Create a temporary directory to act as our rootfs
        let root_dir = TempDir::new()?;
        let root_path = root_dir.path();

        // Create temporary directories for host paths
        let host_dir = TempDir::new()?;
        let host_data = host_dir.path().join("data");
        let host_config = host_dir.path().join("config");
        let host_app = host_dir.path().join("app");

        // Create the host directories
        fs::create_dir_all(&host_data).await?;
        fs::create_dir_all(&host_config).await?;
        fs::create_dir_all(&host_app).await?;

        // Create test directory mappings using our temporary paths
        let mapped_dirs = vec![
            format!("{}:/container/data", host_data.display()).parse::<PathPair>()?,
            format!("{}:/etc/app/config", host_config.display()).parse::<PathPair>()?,
            format!("{}:/app", host_app.display()).parse::<PathPair>()?,
        ];

        // Update fstab
        patch_with_virtiofs_mounts(root_path, &mapped_dirs).await?;

        // Verify fstab file was created with correct content
        let fstab_path = root_path.join("etc/fstab");
        assert!(fstab_path.exists());

        let fstab_content = fs::read_to_string(&fstab_path).await?;

        // Check header
        assert!(fstab_content.contains("# /etc/fstab: static file system information"));
        assert!(fstab_content
            .contains("<file system>\t<mount point>\t<type>\t<options>\t<dump>\t<pass>"));

        // Check entries
        assert!(fstab_content.contains("virtiofs_0\t/container/data\tvirtiofs\tdefaults\t0\t0"));
        assert!(fstab_content.contains("virtiofs_1\t/etc/app/config\tvirtiofs\tdefaults\t0\t0"));
        assert!(fstab_content.contains("virtiofs_2\t/app\tvirtiofs\tdefaults\t0\t0"));

        // Verify mount points were created
        assert!(root_path.join("container/data").exists());
        assert!(root_path.join("etc/app/config").exists());
        assert!(root_path.join("app").exists());

        // Verify file permissions
        let perms = fs::metadata(&fstab_path).await?.permissions();
        assert_eq!(perms.mode() & 0o777, 0o644);

        // Test updating existing fstab
        let host_logs = host_dir.path().join("logs");
        fs::create_dir_all(&host_logs).await?;

        let new_mapped_dirs = vec![
            format!("{}:/container/data", host_data.display()).parse::<PathPair>()?, // Keep one existing
            format!("{}:/var/log", host_logs.display()).parse::<PathPair>()?,        // Add new one
        ];

        // Update fstab again
        patch_with_virtiofs_mounts(root_path, &new_mapped_dirs).await?;

        // Verify updated content
        let updated_content = fs::read_to_string(&fstab_path).await?;
        assert!(updated_content.contains("virtiofs_0\t/container/data\tvirtiofs\tdefaults\t0\t0"));
        assert!(updated_content.contains("virtiofs_1\t/var/log\tvirtiofs\tdefaults\t0\t0"));

        // Verify new mount point was created
        assert!(root_path.join("var/log").exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_patch_rootfs_with_virtiofs_mounts_permission_errors() -> anyhow::Result<()> {
        // Skip this test in CI environments
        if std::env::var("CI").is_ok() {
            println!("Skipping permission test in CI environment");
            return Ok(());
        }

        // Setup a rootfs where we can't write the fstab file
        let readonly_dir = TempDir::new()?;
        let readonly_path = readonly_dir.path();
        let etc_path = readonly_path.join("etc");
        fs::create_dir_all(&etc_path).await?;

        // Make /etc directory read-only to simulate permission issues
        let mut perms = fs::metadata(&etc_path).await?.permissions();
        perms.set_mode(0o400); // read-only
        fs::set_permissions(&etc_path, perms).await?;

        // Verify permissions were actually set (helpful for debugging)
        let actual_perms = fs::metadata(&etc_path).await?.permissions();
        println!("Set /etc permissions to: {:o}", actual_perms.mode());

        // Try to update fstab in a read-only /etc directory
        let host_dir = TempDir::new()?;
        let host_path = host_dir.path().join("test");
        fs::create_dir_all(&host_path).await?;

        let mapped_dirs =
            vec![format!("{}:/container/data", host_path.display()).parse::<PathPair>()?];

        // Function should detect it cannot write to /etc/fstab and return an error
        let result = patch_with_virtiofs_mounts(readonly_path, &mapped_dirs).await;

        // Detailed error reporting for debugging
        if result.is_ok() {
            println!("Warning: Write succeeded despite read-only permissions");
            println!(
                "Current /etc permissions: {:o}",
                fs::metadata(&etc_path).await?.permissions().mode()
            );
            if etc_path.join("fstab").exists() {
                println!(
                    "fstab file was created with permissions: {:o}",
                    fs::metadata(etc_path.join("fstab"))
                        .await?
                        .permissions()
                        .mode()
                );
            }
        }

        assert!(
            result.is_err(),
            "Expected error when writing fstab to read-only /etc directory. \
             Current /etc permissions: {:o}",
            fs::metadata(&etc_path).await?.permissions().mode()
        );
        assert!(matches!(result.unwrap_err(), MicrosandboxError::Io(_)));

        Ok(())
    }

    #[tokio::test]
    async fn test_patch_with_hostnames() -> anyhow::Result<()> {
        use std::net::Ipv4Addr;

        // Create a temporary directory to act as our rootfs
        let root_dir = TempDir::new()?;
        let root_path = root_dir.path();

        // Create test hostname mappings
        let hostname_mappings = vec![
            (Ipv4Addr::new(192, 168, 1, 100), "host1.local".to_string()),
            (Ipv4Addr::new(192, 168, 1, 101), "host2.local".to_string()),
        ];

        // Update hosts file
        _patch_with_hostnames(root_path, &hostname_mappings).await?;

        // Verify hosts file was created with correct content
        let hosts_path = root_path.join("etc/hosts");
        assert!(hosts_path.exists());

        let hosts_content = fs::read_to_string(&hosts_path).await?;

        // Check header
        assert!(hosts_content.contains("# /etc/hosts: static table lookup for hostnames"));
        assert!(hosts_content.contains("127.0.0.1\tlocalhost"));
        assert!(hosts_content.contains("::1\tlocalhost ip6-localhost ip6-loopback"));

        // Check entries
        assert!(hosts_content.contains("192.168.1.100\thost1.local"));
        assert!(hosts_content.contains("192.168.1.101\thost2.local"));

        // Verify file permissions
        let perms = fs::metadata(&hosts_path).await?.permissions();
        assert_eq!(perms.mode() & 0o777, 0o644);

        // Test updating existing hosts file with new entries
        let new_mappings = vec![
            (Ipv4Addr::new(192, 168, 1, 100), "host1.local".to_string()), // Existing entry
            (Ipv4Addr::new(192, 168, 1, 102), "host3.local".to_string()), // New entry
        ];

        // Update hosts file again
        _patch_with_hostnames(root_path, &new_mappings).await?;

        // Verify updated content
        let updated_content = fs::read_to_string(&hosts_path).await?;

        // Should still contain original entries
        assert!(updated_content.contains("127.0.0.1\tlocalhost"));
        assert!(updated_content.contains("::1\tlocalhost ip6-localhost ip6-loopback"));

        // Should contain both old and new entries without duplicates
        assert!(updated_content.contains("192.168.1.100\thost1.local"));
        assert!(updated_content.contains("192.168.1.102\thost3.local"));

        // Count occurrences of the first IP to ensure no duplicates
        let count = updated_content
            .lines()
            .filter(|line| line.contains("192.168.1.100"))
            .count();
        assert_eq!(count, 1, "Should not have duplicate entries");

        Ok(())
    }

    #[tokio::test]
    async fn test_copy_dir_complex_permissions() -> anyhow::Result<()> {
        // Skip this test in CI environments
        if std::env::var("CI").is_ok() {
            println!("Skipping permission test in CI environment");
            return Ok(());
        }

        // Create temporary source and destination directories
        let src_root = TempDir::new()?;
        let dst_root = TempDir::new()?;

        let src_path = src_root.path();
        let dst_path = dst_root.path();

        // Create a complex nested directory structure with restrictive permissions
        // Structure:
        // src/
        //   ├── noaccess/     (0o000 - no permissions)
        //   │   ├── hidden/   (0o700 - rwx------)
        //   │   │   └── file  (0o600 - rw-------)
        //   ├── readonly/     (0o400 - r--------)
        //   │   ├── nested/   (0o500 - r-x------)
        //   │   │   └── file  (0o400 - r--------)
        //   └── normal/       (0o755 - rwxr-xr-x)
        //       └── file      (0o644 - rw-r--r--)

        // Create the directory structure first with normal permissions
        let noaccess_dir = src_path.join("noaccess");
        let hidden_dir = noaccess_dir.join("hidden");
        let hidden_file = hidden_dir.join("file");

        let readonly_dir = src_path.join("readonly");
        let nested_dir = readonly_dir.join("nested");
        let nested_file = nested_dir.join("file");

        let normal_dir = src_path.join("normal");
        let normal_file = normal_dir.join("file");

        // Create directories
        fs::create_dir_all(&hidden_dir).await?;
        fs::create_dir_all(&nested_dir).await?;
        fs::create_dir_all(&normal_dir).await?;

        // Create files with content
        fs::write(&hidden_file, "hidden content").await?;
        fs::write(&nested_file, "nested content").await?;
        fs::write(&normal_file, "normal content").await?;

        // Now set the restrictive permissions
        fs::set_permissions(&noaccess_dir, Permissions::from_mode(0o000)).await?; // ---------
        fs::set_permissions(&hidden_dir, Permissions::from_mode(0o700)).await?; // rwx------
        fs::set_permissions(&hidden_file, Permissions::from_mode(0o600)).await?; // rw-------

        fs::set_permissions(&readonly_dir, Permissions::from_mode(0o400)).await?; // r--------
        fs::set_permissions(&nested_dir, Permissions::from_mode(0o500)).await?; // r-x------
        fs::set_permissions(&nested_file, Permissions::from_mode(0o400)).await?; // r--------

        fs::set_permissions(&normal_dir, Permissions::from_mode(0o755)).await?; // rwxr-xr-x
        fs::set_permissions(&normal_file, Permissions::from_mode(0o644)).await?; // rw-r--r--

        // Verify permissions were set correctly
        let noaccess_perms = fs::metadata(&noaccess_dir).await?.permissions().mode() & 0o777;
        let readonly_perms = fs::metadata(&readonly_dir).await?.permissions().mode() & 0o777;

        println!("No access dir permissions: {:o}", noaccess_perms);
        println!("Read-only dir permissions: {:o}", readonly_perms);

        // This copy should succeed despite the restrictive permissions
        // The PermissionGuard will temporarily add necessary permissions
        copy_dir_recursive(src_path, dst_path).await?;

        // Verify the copy worked, even for directories with restrictive permissions
        let dst_noaccess_dir = dst_path.join("noaccess");
        let dst_hidden_dir = dst_noaccess_dir.join("hidden");
        let dst_hidden_file = dst_hidden_dir.join("file");

        let dst_readonly_dir = dst_path.join("readonly");
        let dst_nested_dir = dst_readonly_dir.join("nested");
        let dst_nested_file = dst_nested_dir.join("file");

        let dst_normal_dir = dst_path.join("normal");
        let dst_normal_file = dst_normal_dir.join("file");

        // Check everything was copied
        assert!(
            dst_noaccess_dir.exists(),
            "No-access directory was not copied"
        );
        assert!(dst_hidden_dir.exists(), "Hidden directory was not copied");
        assert!(dst_hidden_file.exists(), "Hidden file was not copied");

        assert!(
            dst_readonly_dir.exists(),
            "Read-only directory was not copied"
        );
        assert!(dst_nested_dir.exists(), "Nested directory was not copied");
        assert!(dst_nested_file.exists(), "Nested file was not copied");

        assert!(dst_normal_dir.exists(), "Normal directory was not copied");
        assert!(dst_normal_file.exists(), "Normal file was not copied");

        // Check file contents were preserved
        assert_eq!(
            fs::read_to_string(&dst_hidden_file).await?,
            "hidden content"
        );
        assert_eq!(
            fs::read_to_string(&dst_nested_file).await?,
            "nested content"
        );
        assert_eq!(
            fs::read_to_string(&dst_normal_file).await?,
            "normal content"
        );

        // Check permissions were preserved
        let dst_noaccess_perms =
            fs::metadata(&dst_noaccess_dir).await?.permissions().mode() & 0o777;
        let dst_hidden_perms = fs::metadata(&dst_hidden_dir).await?.permissions().mode() & 0o777;
        let dst_hidden_file_perms =
            fs::metadata(&dst_hidden_file).await?.permissions().mode() & 0o777;

        let dst_readonly_perms =
            fs::metadata(&dst_readonly_dir).await?.permissions().mode() & 0o777;
        let dst_nested_perms = fs::metadata(&dst_nested_dir).await?.permissions().mode() & 0o777;
        let dst_nested_file_perms =
            fs::metadata(&dst_nested_file).await?.permissions().mode() & 0o777;

        let dst_normal_perms = fs::metadata(&dst_normal_dir).await?.permissions().mode() & 0o777;
        let dst_normal_file_perms =
            fs::metadata(&dst_normal_file).await?.permissions().mode() & 0o777;

        // Verify all permissions were preserved
        assert_eq!(
            dst_noaccess_perms, 0o000,
            "No-access directory permissions not preserved"
        );
        assert_eq!(
            dst_hidden_perms, 0o700,
            "Hidden directory permissions not preserved"
        );
        assert_eq!(
            dst_hidden_file_perms, 0o600,
            "Hidden file permissions not preserved"
        );

        assert_eq!(
            dst_readonly_perms, 0o400,
            "Read-only directory permissions not preserved"
        );
        assert_eq!(
            dst_nested_perms, 0o500,
            "Nested directory permissions not preserved"
        );
        assert_eq!(
            dst_nested_file_perms, 0o400,
            "Nested file permissions not preserved"
        );

        assert_eq!(
            dst_normal_perms, 0o755,
            "Normal directory permissions not preserved"
        );
        assert_eq!(
            dst_normal_file_perms, 0o644,
            "Normal file permissions not preserved"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_copy_dir_nonexistent_source() -> anyhow::Result<()> {
        // Create a temporary destination directory
        let dst_root = TempDir::new()?;
        let dst_path = dst_root.path();

        // Try to copy from a non-existent source
        let src_path = PathBuf::from("/nonexistent/directory");

        // This should fail with a PathNotFound error
        let result = copy_dir_recursive(&src_path, dst_path).await;

        assert!(
            result.is_err(),
            "Expected an error when source doesn't exist"
        );
        assert!(
            matches!(result.unwrap_err(), MicrosandboxError::PathNotFound(_)),
            "Expected a PathNotFound error"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_patch_with_default_dns_settings() -> anyhow::Result<()> {
        // Create a temporary directory to act as our rootfs
        let root_dir = TempDir::new()?;
        let root_path = root_dir.path();

        // Test case 1: No existing resolv.conf
        patch_with_default_dns_settings(&[root_path.to_path_buf()]).await?;

        // Verify resolv.conf was created with correct content
        let resolv_path = root_path.join("etc/resolv.conf");
        assert!(resolv_path.exists());

        let resolv_content = fs::read_to_string(&resolv_path).await?;

        // Check content
        assert!(resolv_content.contains("# /etc/resolv.conf: DNS resolver configuration"));
        assert!(resolv_content.contains("nameserver 1.1.1.1"));
        assert!(resolv_content.contains("nameserver 8.8.8.8"));

        // Verify file permissions
        let perms = fs::metadata(&resolv_path).await?.permissions();
        assert_eq!(perms.mode() & 0o777, 0o644);

        // Test case 2: Existing resolv.conf with no nameservers
        let root_dir2 = TempDir::new()?;
        let root_path2 = root_dir2.path();
        let resolv_path2 = root_path2.join("etc/resolv.conf");
        fs::create_dir_all(resolv_path2.parent().unwrap()).await?;
        fs::write(&resolv_path2, "# Empty resolv.conf\n").await?;

        patch_with_default_dns_settings(&[root_path2.to_path_buf()]).await?;

        // Verify nameservers were added
        let content2 = fs::read_to_string(&resolv_path2).await?;
        assert!(content2.contains("nameserver 1.1.1.1"));
        assert!(content2.contains("nameserver 8.8.8.8"));

        // Test case 3: Existing resolv.conf with nameservers
        let root_dir3 = TempDir::new()?;
        let root_path3 = root_dir3.path();
        let resolv_path3 = root_path3.join("etc/resolv.conf");
        fs::create_dir_all(resolv_path3.parent().unwrap()).await?;
        fs::write(
            &resolv_path3,
            "# Existing nameservers\nnameserver 192.168.1.1\n",
        )
        .await?;

        patch_with_default_dns_settings(&[root_path3.to_path_buf()]).await?;

        // Verify content was not changed
        let content3 = fs::read_to_string(&resolv_path3).await?;
        assert!(content3.contains("nameserver 192.168.1.1"));
        assert!(!content3.contains("nameserver 1.1.1.1"));
        assert!(!content3.contains("nameserver 8.8.8.8"));

        // Test case 4: Multiple layers (overlayfs)
        let root_dir4 = TempDir::new()?;
        let lower_layer1 = root_dir4.path().join("lower1");
        let lower_layer2 = root_dir4.path().join("lower2");
        let patch_layer = root_dir4.path().join("patch");

        // Create directories
        fs::create_dir_all(&lower_layer1).await?;
        fs::create_dir_all(&lower_layer2).await?;
        fs::create_dir_all(&patch_layer).await?;

        // Test 4a: No resolv.conf in any layer
        patch_with_default_dns_settings(&[
            lower_layer1.clone(),
            lower_layer2.clone(),
            patch_layer.clone(),
        ])
        .await?;

        // Verify resolv.conf was created in patch layer only
        assert!(!lower_layer1.join("etc/resolv.conf").exists());
        assert!(!lower_layer2.join("etc/resolv.conf").exists());
        let patch_resolv = patch_layer.join("etc/resolv.conf");
        assert!(patch_resolv.exists());
        let content = fs::read_to_string(&patch_resolv).await?;
        assert!(content.contains("nameserver 1.1.1.1"));

        // Test 4b: resolv.conf exists in lower layer with nameserver
        let root_dir5 = TempDir::new()?;
        let lower_layer = root_dir5.path().join("lower");
        let patch_layer = root_dir5.path().join("patch");
        fs::create_dir_all(&lower_layer.join("etc")).await?;
        fs::create_dir_all(&patch_layer).await?;

        // Create resolv.conf in lower layer with nameserver
        fs::write(
            lower_layer.join("etc/resolv.conf"),
            "nameserver 192.168.1.1\n",
        )
        .await?;

        patch_with_default_dns_settings(&[lower_layer.clone(), patch_layer.clone()]).await?;

        // Verify no resolv.conf was created in patch layer
        assert!(!patch_layer.join("etc/resolv.conf").exists());
        let lower_content = fs::read_to_string(lower_layer.join("etc/resolv.conf")).await?;
        assert!(lower_content.contains("nameserver 192.168.1.1"));

        Ok(())
    }

    #[tokio::test]
    async fn test_patch_with_stat_override() -> anyhow::Result<()> {
        // Skip this test if no xattr support
        if !xattr::SUPPORTED_PLATFORM {
            println!("Skipping xattr test on unsupported platform");
            return Ok(());
        }

        // Create a temporary directory to act as our rootfs
        let root_dir = TempDir::new()?;
        let root_path = root_dir.path();

        // Patch with stat override
        patch_with_stat_override(root_path).await?;

        // Verify xattr was set correctly
        let xattr_value =
            xattr::get(root_path, "user.containers.override_stat").expect("Failed to get xattr");

        // Check if xattr was set and has the correct value
        assert!(xattr_value.is_some(), "xattr was not set");
        assert_eq!(xattr_value.unwrap(), b"0:0:0555", "xattr value incorrect");

        Ok(())
    }
}
