//! Container image management for Microsandbox.
//!
//! This module provides functionality for managing container images from various
//! registries. It supports pulling images from Docker and Sandboxes.io registries,
//! handling image layers, and managing the local image cache.

use crate::{
    management::db::{self, OCI_DB_MIGRATOR},
    oci::{DockerRegistry, OciRegistryPull, Reference},
    MicrosandboxError, MicrosandboxResult,
};
#[cfg(feature = "cli")]
use flate2::read::GzDecoder;
use futures::future;
#[cfg(feature = "cli")]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(feature = "cli")]
use microsandbox_utils::term::{self, MULTI_PROGRESS};
use microsandbox_utils::{env, EXTRACTED_LAYER_SUFFIX, LAYERS_SUBDIR, OCI_DB_FILENAME};
use sqlx::{Pool, Sqlite};
#[cfg(feature = "cli")]
use std::io::{Read, Result as IoResult};
use std::path::{Path, PathBuf};
#[cfg(feature = "cli")]
use tar::Archive;
use tempfile::tempdir;
use tokio::fs;
#[cfg(feature = "cli")]
use tokio::task::spawn_blocking;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// The domain name for the Docker registry.
const DOCKER_REGISTRY: &str = "docker.io";

/// The domain name for the Sandboxes registry.
const SANDBOXES_REGISTRY: &str = "sandboxes.io";

#[cfg(feature = "cli")]
/// Spinner message used for extracting layers.
const EXTRACT_LAYERS_MSG: &str = "Extracting layers";

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Pulls an image or image group from a supported registry (Docker or Sandboxes.io).
///
/// This function handles pulling container images from different registries based on the provided
/// parameters. It supports both single image pulls and image group pulls (for Sandboxes.io registry only).
///
/// For Sandboxes.io registry:
/// - Library repository images are pulled from Docker registry for compatibility
/// - Other namespaces are also pulled from Docker registry with a warning about potential future changes
///
/// ## Arguments
///
/// * `name` - The reference to the image or image group to pull
/// * `image` - If true, indicates that a single image should be pulled
/// * `image_group` - If true, indicates that an image group should be pulled (Sandboxes.io only)
/// * `layer_path` - The path to store the layer files
///
/// ## Errors
///
/// Returns an error in the following cases:
/// * Both `image` and `image_group` are true (invalid combination)
/// * Image group pull is requested for a non-Sandboxes.io registry
/// * Unsupported registry is specified
/// * Registry-specific pull operations fail
///
/// # Examples
///
/// ```no_run
/// use microsandbox_core::management::image;
/// use microsandbox_core::oci::Reference;
/// use std::path::PathBuf;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// // Pull a single image from Docker registry
/// image::pull("docker.io/library/ubuntu:latest".parse().unwrap(), true, false, None).await?;
///
/// // Pull an image from Sandboxes.io registry
/// image::pull("sandboxes.io/library/alpine:latest".parse().unwrap(), true, false, None).await?;
///
/// // Pull an image from the default registry (when no registry is specified in the reference)
/// image::pull("nginx:latest".parse().unwrap(), true, false, None).await?;
///
/// // You can set the OCI_REGISTRY_DOMAIN environment variable to specify your default registry
/// std::env::set_var("OCI_REGISTRY_DOMAIN", "docker.io");
/// image::pull("alpine:latest".parse().unwrap(), true, false, None).await?;
///
/// // Pull an image from Docker registry and store the layers in a custom directory
/// image::pull("docker.io/library/ubuntu:latest".parse().unwrap(), true, Some(PathBuf::from("/custom/path"))).await?;
/// # Ok(())
/// # }
/// ```
pub async fn pull(
    name: Reference,
    image: bool,
    layer_path: Option<PathBuf>,
) -> MicrosandboxResult<()> {
    // Image must be true
    if !image {
        return Err(MicrosandboxError::InvalidArgument(
            "image must be true".to_string(),
        ));
    }

    // Single image pull mode (default if both flags are false, or if image is true)
    let registry = name.to_string().split('/').next().unwrap_or("").to_string();
    let temp_download_dir = tempdir()?.into_path();

    tracing::info!(
        "temporary download directory: {}",
        temp_download_dir.display()
    );

    if registry == DOCKER_REGISTRY {
        pull_from_docker_registry(&name, &temp_download_dir, layer_path).await
    } else if registry == SANDBOXES_REGISTRY {
        pull_from_sandboxes_registry(&name, &temp_download_dir, layer_path).await
    } else {
        Err(MicrosandboxError::InvalidArgument(format!(
            "Unsupported registry: {}",
            registry
        )))
    }
}

