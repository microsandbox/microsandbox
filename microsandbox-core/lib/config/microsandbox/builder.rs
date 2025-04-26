use std::collections::HashMap;

use microsandbox_utils::DEFAULT_SHELL;
use semver::Version;
use typed_path::Utf8UnixPathBuf;

use crate::{
    config::{EnvPair, PathPair, PortPair, ReferenceOrPath},
    MicrosandboxResult,
};

use super::{Build, Group, Meta, Microsandbox, Module, NetworkScope, Sandbox, SandboxGroup};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Builder for Microsandbox configuration
///
/// ### Optional fields:
/// - `meta`: The metadata for the configuration
/// - `modules`: The modules to import
/// - `builds`: The builds to run
/// - `sandboxes`: The sandboxes to run
/// - `groups`: The groups to run the sandboxes in
#[derive(Default)]
pub struct MicrosandboxBuilder {
    meta: Option<Meta>,
    modules: HashMap<String, Module>,
    builds: HashMap<String, Build>,
    sandboxes: HashMap<String, Sandbox>,
    groups: HashMap<String, Group>,
}

/// Builder for Sandbox configuration
///
/// ### Required fields:
/// - `name`: The name of the sandbox
/// - `image`: The image to use
///
/// ### Optional fields:
/// - `version`: The version of the sandbox
/// - `meta`: The metadata for the sandbox
/// - `memory`: The maximum amount of memory allowed for the sandbox
/// - `cpus`: The maximum number of CPUs allowed for the sandbox
/// - `volumes`: The volumes to mount
/// - `ports`: The ports to expose
/// - `envs`: The environment variables to use
/// - `env_file`: The environment file to use
/// - `groups`: The groups to run the sandbox in
/// - `depends_on`: The sandboxes to depend on
/// - `workdir`: The working directory to use
/// - `shell`: The shell to use
/// - `scripts`: The scripts available in the sandbox
/// - `imports`: The files to import
/// - `exports`: The files to export
/// - `scope`: The network scope for the sandbox
/// - `proxy`: The proxy to use
pub struct SandboxBuilder<I> {
    version: Option<Version>,
    meta: Option<Meta>,
    image: I,
    memory: Option<u32>,
    cpus: Option<u8>,
    volumes: Vec<PathPair>,
    ports: Vec<PortPair>,
    envs: Vec<EnvPair>,
    env_file: Option<Utf8UnixPathBuf>,
    groups: HashMap<String, SandboxGroup>,
    depends_on: Vec<String>,
    workdir: Option<Utf8UnixPathBuf>,
    shell: Option<String>,
    scripts: HashMap<String, String>,
    exec: Option<String>,
    imports: HashMap<String, Utf8UnixPathBuf>,
    exports: HashMap<String, Utf8UnixPathBuf>,
    scope: NetworkScope,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl MicrosandboxBuilder {
    /// Sets the metadata for the configuration
    pub fn meta(mut self, meta: Meta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Sets the modules to import
    pub fn modules(mut self, modules: impl IntoIterator<Item = (String, Module)>) -> Self {
        self.modules = modules.into_iter().collect();
        self
    }

    /// Sets the builds to run
    pub fn builds(mut self, builds: impl IntoIterator<Item = (String, Build)>) -> Self {
        self.builds = builds.into_iter().collect();
        self
    }

    /// Sets the sandboxes to run
    pub fn sandboxes(mut self, sandboxes: impl IntoIterator<Item = (String, Sandbox)>) -> Self {
        self.sandboxes = sandboxes.into_iter().collect();
        self
    }

    /// Sets the groups to run the sandboxes in
    pub fn groups(mut self, groups: impl IntoIterator<Item = (String, Group)>) -> Self {
        self.groups = groups.into_iter().collect();
        self
    }

    /// Builds the Microsandbox configuration with validation
    pub fn build(self) -> MicrosandboxResult<Microsandbox> {
        let microsandbox = self.build_unchecked();
        microsandbox.validate()?;
        Ok(microsandbox)
    }

    /// Builds the Microsandbox configuration without validation
    pub fn build_unchecked(self) -> Microsandbox {
        Microsandbox {
            meta: self.meta,
            modules: self.modules,
            builds: self.builds,
            sandboxes: self.sandboxes,
            groups: self.groups,
        }
    }
}

impl<I> SandboxBuilder<I> {
    /// Sets the version of the sandbox
    pub fn version(mut self, version: impl Into<Version>) -> SandboxBuilder<I> {
        self.version = Some(version.into());
        self
    }

    /// Sets the metadata for the sandbox
    pub fn meta(mut self, meta: Meta) -> SandboxBuilder<I> {
        self.meta = Some(meta);
        self
    }

    /// Sets the image for the sandbox
    pub fn image(self, image: impl Into<ReferenceOrPath>) -> SandboxBuilder<ReferenceOrPath> {
        SandboxBuilder {
            version: self.version,
            meta: self.meta,
            image: image.into(),
            memory: self.memory,
            cpus: self.cpus,
            volumes: self.volumes,
            ports: self.ports,
            envs: self.envs,
            env_file: self.env_file,
            groups: self.groups,
            depends_on: self.depends_on,
            workdir: self.workdir,
            shell: self.shell,
            scripts: self.scripts,
            exec: self.exec,
            imports: self.imports,
            exports: self.exports,
            scope: self.scope,
        }
    }

    /// Sets the maximum amount of memory allowed for the sandbox
    pub fn memory(mut self, memory: u32) -> SandboxBuilder<I> {
        self.memory = Some(memory);
        self
    }

    /// Sets the maximum number of CPUs allowed for the sandbox
    pub fn cpus(mut self, cpus: u8) -> SandboxBuilder<I> {
        self.cpus = Some(cpus);
        self
    }

    /// Sets the volumes to mount for the sandbox
    pub fn volumes(mut self, volumes: impl IntoIterator<Item = PathPair>) -> SandboxBuilder<I> {
        self.volumes = volumes.into_iter().collect();
        self
    }

    /// Sets the ports to expose for the sandbox
    pub fn ports(mut self, ports: impl IntoIterator<Item = PortPair>) -> SandboxBuilder<I> {
        self.ports = ports.into_iter().collect();
        self
    }

    /// Sets the environment variables for the sandbox
    pub fn envs(mut self, envs: impl IntoIterator<Item = EnvPair>) -> SandboxBuilder<I> {
        self.envs = envs.into_iter().collect();
        self
    }

    /// Sets the environment file for the sandbox
    pub fn env_file(mut self, env_file: impl Into<Utf8UnixPathBuf>) -> SandboxBuilder<I> {
        self.env_file = Some(env_file.into());
        self
    }

    /// Sets the groups for the sandbox
    pub fn groups(
        mut self,
        groups: impl IntoIterator<Item = (String, SandboxGroup)>,
    ) -> SandboxBuilder<I> {
        self.groups = groups.into_iter().collect();
        self
    }

    /// Sets the sandboxes that the sandbox depends on
    pub fn depends_on(mut self, depends_on: impl IntoIterator<Item = String>) -> SandboxBuilder<I> {
        self.depends_on = depends_on.into_iter().collect();
        self
    }

    /// Sets the working directory for the sandbox
    pub fn workdir(mut self, workdir: impl Into<Utf8UnixPathBuf>) -> SandboxBuilder<I> {
        self.workdir = Some(workdir.into());
        self
    }

    /// Sets the shell for the sandbox
    pub fn shell(mut self, shell: impl AsRef<str>) -> SandboxBuilder<I> {
        self.shell = Some(shell.as_ref().to_string());
        self
    }

    /// Sets the scripts for the sandbox
    pub fn scripts(
        mut self,
        scripts: impl IntoIterator<Item = (String, String)>,
    ) -> SandboxBuilder<I> {
        self.scripts = scripts.into_iter().collect();
        self
    }

    /// Sets the exec command for the sandbox
    pub fn exec(mut self, exec: impl AsRef<str>) -> SandboxBuilder<I> {
        self.exec = Some(exec.as_ref().to_string());
        self
    }

    /// Sets the files to import for the sandbox
    pub fn imports(
        mut self,
        imports: impl IntoIterator<Item = (String, Utf8UnixPathBuf)>,
    ) -> SandboxBuilder<I> {
        self.imports = imports.into_iter().collect();
        self
    }

    /// Sets the files to export for the sandbox
    pub fn exports(
        mut self,
        exports: impl IntoIterator<Item = (String, Utf8UnixPathBuf)>,
    ) -> SandboxBuilder<I> {
        self.exports = exports.into_iter().collect();
        self
    }

    /// Sets the network scope for the sandbox
    pub fn scope(mut self, scope: NetworkScope) -> SandboxBuilder<I> {
        self.scope = scope;
        self
    }
}

impl SandboxBuilder<ReferenceOrPath> {
    /// Builds the sandbox
    pub fn build(self) -> Sandbox {
        Sandbox {
            version: self.version,
            meta: self.meta,
            image: self.image,
            memory: self.memory,
            cpus: self.cpus,
            volumes: self.volumes,
            ports: self.ports,
            envs: self.envs,
            groups: self.groups,
            depends_on: self.depends_on,
            workdir: self.workdir,
            shell: self.shell,
            scripts: self.scripts,
            exec: self.exec,
            imports: self.imports,
            exports: self.exports,
            scope: self.scope,
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl Default for SandboxBuilder<()> {
    fn default() -> Self {
        Self {
            version: None,
            meta: None,
            image: (),
            memory: None,
            cpus: None,
            volumes: Vec::new(),
            ports: Vec::new(),
            envs: Vec::new(),
            env_file: None,
            groups: HashMap::new(),
            depends_on: Vec::new(),
            workdir: None,
            shell: Some(DEFAULT_SHELL.to_string()),
            scripts: HashMap::new(),
            exec: None,
            imports: HashMap::new(),
            exports: HashMap::new(),
            scope: NetworkScope::default(),
        }
    }
}
