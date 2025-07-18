//! Microsandbox configuration types and helpers.

use std::{
    collections::HashMap,
    fmt::{self, Display},
    str::FromStr,
};

use getset::{Getters, Setters};
use semver::Version;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use typed_path::Utf8UnixPathBuf;

use crate::{
    config::{EnvPair, PathPair, PortPair, ReferenceOrPath},
    MicrosandboxError, MicrosandboxResult,
};

use super::{MicrosandboxBuilder, SandboxBuilder};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// The start script name.
pub const START_SCRIPT_NAME: &str = "start";

/// The default network scope for a sandbox.
pub const DEFAULT_NETWORK_SCOPE: NetworkScope = NetworkScope::Public;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// The microsandbox configuration.
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Getters)]
#[getset(get = "pub with_prefix")]
pub struct Microsandbox {
    /// The metadata about the configuration.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) meta: Option<Meta>,

    /// The modules to import.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub(crate) modules: HashMap<String, Module>,

    /// The builds to run.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub(crate) builds: HashMap<String, Build>,

    /// The sandboxes to run.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub(crate) sandboxes: HashMap<String, Sandbox>,
}

/// The metadata about the configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize, TypedBuilder, PartialEq, Eq, Getters)]
#[getset(get = "pub with_prefix")]
pub struct Meta {
    /// The authors of the configuration.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) authors: Option<Vec<String>>,

    /// The description of the sandbox.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) description: Option<String>,

    /// The homepage of the configuration.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) homepage: Option<String>,

    /// The repository of the configuration.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) repository: Option<String>,

    /// The path to the readme file.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        serialize_with = "serialize_optional_path",
        deserialize_with = "deserialize_optional_path"
    )]
    #[builder(default, setter(strip_option))]
    pub(crate) readme: Option<Utf8UnixPathBuf>,

    /// The tags for the configuration.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) tags: Option<Vec<String>>,

    /// The icon for the configuration.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        serialize_with = "serialize_optional_path",
        deserialize_with = "deserialize_optional_path"
    )]
    #[builder(default, setter(strip_option))]
    pub(crate) icon: Option<Utf8UnixPathBuf>,
}

/// Component mapping for imports.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder, PartialEq, Getters)]
#[getset(get = "pub with_prefix")]
pub struct ComponentMapping {
    /// The alias for the component.
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "as")]
    #[builder(default, setter(strip_option))]
    pub(crate) as_: Option<String>,
}

/// Module import configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Module(pub HashMap<String, Option<ComponentMapping>>);

/// A build to run.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder, PartialEq, Getters)]
#[getset(get = "pub with_prefix")]
pub struct Build {
    /// The image to use. This can be a path to a local rootfs or an OCI image reference.
    pub(crate) image: ReferenceOrPath,

    /// The amount of memory in MiB to use.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) memory: Option<u32>,

    /// The number of vCPUs to use.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) cpus: Option<u8>,

    /// The volumes to mount.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    pub(crate) volumes: Vec<PathPair>,

    /// The ports to expose.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    pub(crate) ports: Vec<PortPair>,

    /// The environment variables to use.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    pub(crate) envs: Vec<EnvPair>,

    /// The builds to depend on.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    pub(crate) depends_on: Vec<String>,

    /// The working directory to use.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        serialize_with = "serialize_optional_path",
        deserialize_with = "deserialize_optional_path"
    )]
    #[builder(default, setter(strip_option))]
    pub(crate) workdir: Option<Utf8UnixPathBuf>,

    /// The shell to use.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) shell: Option<String>,

    /// The steps that will be run.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    #[builder(default)]
    pub(crate) steps: HashMap<String, String>,

    /// The command to run. This is a list of command and arguments.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    pub(crate) command: Vec<String>,

    /// The files to import.
    #[serde(
        skip_serializing_if = "HashMap::is_empty",
        default,
        serialize_with = "serialize_path_map",
        deserialize_with = "deserialize_path_map"
    )]
    #[builder(default)]
    pub(crate) imports: HashMap<String, Utf8UnixPathBuf>,

    /// The artifacts produced by the build.
    #[serde(
        skip_serializing_if = "HashMap::is_empty",
        default,
        serialize_with = "serialize_path_map",
        deserialize_with = "deserialize_path_map"
    )]
    #[builder(default)]
    pub(crate) exports: HashMap<String, Utf8UnixPathBuf>,
}

