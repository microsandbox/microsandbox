use crate::error::to_napi_error;
use crate::mount_builder::JsMountBuilder;
use crate::pull_progress::JsPullProgressStream;
use crate::sandbox::Sandbox;
use crate::sandbox_builder::{JsPullProgressCreate, parse_bind_addr};
use microsandbox::sandbox::{LogLevel as RustLogLevel, RestoreBuilder};
use napi::bindgen_prelude::*;
use napi_derive::napi;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Snapshot restoration with explicit destination resource bindings.
#[napi(js_name = "RestoreBuilder")]
pub struct JsRestoreBuilder {
    inner: Option<RestoreBuilder>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

#[napi]
impl JsRestoreBuilder {
    /// Select an installed snapshot or archive; this does not start a VM.
    #[napi(constructor)]
    pub fn new(snapshot: String) -> Self {
        Self {
            inner: Some(microsandbox::Sandbox::restore(snapshot)),
        }
    }

    /// Choose the destination sandbox name.
    #[napi]
    pub fn name(&mut self, name: String) -> Result<&Self> {
        let inner = self.take_inner()?;
        self.inner = Some(inner.name(name));
        Ok(self)
    }

    /// Explicitly reuse locally validated source resource bindings.
    #[napi]
    pub fn dangerously_inherit_resources(&mut self) -> Result<&Self> {
        let inner = self.take_inner()?;
        self.inner = Some(inner.dangerously_inherit_resources());
        Ok(self)
    }
    /// Supply the base for omitted disk layers and RAM objects in a snapshot archive.
    #[napi]
    pub fn snapshot_base(&mut self, base: String) -> Result<&Self> {
        let prev = self.take_inner()?;
        self.inner = Some(prev.snapshot_base(base));
        Ok(self)
    }

    /// Cold-boot only the disk state carried by a full snapshot.
    #[napi(js_name = "diskOnly")]
    pub fn disk_only(&mut self) -> Result<&Self> {
        let prev = self.take_inner()?;
        self.inner = Some(prev.disk_only());
        Ok(self)
    }

    /// Restore a full snapshot with private copy-on-write memory.
    #[napi]
    pub fn forked(&mut self) -> Result<&Self> {
        let prev = self
            .inner
            .take()
            .ok_or_else(|| napi::Error::from_reason("builder already consumed"))?;
        self.inner = Some(prev.forked());
        Ok(self)
    }

    /// Validate authorized filesystem mappings strictly (default) or allow supported mismatches.
    /// Neither policy inherits resources; unmapped filesystems remain unavailable.
    #[napi(ts_args_type = "policy: 'strict' | 'relaxed'")]
    pub fn external_mount_policy(&mut self, policy: String) -> Result<&Self> {
        let policy = match policy.as_str() {
            "strict" => microsandbox::sandbox::ExternalMountRestorePolicy::Strict,
            "relaxed" => microsandbox::sandbox::ExternalMountRestorePolicy::Relaxed,
            _ => {
                return Err(napi::Error::from_reason(
                    "external mount policy must be strict or relaxed",
                ));
            }
        };
        let previous = self.take_inner()?;
        self.inner = Some(previous.external_mount_policy(policy));
        Ok(self)
    }

    /// Override log verbosity: `"trace" | "debug" | "info" | "warn" | "error"`.
    #[napi(js_name = "logLevel")]
    pub fn log_level(&mut self, level: String) -> Result<&Self> {
        let l = match level.as_str() {
            "trace" => RustLogLevel::Trace,
            "debug" => RustLogLevel::Debug,
            "info" => RustLogLevel::Info,
            "warn" => RustLogLevel::Warn,
            "error" => RustLogLevel::Error,
            other => {
                return Err(napi::Error::from_reason(format!(
                    "invalid log level `{other}`"
                )));
            }
        };
        let prev = self.take_inner()?;
        self.inner = Some(prev.log_level(l));
        Ok(self)
    }

    /// Default running user.
    #[napi]
    pub fn user(&mut self, user: String) -> Result<&Self> {
        let prev = self.take_inner()?;
        self.inner = Some(prev.user(user));
        Ok(self)
    }

    /// Configure a volume mount via a callback. The callback receives a
    /// `MountBuilder` already pre-bound to `guestPath`.
    #[napi]
    pub fn volume(
        &mut self,
        env: &Env,
        guest_path: String,
        configure: Function<ClassInstance<JsMountBuilder>, ClassInstance<JsMountBuilder>>,
    ) -> Result<&Self> {
        let initial = JsMountBuilder::new(guest_path.clone()).into_instance(env)?;
        let mut returned = configure.call(initial)?;
        let mount_builder = returned.take_inner_builder()?;
        let prev = self.take_inner()?;
        // The core's volume() signature is volume(guest_path, FnOnce(MountBuilder) -> MountBuilder).
        // The MountBuilder we hand back already encodes the guest path
        // (we constructed it that way above); the default supplied by
        // the core is discarded.
        self.inner = Some(prev.volume(guest_path, |_default| mount_builder));
        Ok(self)
    }

