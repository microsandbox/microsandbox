//! Microsandbox configuration types and helpers.

use std::{
    collections::HashMap,
    fmt::{self, Display},
    net::Ipv4Addr,
    str::FromStr,
};

use getset::{Getters, Setters};
use ipnetwork::Ipv4Network as Ipv4Net;
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

    /// The groups to run the sandboxes in.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub(crate) groups: HashMap<String, Group>,
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

    /// Sandboxes can only communicate within their subnet
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

/// Network configuration for a sandbox in a group.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder, PartialEq, Eq, Getters)]
#[getset(get = "pub with_prefix")]
pub struct SandboxGroupNetwork {
    /// The IP address for the sandbox in this group
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) ip: Option<Ipv4Addr>,

    /// The hostname for this sandbox in the group
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) hostname: Option<String>,
}

/// Network configuration for a group.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder, PartialEq, Eq, Getters)]
#[getset(get = "pub with_prefix")]
pub struct GroupNetwork {
    /// The subnet CIDR for the group. Must be an IPv4 network.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) subnet: Option<Ipv4Net>,
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

    /// The groups to run in.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub(crate) groups: HashMap<String, SandboxGroup>,

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

/// Configuration for a sandbox's group membership.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder, PartialEq, Getters)]
#[getset(get = "pub with_prefix")]
pub struct SandboxGroup {
    /// The volumes to mount.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    #[builder(default)]
    pub(crate) volumes: HashMap<String, String>,

    /// The network configuration for this sandbox in the group.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) network: Option<SandboxGroupNetwork>,
}

/// The group to run the sandboxes in.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder, PartialEq, Eq, Getters)]
#[getset(get = "pub with_prefix")]
pub struct Group {
    /// The version of the group.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) version: Option<Version>,

    /// The metadata about the group.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) meta: Option<Meta>,

    /// The network configuration for the group.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(default, setter(strip_option))]
    pub(crate) network: Option<GroupNetwork>,