/// Network scope configuration for a sandbox.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NetworkScope {
    /// Sandboxes cannot communicate with any other sandboxes
    #[serde(rename = "none")]
    None = 0,

    /// Sandboxes can only communicate within their subnet (Not implemented)
    #[serde(rename = "group")]
    Group = 1,

    /// Sandboxes can communicate with any other non-private address
    #[serde(rename = "public")]
    #[default]
    Public = 2,

    /// Sandboxes can communicate with any address
    #[serde(rename = "any")]
    Any = 3,
}

/// The sandbox to run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Getters, Setters)]
#[getset(get = "pub with_prefix", set = "pub with_prefix")]
pub struct Sandbox {
    /// The version of the sandbox.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) version: Option<Version>,

    /// The metadata about the sandbox.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) meta: Option<Meta>,

    /// The image to use. This can be a path to a local rootfs or an OCI image reference.
    pub(crate) image: ReferenceOrPath,

    /// The amount of memory in MiB to use.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) memory: Option<u32>,

    /// The number of vCPUs to use.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) cpus: Option<u8>,

    /// The volumes to mount.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) volumes: Vec<PathPair>,

    /// The ports to expose.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) ports: Vec<PortPair>,

    /// The environment variables to use.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) envs: Vec<EnvPair>,

    /// The sandboxes to depend on.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) depends_on: Vec<String>,

    /// The working directory to use.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        serialize_with = "serialize_optional_path",
        deserialize_with = "deserialize_optional_path"
    )]
    pub(crate) workdir: Option<Utf8UnixPathBuf>,

    /// The shell to use.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) shell: Option<String>,

    /// The scripts that can be run.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub(crate) scripts: HashMap<String, String>,

    /// The command to run. This is a list of command and arguments.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) command: Vec<String>,

    /// The files to import.
    #[serde(
        skip_serializing_if = "HashMap::is_empty",
        default,
        serialize_with = "serialize_path_map",
        deserialize_with = "deserialize_path_map"
    )]
    pub(crate) imports: HashMap<String, Utf8UnixPathBuf>,

    /// The artifacts produced by the sandbox.
    #[serde(
        skip_serializing_if = "HashMap::is_empty",
        default,
        serialize_with = "serialize_path_map",
        deserialize_with = "deserialize_path_map"
    )]
    pub(crate) exports: HashMap<String, Utf8UnixPathBuf>,

    /// The network scope for the sandbox.
    #[serde(default)]
    pub(crate) scope: NetworkScope,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl Microsandbox {
    /// The maximum sandbox dependency chain length.
    pub const MAX_DEPENDENCY_DEPTH: usize = 32;

    /// Get a sandbox by name in this configuration
    pub fn get_sandbox(&self, sandbox_name: &str) -> Option<&Sandbox> {
        self.sandboxes.get(sandbox_name)
    }

    /// Get a build by name in this configuration
    pub fn get_build(&self, build_name: &str) -> Option<&Build> {
        self.builds.get(build_name)
    }

    /// Validates the configuration.
    pub fn validate(&self) -> MicrosandboxResult<()> {
        // Validate all sandboxes
        for sandbox in self.sandboxes.values() {
            sandbox.validate()?;
        }

        Ok(())
    }

    /// Returns a builder for the Microsandbox configuration.
    ///
    /// See [`MicrosandboxBuilder`] for options.
    pub fn builder() -> MicrosandboxBuilder {
        MicrosandboxBuilder::default()
    }
}

impl Sandbox {
    /// Returns a builder for the sandbox.
    ///
    /// See [`SandboxBuilder`] for options.
    pub fn builder() -> SandboxBuilder<()> {
        SandboxBuilder::default()
    }