/// Pulls a single image from the Docker registry.
///
/// ## Arguments
///
/// * `image` - The reference to the Docker image to pull
/// * `download_dir` - The directory to download the image layers to
/// * `layer_path` - Optional custom path to store layers
///
/// ## Errors
///
/// Returns an error if:
/// * Failed to create temporary directories
/// * Failed to initialize Docker registry client
/// * Failed to pull the image from Docker registry
pub async fn pull_from_docker_registry(
    image: &Reference,
    download_dir: impl AsRef<Path>,
    layer_path: Option<PathBuf>,
) -> MicrosandboxResult<()> {
    let download_dir = download_dir.as_ref();
    let microsandbox_home_path = env::get_microsandbox_home_path();
    let db_path = microsandbox_home_path.join(OCI_DB_FILENAME);

    // Use custom layer_path if specified, otherwise use default microsandbox layers directory
    let layers_dir = match layer_path {
        Some(path) => path,
        None => microsandbox_home_path.join(LAYERS_SUBDIR),
    };

    // Create layers directory if it doesn't exist
    fs::create_dir_all(&layers_dir).await?;

    let docker_registry = DockerRegistry::new(download_dir, &db_path).await?;

    // Get or create a connection pool to the database
    let pool = db::get_or_create_pool(&db_path, &OCI_DB_MIGRATOR).await?;

    // Check if we need to pull the image
    if check_image_layers(&pool, image, &layers_dir).await? {
        tracing::info!("image {} and all its layers exist, skipping pull", image);
        return Ok(());
    }

    docker_registry
        .pull_image(image.get_repository(), image.get_selector().clone())
        .await?;

    // Find and extract layers in parallel
    let layer_paths = collect_layer_files(download_dir).await?;

    #[cfg(feature = "cli")]
    let extract_layers_sp = term::create_spinner(
        EXTRACT_LAYERS_MSG.to_string(),
        None,
        Some(layer_paths.len() as u64),
    );

    let extraction_futures: Vec<_> = layer_paths
        .into_iter()
        .map(|path| {
            let layers_dir = layers_dir.clone();
            #[cfg(feature = "cli")]
            let extract_layers_sp = extract_layers_sp.clone();
            async move {
                let result = extract_layer(path, &layers_dir).await;
                #[cfg(feature = "cli")]
                extract_layers_sp.inc(1);
                result
            }
        })
        .collect();

    // Wait for all extractions to complete
    for result in future::join_all(extraction_futures).await {
        result?;
    }

    #[cfg(feature = "cli")]
    extract_layers_sp.finish();

    Ok(())
}

/// Pulls a single image from the Sandboxes.io registry.
///
/// For library repository images, this function delegates to `pull_from_docker_registry` for compatibility.
/// For other namespaces, it also uses Docker registry but displays a warning about potential future changes.
///
/// ## Arguments
///
/// * `image` - The reference to the Sandboxes.io image to pull
/// * `download_dir` - The directory to download the image layers to
/// * `layer_path` - Optional custom path to store layers
///
/// ## Errors
///
/// Returns an error if the underlying Docker registry pull fails
pub async fn pull_from_sandboxes_registry(
    image: &Reference,
    download_dir: impl AsRef<Path>,
    layer_path: Option<PathBuf>,
) -> MicrosandboxResult<()> {
    // Check if this is a library repository image
    let repository = image.get_repository();

    // Create a Docker reference string using the original repository but with docker.io registry
    // Format: docker.io/repository:tag
    let docker_ref_str = format!(
        "{}/{}",
        DOCKER_REGISTRY,
        image
            .to_string()
            .split('/')
            .skip(1)
            .collect::<Vec<&str>>()
            .join("/")
    );
    let docker_reference: Reference = docker_ref_str.parse()?;

    if repository.starts_with("library/") {
        tracing::info!("pulling library image from Docker registry for compatibility");
    } else {
        tracing::warn!(
            "Non-library namespace image requested from Sandboxes registry: {}",
            repository
        );
        tracing::warn!(
            "Currently using Docker registry for compatibility, but namespace mappings may change in the future"
        );
        tracing::info!(
            "To ensure consistent behavior, consider setting OCI_REGISTRY_DOMAIN=docker.io if you want to use Docker registry consistently"
        );
    }

    pull_from_docker_registry(&docker_reference, download_dir, layer_path).await
}