    /// The volumes to mount.
    #[serde(
        skip_serializing_if = "HashMap::is_empty",
        default,
        serialize_with = "serialize_path_map",
        deserialize_with = "deserialize_path_map"
    )]
    #[builder(default)]
    pub(crate) volumes: HashMap<String, Utf8UnixPathBuf>,
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

    /// Get a group by name in this configuration
    pub fn get_group(&self, group_name: &str) -> Option<&Group> {
        self.groups.get(group_name)
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
    use std::net::Ipv4Addr;

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
        assert!(config.groups.is_empty());
    }

    #[test]
    fn test_microsandbox_config_default_config() {
        // Test Default trait implementation
        let config = Microsandbox::default();
        assert!(config.meta.is_none());
        assert!(config.modules.is_empty());
        assert!(config.builds.is_empty());
        assert!(config.sandboxes.is_empty());
        assert!(config.groups.is_empty());

        // Test empty sections
        let yaml = r#"
            meta: {}
            modules: {}
            builds: {}
            sandboxes: {}
            groups: {}
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        assert!(config.meta.unwrap() == Meta::default());
        assert!(config.modules.is_empty());
        assert!(config.builds.is_empty());
        assert!(config.sandboxes.is_empty());
        assert!(config.groups.is_empty());
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
        assert_eq!(sandbox.scope, NetworkScope::Group);
    }

    #[test]
    fn test_microsandbox_config_default_scope() {
        // Test default scope for sandbox is Group
        let sandbox = Sandbox::builder()
            .image(ReferenceOrPath::Reference("alpine:latest".parse().unwrap()))
            .shell("/bin/sh")
            .build();
        assert_eq!(sandbox.scope, NetworkScope::Group);

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

        assert_eq!(sandbox.scope, NetworkScope::Group);
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
                groups:
                  build_group:
                    volumes:
                      logs: "/var/log"

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
                groups:
                  backend_group:
                    network:
                      ip: "10.0.1.10"
                      hostname: "api.internal"

            groups:
              backend_group:
                version: "1.0.0"
                meta:
                  description: "Backend services group"
                network:
                  subnet: "10.0.1.0/24"
                volumes:
                  logs: "/var/log"
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

        let api_group = api.groups.get("backend_group").unwrap();
        assert_eq!(
            api_group.network.as_ref().unwrap().ip.unwrap(),
            Ipv4Addr::new(10, 0, 1, 10)
        );
        assert_eq!(
            api_group
                .network
                .as_ref()
                .unwrap()
                .hostname
                .as_ref()
                .unwrap(),
            "api.internal"
        );

        // Test groups
        let groups = &config.groups;
        let backend_group = groups.get("backend_group").unwrap();
        assert_eq!(backend_group.version.as_ref().unwrap().to_string(), "1.0.0");
        assert_eq!(
            backend_group
                .meta
                .as_ref()
                .unwrap()
                .description
                .as_ref()
                .unwrap(),
            "Backend services group"
        );
        assert_eq!(
            backend_group
                .network
                .as_ref()
                .unwrap()
                .subnet
                .unwrap()
                .to_string(),
            "10.0.1.0/24"
        );
        assert_eq!(
            backend_group.volumes.get("logs").unwrap(),
            &Utf8UnixPathBuf::from("/var/log")
        );
    }

    #[test]
    fn test_microsandbox_config_network_configuration() {
        let yaml = r#"
            sandboxes:
              test_sandbox:
                image: "alpine:latest"
                shell: "/bin/sh"
                scope: "group"
                groups:
                  test_group:
                    network:
                      ip: "10.0.1.10"
                      hostname: "test.internal"

            groups:
              test_group:
                network:
                  subnet: "10.0.1.0/24"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();

        // Fix temporary value dropped issue by using direct reference
        let sandboxes = &config.sandboxes;
        let sandbox = sandboxes.get("test_sandbox").unwrap();
        assert_eq!(sandbox.scope, NetworkScope::Group);

        let sandbox_group = sandbox.groups.get("test_group").unwrap();
        assert_eq!(
            sandbox_group.network.as_ref().unwrap().ip.unwrap(),
            Ipv4Addr::new(10, 0, 1, 10)
        );
        assert_eq!(
            sandbox_group
                .network
                .as_ref()
                .unwrap()
                .hostname
                .as_ref()
                .unwrap(),
            "test.internal"
        );

        // Fix temporary value dropped issue for groups
        let groups = &config.groups;
        let group = groups.get("test_group").unwrap();
        assert_eq!(
            group.network.as_ref().unwrap().subnet.unwrap().to_string(),
            "10.0.1.0/24"
        );
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

    #[test]
    fn test_microsandbox_config_group_basic() {
        let yaml = r#"
            groups:
              simple_group:
                version: "1.0.0"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        let groups = &config.groups;

        assert!(groups.contains_key("simple_group"));
        let group = groups.get("simple_group").unwrap();
        assert_eq!(group.version.as_ref().unwrap().to_string(), "1.0.0");
        assert!(group.meta.is_none());
        assert!(group.network.is_none());
        assert!(group.volumes.is_empty());
    }

    #[test]
    fn test_microsandbox_config_group_metadata() {
        let yaml = r#"
            groups:
              metadata_group:
                meta:
                  description: "A group with metadata"
                  authors:
                    - "Test Author <test@example.com>"
                  tags:
                    - "test"
                    - "metadata"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        let groups = &config.groups;

        assert!(groups.contains_key("metadata_group"));
        let group = groups.get("metadata_group").unwrap();
        assert!(group.version.is_none());

        let meta = group.meta.as_ref().unwrap();
        assert_eq!(meta.description.as_ref().unwrap(), "A group with metadata");
        assert_eq!(
            meta.authors.as_ref().unwrap()[0],
            "Test Author <test@example.com>"
        );
        assert_eq!(meta.tags.as_ref().unwrap(), &vec!["test", "metadata"]);
    }

    #[test]
    fn test_microsandbox_config_group_network() {
        let yaml = r#"
            groups:
              network_group:
                network:
                  subnet: "10.0.2.0/24"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        let groups = &config.groups;

        assert!(groups.contains_key("network_group"));
        let group = groups.get("network_group").unwrap();
        assert!(group.version.is_none());
        assert!(group.meta.is_none());

        let network = group.network.as_ref().unwrap();
        assert_eq!(network.subnet.unwrap().to_string(), "10.0.2.0/24");
    }

    #[test]
    fn test_microsandbox_config_group_volumes() {
        let yaml = r#"
            groups:
              volume_group:
                volumes:
                  data: "/data"
                  logs: "/var/log"
                  static: "/var/www/static"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        let groups = &config.groups;

        assert!(groups.contains_key("volume_group"));
        let group = groups.get("volume_group").unwrap();
        assert!(group.version.is_none());
        assert!(group.meta.is_none());
        assert!(group.network.is_none());

        let volumes = &group.volumes;
        assert_eq!(volumes.len(), 3);
        assert_eq!(
            volumes.get("data").unwrap(),
            &Utf8UnixPathBuf::from("/data")
        );
        assert_eq!(
            volumes.get("logs").unwrap(),
            &Utf8UnixPathBuf::from("/var/log")
        );
        assert_eq!(
            volumes.get("static").unwrap(),
            &Utf8UnixPathBuf::from("/var/www/static")
        );
    }

    #[test]
    fn test_microsandbox_config_group_complete() {
        let yaml = r#"
            groups:
              complete_group:
                version: "2.1.0"
                meta:
                  description: "A complete group with all properties"
                  authors:
                    - "Test Author <test@example.com>"
                  tags:
                    - "test"
                    - "complete"
                  readme: "./README.md"
                network:
                  subnet: "10.1.0.0/16"
                volumes:
                  cache: "/var/cache"
                  db: "/var/lib/database"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        let groups = &config.groups;

        assert!(groups.contains_key("complete_group"));
        let group = groups.get("complete_group").unwrap();

        // Check version
        assert_eq!(group.version.as_ref().unwrap().to_string(), "2.1.0");

        // Check metadata
        let meta = group.meta.as_ref().unwrap();
        assert_eq!(
            meta.description.as_ref().unwrap(),
            "A complete group with all properties"
        );
        assert_eq!(
            meta.authors.as_ref().unwrap()[0],
            "Test Author <test@example.com>"
        );
        assert_eq!(meta.tags.as_ref().unwrap(), &vec!["test", "complete"]);
        assert_eq!(
            meta.readme.as_ref().unwrap(),
            &Utf8UnixPathBuf::from("./README.md")
        );

        // Check network
        let network = group.network.as_ref().unwrap();
        assert_eq!(network.subnet.unwrap().to_string(), "10.1.0.0/16");

        // Check volumes
        let volumes = &group.volumes;
        assert_eq!(volumes.len(), 2);
        assert_eq!(
            volumes.get("cache").unwrap(),
            &Utf8UnixPathBuf::from("/var/cache")
        );
        assert_eq!(
            volumes.get("db").unwrap(),
            &Utf8UnixPathBuf::from("/var/lib/database")
        );
    }

    #[test]
    fn test_microsandbox_config_group_sandbox_association() {
        let yaml = r#"
            sandboxes:
              web:
                image: "nginx:alpine"
                shell: "/bin/sh"
                groups:
                  frontend_group:
                    network:
                      ip: "10.2.0.10"
                      hostname: "web.internal"
              api:
                image: "python:3.9-slim"
                shell: "/bin/bash"
                groups:
                  backend_group:
                    network:
                      ip: "10.3.0.20"
                      hostname: "api.internal"
                  frontend_group:
                    network:
                      ip: "10.2.0.20"
                      hostname: "api-frontend.internal"

            groups:
              frontend_group:
                network:
                  subnet: "10.2.0.0/24"
              backend_group:
                network:
                  subnet: "10.3.0.0/24"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();

        // Check that sandboxes are properly associated with groups
        let sandboxes = &config.sandboxes;
        let groups = &config.groups;

        // Check web sandbox in frontend group
        let web = sandboxes.get("web").unwrap();
        assert!(web.groups.contains_key("frontend_group"));
        let web_frontend = web.groups.get("frontend_group").unwrap();
        assert_eq!(
            web_frontend.network.as_ref().unwrap().ip.unwrap(),
            Ipv4Addr::new(10, 2, 0, 10)
        );
        assert_eq!(
            web_frontend
                .network
                .as_ref()
                .unwrap()
                .hostname
                .as_ref()
                .unwrap(),
            "web.internal"
        );

        // Check api sandbox in backend and frontend groups
        let api = sandboxes.get("api").unwrap();
        assert!(api.groups.contains_key("backend_group"));
        assert!(api.groups.contains_key("frontend_group"));

        let api_backend = api.groups.get("backend_group").unwrap();
        assert_eq!(
            api_backend.network.as_ref().unwrap().ip.unwrap(),
            Ipv4Addr::new(10, 3, 0, 20)
        );
        assert_eq!(
            api_backend
                .network
                .as_ref()
                .unwrap()
                .hostname
                .as_ref()
                .unwrap(),
            "api.internal"
        );

        let api_frontend = api.groups.get("frontend_group").unwrap();
        assert_eq!(
            api_frontend.network.as_ref().unwrap().ip.unwrap(),
            Ipv4Addr::new(10, 2, 0, 20)
        );
        assert_eq!(
            api_frontend
                .network
                .as_ref()
                .unwrap()
                .hostname
                .as_ref()
                .unwrap(),
            "api-frontend.internal"
        );

        // Check group subnets
        let frontend_group = groups.get("frontend_group").unwrap();
        assert_eq!(
            frontend_group
                .network
                .as_ref()
                .unwrap()
                .subnet
                .unwrap()
                .to_string(),
            "10.2.0.0/24"
        );

        let backend_group = groups.get("backend_group").unwrap();
        assert_eq!(
            backend_group
                .network
                .as_ref()
                .unwrap()
                .subnet
                .unwrap()
                .to_string(),
            "10.3.0.0/24"
        );
    }

    #[test]
    fn test_microsandbox_config_group_multiple() {
        let yaml = r#"
            groups:
              group_a:
                version: "1.0.0"
                network:
                  subnet: "10.10.0.0/24"
              group_b:
                version: "1.0.0"
                network:
                  subnet: "10.20.0.0/24"
              group_c:
                version: "1.0.0"
                network:
                  subnet: "10.30.0.0/24"
        "#;

        let config: Microsandbox = serde_yaml::from_str(yaml).unwrap();
        let groups = &config.groups;

        assert_eq!(groups.len(), 3);
        assert!(groups.contains_key("group_a"));
        assert!(groups.contains_key("group_b"));
        assert!(groups.contains_key("group_c"));

        // Check subnets of each group
        assert_eq!(
            groups
                .get("group_a")
                .unwrap()
                .network
                .as_ref()
                .unwrap()
                .subnet
                .unwrap()
                .to_string(),
            "10.10.0.0/24"
        );
        assert_eq!(
            groups
                .get("group_b")
                .unwrap()
                .network
                .as_ref()
                .unwrap()
                .subnet
                .unwrap()
                .to_string(),
            "10.20.0.0/24"
        );
        assert_eq!(
            groups
                .get("group_c")
                .unwrap()
                .network
                .as_ref()
                .unwrap()
                .subnet
                .unwrap()
                .to_string(),
            "10.30.0.0/24"
        );
    }
}