    /// Validates the configuration.
    pub fn validate(&self) -> MicrosandboxResult<()> {
        // Error if start and exec are both not defined
        if self.scripts.get(START_SCRIPT_NAME).is_none()
            && self.command.is_empty()
            && self.shell.is_none()
        {
            return Err(MicrosandboxError::MissingStartOrExecOrShell);
        }

        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl TryFrom<&str> for NetworkScope {
    type Error = MicrosandboxError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "none" => Ok(NetworkScope::None),
            "group" => Ok(NetworkScope::Group),
            "public" => Ok(NetworkScope::Public),
            "any" => Ok(NetworkScope::Any),
            _ => Err(MicrosandboxError::InvalidNetworkScope(s.to_string())),
        }
    }
}

impl Display for NetworkScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkScope::None => write!(f, "none"),
            NetworkScope::Group => write!(f, "group"),
            NetworkScope::Public => write!(f, "public"),
            NetworkScope::Any => write!(f, "any"),
        }
    }
}

impl FromStr for NetworkScope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NetworkScope::try_from(s)?)
    }
}

impl TryFrom<String> for NetworkScope {
    type Error = MicrosandboxError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(NetworkScope::try_from(s.as_str())?)
    }
}

impl TryFrom<u8> for NetworkScope {
    type Error = MicrosandboxError;