/// Pulls an image group from the Sandboxes.io registry.
///
/// ## Arguments
///
/// * `group` - The reference to the image group to pull
/// ## Errors
///
/// Returns an error if:
/// * Sandboxes registry image group pull is not implemented
pub async fn pull_group_from_sandboxes_registry(_group: &Reference) -> MicrosandboxResult<()> {
    return Err(MicrosandboxError::NotImplemented(
        "Sandboxes registry image group pull is not implemented".to_string(),
    ));
}

//--------------------------------------------------------------------------------------------------
// Functions: Helpers
//--------------------------------------------------------------------------------------------------

/// Checks if all layers for an image exist in both the database and the layers directory.
///
/// ## Arguments
///
/// * `pool` - The database connection pool
/// * `image` - The reference to the image to check
/// * `layers_dir` - The directory where layers should be stored
///
/// ## Returns
///
/// Returns Ok(true) if all layers exist and are valid, Ok(false) if any layers are missing
/// or invalid. Any errors during the check process will return Ok(false) with a warning log.
async fn check_image_layers(
    pool: &Pool<Sqlite>,
    image: &Reference,
    layers_dir: impl AsRef<Path>,
) -> MicrosandboxResult<bool> {
    let layers_dir = layers_dir.as_ref();

    // Check if the image exists in the database
    match db::image_exists(pool, &image.to_string()).await {
        Ok(true) => {
            // Image exists, get all layer digests for this image
            match db::get_image_layer_digests(pool, &image.to_string()).await {
                Ok(layer_digests) => {
                    tracing::info!("layer_digests: {:?}", layer_digests);
                    if layer_digests.is_empty() {
                        tracing::warn!("no layers found for image {}", image);
                        return Ok(false);
                    }

                    // Check if all layers exist in the layers directory
                    for digest in &layer_digests {
                        let layer_path =
                            layers_dir.join(format!("{}.{}", digest, EXTRACTED_LAYER_SUFFIX));
                        if !layer_path.exists() {
                            tracing::warn!("layer {} not found in layers directory", digest);
                            return Ok(false);
                        }

                        // Also check that the layer directory actually has content
                        let mut read_dir = fs::read_dir(&layer_path).await?;
                        let dir_empty = read_dir.next_entry().await?.is_none();
                        if dir_empty {
                            tracing::warn!("layer {} exists but is empty", digest);
                        }

                        tracing::info!("layer {} found in layers directory", digest);
                    }

                    // Get the layers from database to verify database records exist for all digests
                    let db_layers = db::get_layers_by_digest(pool, &layer_digests).await?;

                    if db_layers.len() < layer_digests.len() {
                        tracing::warn!(
                            "some layers for image {} exist on disk but missing in db",
                            image
                        );
                        return Ok(false);
                    }

                    tracing::info!("all layers for image {} exist and are valid", image);
                    Ok(true)
                }
                Err(e) => {
                    tracing::warn!("error checking layer digests: {}, will pull image", e);
                    Ok(false)
                }
            }
        }
        Ok(false) => {
            tracing::warn!("image {} does not exist in db, will pull image", image);
            Ok(false)
        }
        Err(e) => {
            tracing::warn!("error checking image existence: {}, will pull image", e);
            Ok(false)
        }
    }
}

