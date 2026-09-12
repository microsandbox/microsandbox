//! Dedicated restore entry point; unknown creation options are rejected at the FFI boundary.

use std::collections::HashMap;
use std::os::raw::{c_char, c_uchar};

use microsandbox::Sandbox;
use microsandbox::sandbox::{ExternalMountRestorePolicy, RestoreBuilder};

use super::{
    FfiError, MountSpec, PortBindingOpts, VsockRouteOpts, cstr, parse_log_level, register, run_c,
    volume_mount,
};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RestoreOptions {
    snapshot: String,
    creation_progress: Option<u64>,
    #[serde(default)]
    forked: bool,
    #[serde(default)]
    disk_only: bool,
    snapshot_base: Option<String>,
    user: Option<String>,
    log_level: Option<String>,
    external_mount_policy: Option<ExternalMountRestorePolicy>,
    #[serde(default)]
    dangerously_inherit_resources: bool,
    #[serde(default)]
    volumes: HashMap<String, MountSpec>,
    #[serde(default)]
    captured_volumes: Vec<String>,
    #[serde(default)]
    ports: Vec<PortBindingOpts>,
    #[serde(default)]
    vsock: Vec<VsockRouteOpts>,
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

fn builder(name: String, opts: &RestoreOptions) -> Result<RestoreBuilder, FfiError> {
    let mut builder = Sandbox::restore(&opts.snapshot).name(name);
    if opts.forked {
        builder = builder.forked();
    }
    if opts.disk_only {
        builder = builder.disk_only();
    }
    if let Some(base) = &opts.snapshot_base {
        builder = builder.snapshot_base(base);
    }
    if let Some(user) = &opts.user {
        builder = builder.user(user);
    }
    if let Some(level) = &opts.log_level {
        builder = builder.log_level(parse_log_level(level)?);
    }
    if let Some(policy) = opts.external_mount_policy {
        builder = builder.external_mount_policy(policy);
    }
    if opts.dangerously_inherit_resources {
        builder = builder.dangerously_inherit_resources();
    }
    for (guest, spec) in &opts.volumes {
        let mount = volume_mount(guest, spec)?;
        builder = builder.volume(guest, |_| mount);
    }
    for guest in &opts.captured_volumes {
        builder = builder.volume(guest, |m| m.captured());
    }
    for port in &opts.ports {
        let bind = port
            .bind
            .parse()
            .map_err(|_| FfiError::invalid_argument("invalid restore bind address"))?;
        builder = match port.protocol.as_str() {
            "tcp" => builder.port_bind(bind, port.host_port, port.guest_port),
            "udp" => builder.port_udp_bind(bind, port.host_port, port.guest_port),
            _ => return Err(FfiError::invalid_argument("invalid restore port protocol")),
        };
    }
    for route in &opts.vsock {
        builder = match route.socket_type.as_str() {
            "stream" => builder.vsock(&route.host_socket, route.port),
            "dgram" => builder.vsock_dgram(&route.host_socket, route.port),
            _ => {
                return Err(FfiError::invalid_argument(
                    "invalid restore vsock socket type",
                ));
            }
        };
    }
    Ok(builder)
}

/// Restore a detached sandbox through a dedicated C entry point.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msb_sandbox_restore(
    cancel_id: u64,
    name: *const c_char,
    opts_json: *const c_char,
    buf: *mut c_uchar,
    buf_len: usize,
) -> *mut c_char {
    run_c(cancel_id, buf, buf_len, || {
        let name = unsafe { cstr(name) }?;
        let options: RestoreOptions = serde_json::from_str(&unsafe { cstr(opts_json) }?)
            .map_err(|e| FfiError::invalid_argument(format!("invalid restore options: {e}")))?;
        let builder = builder(name, &options)?;
        Ok(Box::pin(async move {
            let sandbox = if let Some(progress) = options.creation_progress {
                super::creation_progress::restore(builder, progress).await?
            } else {
                builder.restore().await?
            };
            let backend_kind = sandbox.backend_kind().as_str();
            let id = sandbox.id().to_string();
            let handle = register(sandbox)?;
            Ok(
                serde_json::json!({"handle":handle,"backend_kind":backend_kind,"id":id})
                    .to_string(),
            )
        }))
    })
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restore_rejects_fresh_boot_options() {
        for field in ["image", "memory_mib", "cmd", "replace", "detached"] {
            let mut value = serde_json::json!({"snapshot":"saved"});
            value[field] = serde_json::Value::Null;
            assert!(
                serde_json::from_value::<RestoreOptions>(value).is_err(),
                "{field}"
            );
        }
    }

    #[test]
    fn create_rejects_discarded_restore_fields() {
        for field in ["snapshot", "snapshot_base", "snapshot_disk_only", "forked"] {
            let mut value = serde_json::json!({"image":"alpine"});
            value[field] = serde_json::Value::Null;
            assert!(
                serde_json::from_value::<super::super::SandboxCreateOpts>(value).is_err(),
                "{field}"
            );
        }
    }
}