    fn try_from(u: u8) -> Result<Self, Self::Error> {
        match u {
            0 => Ok(NetworkScope::None),
            1 => Ok(NetworkScope::Group),
            2 => Ok(NetworkScope::Public),
            3 => Ok(NetworkScope::Any),
            _ => Err(MicrosandboxError::InvalidNetworkScope(u.to_string())),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Functions: Serialization helpers
//--------------------------------------------------------------------------------------------------

fn serialize_optional_path<S>(
    path: &Option<Utf8UnixPathBuf>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match path {
        Some(p) => serializer.serialize_str(p.as_str()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_optional_path<'de, D>(deserializer: D) -> Result<Option<Utf8UnixPathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|s| Ok(Utf8UnixPathBuf::from(s)))
        .transpose()
}

fn serialize_path_map<S>(
    map: &HashMap<String, Utf8UnixPathBuf>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map_ser = serializer.serialize_map(Some(map.len()))?;
    for (k, v) in map {
        map_ser.serialize_entry(k, v.as_str())?;
    }
    map_ser.end()
}

fn deserialize_path_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Utf8UnixPathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    HashMap::<String, String>::deserialize(deserializer).map(|string_map| {
        string_map
            .into_iter()
            .map(|(k, v)| (k, Utf8UnixPathBuf::from(v)))
            .collect()
    })
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microsandbox_config_empty_config() {
        let yaml = r#"
            # Empty config with no fields
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        assert!(config.meta.is_none());
        assert!(config.modules.is_empty());
        assert!(config.builds.is_empty());
        assert!(config.sandboxes.is_empty());
    }

    #[test]
    fn test_microsandbox_config_default_config() {
        // Test Default trait implementation
        let config = Microsandbox::default();
        assert!(config.meta.is_none());
        assert!(config.modules.is_empty());
        assert!(config.builds.is_empty());
        assert!(config.sandboxes.is_empty());

        // Test empty sections
        let yaml = r#"
            meta: {}
            modules: {}
            builds: {}
            sandboxes: {}
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        assert!(config.meta.unwrap() == Meta::default());
        assert!(config.modules.is_empty());
        assert!(config.builds.is_empty());
        assert!(config.sandboxes.is_empty());
    }

    #[test]
    fn test_microsandbox_config_minimal_sandbox_config() {
        let yaml = r#"
            sandboxes:
              test:
                image: "alpine:latest"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        let sandboxes = &config.sandboxes;
        let sandbox = sandboxes.get("test").unwrap();

        assert!(sandbox.version.is_none());
        assert!(sandbox.memory.is_none());
        assert!(sandbox.cpus.is_none());
        assert!(sandbox.volumes.is_empty());
        assert!(sandbox.ports.is_empty());
        assert!(sandbox.envs.is_empty());
        assert!(sandbox.workdir.is_none());
        assert!(sandbox.shell.is_none());
        assert!(sandbox.scripts.is_empty());
        assert_eq!(sandbox.scope, NetworkScope::Public);
    }

    #[test]
    fn test_microsandbox_config_default_scope() {
        // Test default scope for sandbox is Public
        let sandbox = Sandbox::builder()
            .image(ReferenceOrPath::Reference("alpine:latest".parse().unwrap()))
            .shell("/bin/sh")
            .build();
        assert_eq!(sandbox.scope, NetworkScope::Public);

        // Test default scope in YAML
        let yaml = r#"
            sandboxes:
              test:
                image: "alpine:latest"
                shell: "/bin/sh"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        let sandboxes = &config.sandboxes;
        let sandbox = sandboxes.get("test").unwrap();

        assert_eq!(sandbox.scope, NetworkScope::Public);
    }

    #[test]
    fn test_microsandbox_config_basic_microsandbox_config() {
        let yaml = r#"
            meta:
              authors:
                - "John Doe <john@example.com>"
              description: "Test configuration"
              homepage: "https://example.com"
              repository: "https://github.com/example/test"
              readme: "./README.md"
              tags:
                - "test"
                - "example"
              icon: "./icon.png"

            sandboxes:
              test_sandbox:
                version: "1.0.0"
                image: "alpine:latest"
                memory: 1024
                cpus: 2
                volumes:
                  - "./src:/app/src"
                ports:
                  - "8080:80"
                envs:
                  - "DEBUG=true"
                workdir: "/app"
                shell: "/bin/sh"
                scripts:
                  start: "echo 'Hello, World!'"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();

        // Verify meta section
        let meta = config.meta.as_ref().unwrap();
        assert_eq!(
            meta.authors.as_ref().unwrap()[0],
            "John Doe <john@example.com>"
        );
        assert_eq!(meta.description.as_ref().unwrap(), "Test configuration");
        assert_eq!(meta.homepage.as_ref().unwrap(), "https://example.com");
        assert_eq!(
            meta.repository.as_ref().unwrap(),
            "https://github.com/example/test"
        );
        assert_eq!(
            meta.readme.as_ref().unwrap(),
            &Utf8UnixPathBuf::from("./README.md")
        );
        assert_eq!(meta.tags.as_ref().unwrap(), &vec!["test", "example"]);
        assert_eq!(
            meta.icon.as_ref().unwrap(),
            &Utf8UnixPathBuf::from("./icon.png")
        );

        // Verify sandbox section
        let sandboxes = &config.sandboxes;
        let sandbox = sandboxes.get("test_sandbox").unwrap();
        assert_eq!(sandbox.version.as_ref().unwrap().to_string(), "1.0.0");
        assert_eq!(sandbox.memory.unwrap(), 1024);
        assert_eq!(sandbox.cpus.unwrap(), 2);
        assert_eq!(sandbox.volumes[0].to_string(), "./src:/app/src");
        assert_eq!(sandbox.ports[0].to_string(), "8080:80");
        assert_eq!(sandbox.envs[0].to_string(), "DEBUG=true");
        assert_eq!(
            sandbox.workdir.as_ref().unwrap(),
            &Utf8UnixPathBuf::from("/app")
        );
        assert_eq!(sandbox.shell, Some("/bin/sh".to_string()));
        assert_eq!(
            sandbox.scripts.get("start").unwrap(),
            "echo 'Hello, World!'"
        );
    }

    #[test]
    fn test_microsandbox_config_full_microsandbox_config() {
        let yaml = r#"
            meta:
              description: "Full test configuration"

            modules:
              "./database.yaml":
                database: {}
              "./redis.yaml":
                redis:
                  as: "cache"

            builds:
              base_build:
                image: "python:3.11-slim"
                memory: 2048
                cpus: 2
                volumes:
                  - "./requirements.txt:/build/requirements.txt"
                envs:
                  - "PYTHON_VERSION=3.11"
                workdir: "/build"
                shell: "/bin/bash"
                steps:
                  build: "pip install -r requirements.txt"
                imports:
                  requirements: "./requirements.txt"
                exports:
                  packages: "/build/dist/packages"

            sandboxes:
              api:
                version: "1.0.0"
                image: "python:3.11-slim"
                memory: 1024
                cpus: 1
                volumes:
                  - "./api:/app/src"
                ports:
                  - "8000:8000"
                envs:
                  - "DEBUG=false"
                depends_on:
                  - "database"
                  - "cache"
                workdir: "/app"
                shell: "/bin/bash"
                scripts:
                  start: "python -m uvicorn src.main:app"
                scope: "public"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();

        // Test modules
        let modules = &config.modules;
        assert!(modules.contains_key("./database.yaml"));
        assert!(modules.contains_key("./redis.yaml"));

        // Fix for the ComponentMapping.as_() error
        let redis_module = &modules.get("./redis.yaml").unwrap().0;
        let redis_comp = redis_module.get("redis").unwrap().as_ref().unwrap();
        // Access as_ field directly as a field, not a method
        assert_eq!(redis_comp.as_.as_ref().unwrap(), "cache");

        // Test builds
        let builds = &config.builds;
        let base_build = builds.get("base_build").unwrap();
        assert_eq!(base_build.memory.unwrap(), 2048);
        assert_eq!(base_build.cpus.unwrap(), 2);
        assert_eq!(
            base_build.workdir.as_ref().unwrap(),
            &Utf8UnixPathBuf::from("/build")
        );
        assert_eq!(base_build.shell, Some("/bin/bash".to_string()));
        assert_eq!(
            base_build.steps.get("build").unwrap(),
            "pip install -r requirements.txt"
        );
        assert_eq!(
            base_build.imports.get("requirements").unwrap(),
            &Utf8UnixPathBuf::from("./requirements.txt")
        );
        assert_eq!(
            base_build.exports.get("packages").unwrap(),
            &Utf8UnixPathBuf::from("/build/dist/packages")
        );

        // Test sandboxes
        let sandboxes = &config.sandboxes;
        let api = sandboxes.get("api").unwrap();
        assert_eq!(api.version.as_ref().unwrap().to_string(), "1.0.0");
        assert_eq!(api.memory.unwrap(), 1024);
        assert_eq!(api.cpus.unwrap(), 1);
        assert_eq!(api.depends_on, vec!["database", "cache"]);
        assert_eq!(api.scope, NetworkScope::Public);
    }

    #[test]
    fn test_microsandbox_config_build_dependencies() {
        let yaml = r#"
            builds:
              base:
                image: "python:3.11-slim"
                depends_on: ["deps"]
              deps:
                image: "python:3.11-slim"
                scripts:
                  install: "pip install -r requirements.txt"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        let builds = &config.builds;

        let base = builds.get("base").unwrap();
        assert_eq!(base.depends_on, vec!["deps"]);

        let deps = builds.get("deps").unwrap();
        assert_eq!(
            deps.steps.get("install").unwrap(),
            "pip install -r requirements.txt"
        );
    }

    #[test]
    fn test_microsandbox_config_invalid_configurations() {
        // Test invalid scope
        let yaml = r#"
            sandboxes:
              test:
                image: "alpine:latest"
                shell: "/bin/sh"
                scope: "invalid"
        "#;
        assert!(serde_yaml::from_str::<Microsandbox>(yaml).is_err());

        // Test invalid IP address
        let yaml = r#"
            sandboxes:
              test:
                image: "alpine:latest"
                shell: "/bin/sh"
                groups:
                  test_group:
                    network:
                      ip: "invalid"
        "#;
        assert!(serde_yaml::from_str::<Microsandbox>(yaml).is_err());

        // Test invalid subnet
        let yaml = r#"
            groups:
              test:
                network:
                  subnet: "invalid"
        "#;
        assert!(serde_yaml::from_str::<Microsandbox>(yaml).is_err());

        // Test invalid version
        let yaml = r#"
            sandboxes:
              test:
                image: "alpine:latest"
                shell: "/bin/sh"
                version: "invalid"
        "#;
        assert!(serde_yaml::from_str::<Microsandbox>(yaml).is_err());
    }
}