/// Extracts a layer from the downloaded tar.gz file into an extracted directory.
/// The extracted directory will be named as <layer-name>.extracted
async fn extract_layer(
    layer_path: impl AsRef<std::path::Path>,
    extract_base_dir: impl AsRef<Path>,
) -> MicrosandboxResult<()> {
    let layer_path = layer_path.as_ref();
    let file_name = layer_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| MicrosandboxError::LayerHandling {
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "invalid layer file name"),
            layer: layer_path.display().to_string(),
        })?;

    // Create the extraction directory with name <layer-name>.extracted
    let extract_dir = extract_base_dir
        .as_ref()
        .join(format!("{}.{}", file_name, EXTRACTED_LAYER_SUFFIX));

    // Check if the layer is already extracted
    if extract_dir.exists() {
        // Check if the directory has content (not empty)
        let mut read_dir =
            fs::read_dir(&extract_dir)
                .await
                .map_err(|e| MicrosandboxError::LayerHandling {
                    source: e,
                    layer: file_name.to_string(),
                })?;

        if read_dir.next_entry().await?.is_some() {
            tracing::info!(
                "layer {} already extracted at {}, skipping extraction",
                file_name,
                extract_dir.display()
            );
            return Ok(());
        }
    }

    fs::create_dir_all(&extract_dir)
        .await
        .map_err(|e| MicrosandboxError::LayerHandling {
            source: e,
            layer: file_name.to_string(),
        })?;

    tracing::info!(
        "extracting layer {} to {}",
        file_name,
        extract_dir.display()
    );

    #[cfg(feature = "cli")]
    struct ProgressReader<R> {
        inner: R,
        bar: ProgressBar,
    }

    #[cfg(feature = "cli")]
    impl<R: Read> Read for ProgressReader<R> {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            let n = self.inner.read(buf)?;
            if n > 0 {
                self.bar.inc(n as u64);
            }
            Ok(n)
        }
    }

    #[cfg(feature = "cli")]
    {
        let total_bytes = fs::metadata(layer_path).await?.len();
        let pb = MULTI_PROGRESS.add(ProgressBar::new(total_bytes));
        pb.set_style(
            ProgressStyle::with_template(
                "{prefix:.bold.dim} {bar:40.green/green.dim} {bytes:.bold}/{total_bytes:.dim}",
            )
            .unwrap()
            .progress_chars("=+-"),
        );
        let digest_short = if let Some(rest) = file_name.strip_prefix("sha256:") {
            &rest[..8.min(rest.len())]
        } else {
            &file_name[..8.min(file_name.len())]
        };
        pb.set_prefix(format!("{}", digest_short));

        let layer_path_clone = layer_path.to_path_buf();
        let extract_dir_clone = extract_dir.clone();
        let pb_clone = pb.clone();

        spawn_blocking(move || -> MicrosandboxResult<()> {
            let file = std::fs::File::open(&layer_path_clone)?;
            let reader = ProgressReader {
                inner: file,
                bar: pb_clone.clone(),
            };
            let decoder = GzDecoder::new(reader);
            let mut archive = Archive::new(decoder);
            archive.unpack(&extract_dir_clone)?;
            Ok(())
        })
        .await
        .map_err(|e| MicrosandboxError::LayerExtraction(format!("{:?}", e)))??;

        pb.finish_and_clear();
    }

    #[cfg(not(feature = "cli"))]
    {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let file =
            std::fs::File::open(layer_path).map_err(|e| MicrosandboxError::LayerHandling {
                source: e,
                layer: file_name.to_string(),
            })?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        archive
            .unpack(&extract_dir)
            .map_err(|e| MicrosandboxError::LayerExtraction(format!("{:?}", e)))?;
    }

    tracing::info!(
        "successfully extracted layer {} to {}",
        file_name,
        extract_dir.display()
    );
    Ok(())
}