    /// Publish a TCP port from host -> guest.
    #[napi]
    pub fn port(&mut self, host_port: u32, guest_port: u32) -> Result<&Self> {
        let h = u16::try_from(host_port)
            .map_err(|_| napi::Error::from_reason("host port out of range"))?;
        let g = u16::try_from(guest_port)
            .map_err(|_| napi::Error::from_reason("guest port out of range"))?;
        let prev = self.take_inner()?;
        self.inner = Some(prev.port(h, g));
        Ok(self)
    }

    /// Publish a TCP port from host -> guest on a specific host bind address.
    #[napi(js_name = "portBind")]
    pub fn port_bind(&mut self, bind: String, host_port: u32, guest_port: u32) -> Result<&Self> {
        let bind = parse_bind_addr(&bind)?;
        let h = u16::try_from(host_port)
            .map_err(|_| napi::Error::from_reason("host port out of range"))?;
        let g = u16::try_from(guest_port)
            .map_err(|_| napi::Error::from_reason("guest port out of range"))?;
        let prev = self.take_inner()?;
        self.inner = Some(prev.port_bind(bind, h, g));
        Ok(self)
    }

    /// Publish a UDP port from host -> guest.
    #[napi(js_name = "portUdp")]
    pub fn port_udp(&mut self, host_port: u32, guest_port: u32) -> Result<&Self> {
        let h = u16::try_from(host_port)
            .map_err(|_| napi::Error::from_reason("host port out of range"))?;
        let g = u16::try_from(guest_port)
            .map_err(|_| napi::Error::from_reason("guest port out of range"))?;
        let prev = self.take_inner()?;
        self.inner = Some(prev.port_udp(h, g));
        Ok(self)
    }

    /// Publish a UDP port from host -> guest on a specific host bind address.
    #[napi(js_name = "portUdpBind")]
    pub fn port_udp_bind(
        &mut self,
        bind: String,
        host_port: u32,
        guest_port: u32,
    ) -> Result<&Self> {
        let bind = parse_bind_addr(&bind)?;
        let h = u16::try_from(host_port)
            .map_err(|_| napi::Error::from_reason("host port out of range"))?;
        let g = u16::try_from(guest_port)
            .map_err(|_| napi::Error::from_reason("guest port out of range"))?;
        let prev = self.take_inner()?;
        self.inner = Some(prev.port_udp_bind(bind, h, g));
        Ok(self)
    }

    /// Expose a host Unix stream socket or local Windows named pipe on a guest-to-host vsock port.
    #[napi]
    pub fn vsock(&mut self, host_path: String, port: u32) -> Result<&Self> {
        let prev = self.take_inner()?;
        self.inner = Some(prev.vsock(host_path, port));
        Ok(self)
    }

    /// Expose a host Unix datagram socket on a guest-to-host vsock port.
    #[napi(js_name = "vsockDgram")]
    pub fn vsock_dgram(&mut self, host_path: String, port: u32) -> Result<&Self> {
        let prev = self.take_inner()?;
        self.inner = Some(prev.vsock_dgram(host_path, port));
        Ok(self)
    }

    /// Restore a detached sandbox and wait until ready.
    ///
    /// # Safety
    /// The builder is consumed before suspension; callers must not reuse it.
    #[napi]
    pub async unsafe fn restore(&mut self) -> Result<Sandbox> {
        let inner = self.take_inner()?;
        Ok(Sandbox::from_rust(
            inner.restore().await.map_err(to_napi_error)?,
        ))
    }

    /// Restore with image, snapshot preparation and activation progress.
    ///
    /// # Safety
    /// The builder is consumed before suspension; callers must not reuse it.
    #[napi]
    pub async unsafe fn restore_with_progress(&mut self) -> Result<JsPullProgressCreate> {
        let (handle, task) = self
            .take_inner()?
            .restore_with_progress()
            .map_err(to_napi_error)?;
        Ok(JsPullProgressCreate {
            stream: JsPullProgressStream::from_creation(handle),
            abort: task.abort_handle(),
            task: std::sync::Arc::new(tokio::sync::Mutex::new(Some(task))),
        })
    }
}

impl JsRestoreBuilder {
    fn take_inner(&mut self) -> Result<RestoreBuilder> {
        self.inner
            .take()
            .ok_or_else(|| napi::Error::from_reason("RestoreBuilder already consumed"))
    }
}