/// Collects all layer files in the given directory that start with "sha256:".
async fn collect_layer_files(dir: impl AsRef<Path>) -> MicrosandboxResult<Vec<PathBuf>> {
    let mut layer_paths = Vec::new();
    let mut read_dir = fs::read_dir(dir).await?;

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("sha256:") {
                    layer_paths.push(path.clone());
                }
            }
        }
    }

    tracing::info!("found {} layers to extract", layer_paths.len());
    Ok(layer_paths)
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test_log::test(tokio::test)]
    #[ignore = "makes network requests to Docker registry to pull an image"]
    async fn test_image_pull_from_docker_registry() -> MicrosandboxResult<()> {
        // Create temporary directories for test
        let temp_dir = TempDir::new()?;
        let microsandbox_home = temp_dir.path().join("microsandbox_home");
        let download_dir = temp_dir.path().join("download");
        fs::create_dir_all(&microsandbox_home).await?;
        fs::create_dir_all(&download_dir).await?;

        // Set up test environment
        std::env::set_var("MICROSANDBOX_HOME", microsandbox_home.to_str().unwrap());

        // Create test image reference (using a small image for faster tests)
        let image_ref: Reference = "docker.io/library/nginx:stable-alpine".parse().unwrap();

        // Call the function under test
        pull_from_docker_registry(&image_ref, &download_dir, None).await?;

        // Initialize database connection for verification
        let db_path = microsandbox_home.join(OCI_DB_FILENAME);
        let pool = db::get_or_create_pool(&db_path, &OCI_DB_MIGRATOR).await?;

        // Verify image exists in database
        let image_exists = db::image_exists(&pool, &image_ref.to_string()).await?;
        assert!(image_exists, "Image should exist in database");

        // Verify layers directory exists and contains extracted layers
        let layers_dir = microsandbox_home.join(LAYERS_SUBDIR);
        assert!(layers_dir.exists(), "Layers directory should exist");

        // Verify extracted layer directories exist
        let mut entries = fs::read_dir(&layers_dir).await?;
        let mut found_extracted_layers = false;
        while let Some(entry) = entries.next_entry().await? {
            if entry
                .file_name()
                .to_string_lossy()
                .ends_with(EXTRACTED_LAYER_SUFFIX)
            {
                found_extracted_layers = true;
                assert!(
                    entry.path().is_dir(),
                    "Extracted layer path should be a directory"
                );
            }
        }
        assert!(
            found_extracted_layers,
            "Should have found extracted layer directories"
        );

        // Verify nginx files exist in the extracted layers
        helper::verify_nginx_files(&layers_dir).await?;

        Ok(())
    }
}

#[cfg(test)]
mod helper {
    use super::*;

    /// Helper function to verify that all expected nginx files exist in the extracted layers
    pub(super) async fn verify_nginx_files(layers_dir: impl AsRef<Path>) -> MicrosandboxResult<()> {
        let mut found_nginx_conf = false;
        let mut found_default_conf = false;
        let mut found_nginx_binary = false;

        // Check each extracted layer directory for nginx files
        let mut entries = fs::read_dir(layers_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if !entry
                .file_name()
                .to_string_lossy()
                .ends_with(EXTRACTED_LAYER_SUFFIX)
            {
                continue;
            }

            let layer_path = entry.path();
            tracing::info!("checking layer: {}", layer_path.display());

            // Check for nginx.conf
            let nginx_conf = layer_path.join("etc").join("nginx").join("nginx.conf");
            if nginx_conf.exists() {
                found_nginx_conf = true;
                tracing::info!("found nginx.conf at {}", nginx_conf.display());
            }

            // Check for default.conf
            let default_conf = layer_path
                .join("etc")
                .join("nginx")
                .join("conf.d")
                .join("default.conf");
            if default_conf.exists() {
                found_default_conf = true;
                tracing::info!("found default.conf at {}", default_conf.display());
            }

            // Check for nginx binary
            let nginx_binary = layer_path.join("usr").join("sbin").join("nginx");
            if nginx_binary.exists() {
                found_nginx_binary = true;
                tracing::info!("found nginx binary at {}", nginx_binary.display());
            }

            // If we found all files, we can stop checking
            if found_nginx_conf && found_default_conf && found_nginx_binary {
                break;
            }
        }

        // Assert that we found all the expected files
        assert!(
            found_nginx_conf,
            "nginx.conf should exist in one of the layers"
        );
        assert!(
            found_default_conf,
            "default.conf should exist in one of the layers"
        );
        assert!(
            found_nginx_binary,
            "nginx binary should exist in one of the layers"
        );

        Ok(())
    }
}
