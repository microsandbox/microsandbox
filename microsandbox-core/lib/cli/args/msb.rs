use std::{error::Error, path::PathBuf};

use crate::{cli::styles, oci::Reference};
use clap::Parser;
use typed_path::Utf8UnixPathBuf;

//-------------------------------------------------------------------------------------------------
// Types
//-------------------------------------------------------------------------------------------------

/// msb (microsandbox) is a tool for managing lightweight sandboxes and images
#[derive(Debug, Parser)]
#[command(name = "msb", author, styles=styles::styles())]
pub struct MicrosandboxArgs {
    /// The subcommand to run
    #[command(subcommand)]
    pub subcommand: Option<MicrosandboxSubcommand>,

    /// Show version
    #[arg(short = 'v', long, global = true)]
    pub version: bool,

    /// Show logs with error level
    #[arg(long, global = true)]
    pub error: bool,

    /// Show logs with warn level
    #[arg(long, global = true)]
    pub warn: bool,

    /// Show logs with info level
    #[arg(long, global = true)]
    pub info: bool,

    /// Show logs with debug level
    #[arg(long, global = true)]
    pub debug: bool,

    /// Show logs with trace level
    #[arg(long, global = true)]
    pub trace: bool,
}

/// Available subcommands for managing services
#[derive(Debug, Parser)]
pub enum MicrosandboxSubcommand {
    /// Initialize a new microsandbox project
    #[command(name = "init")]
    Init {
        /// Specifies the directory to initialize the project in
        #[arg(required = false, name = "PATH")]
        path: Option<PathBuf>,

        /// Specifies the directory to initialize the project in
        #[arg(short, long = "path", name = "PATH\0")]
        path_with_flag: Option<PathBuf>,
    },

    /// Add a new sandbox to the project
    #[command(name = "add")]
    Add {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Whether command should apply for a build sandbox
        #[arg(short, long)]
        build: bool,

        /// Whether command should apply for a group
        #[arg(short, long)]
        group: bool,

        /// Names of components to add
        #[arg(required = true)]
        names: Vec<String>,

        /// Image to use
        #[arg(short, long)]
        image: String,

        /// Memory in MiB
        #[arg(long)]
        memory: Option<u32>,

        /// Number of CPUs
        #[arg(long, alias = "cpu")]
        cpus: Option<u32>,

        /// Volume mappings, format: <host_path>:<container_path>
        #[arg(long = "volume", name = "VOLUME")]
        volumes: Vec<String>,

        /// Port mappings, format: <host_port>:<container_port>
        #[arg(long = "port", name = "PORT")]
        ports: Vec<String>,

        /// Environment variables, format: <key>=<value>
        #[arg(long = "env", name = "ENV")]
        envs: Vec<String>,

        /// Environment file
        #[arg(long)]
        env_file: Option<Utf8UnixPathBuf>,

        /// Dependencies
        #[arg(long)]
        depends_on: Vec<String>,

        /// Working directory
        #[arg(long)]
        workdir: Option<Utf8UnixPathBuf>,

        /// Shell to use
        #[arg(long)]
        shell: Option<String>,

        /// Scripts to add
        #[arg(long = "script", name = "SCRIPT", value_parser = parse_key_val::<String, String>)]
        scripts: Vec<(String, String)>,

        /// Files to import, format: <name>=<path>
        #[arg(long = "import", name = "IMPORT", value_parser = parse_key_val::<String, String>)]
        imports: Vec<(String, String)>,

        /// Files to export, format: <name>=<path>
        #[arg(long = "export", name = "EXPORT", value_parser = parse_key_val::<String, String>)]
        exports: Vec<(String, String)>,

        /// Network scope, options: local, public, any, none
        #[arg(long)]
        scope: Option<String>,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Remove a sandbox from the project
    #[command(name = "remove")]
    Remove {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Whether command should apply for a build sandbox
        #[arg(short, long)]
        build: bool,

        /// Whether command should apply for a group
        #[arg(short, long)]
        group: bool,

        /// Names of components to remove
        #[arg(required = true)]
        names: Vec<String>,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,
    },

    /// List sandboxs in the project
    #[command(name = "list")]
    List {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Whether command should apply for a build sandbox
        #[arg(short, long)]
        build: bool,

        /// Whether command should apply for a group
        #[arg(short, long)]
        group: bool,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Show logs of a running build, sandbox, or group
    #[command(name = "log")]
    Log {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Whether command should apply for a build sandbox
        #[arg(short, long)]
        build: bool,

        /// Whether command should apply for a group
        #[arg(short, long)]
        group: bool,

        /// Name of the component
        #[arg(required = true)]
        name: String,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,

        /// Follow the logs
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show from the end
        #[arg(short = 'n', long)]
        tail: Option<usize>,
    },

    /// Show tree of layers that make up a sandbox
    #[command(name = "tree")]
    Tree {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Whether command should apply for a build sandbox
        #[arg(short, long)]
        build: bool,

        /// Whether command should apply for a group
        #[arg(short, long)]
        group: bool,

        /// Names of components to show
        #[arg(required = true)]
        names: Vec<String>,

        /// Maximum depth level
        #[arg(short = 'L')]
        level: Option<usize>,
    },

    /// Run a sandbox script
    #[command(name = "run", alias = "r")]
    Run {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Whether command should apply for a build sandbox
        #[arg(short, long)]
        build: bool,

        /// Name of the component
        #[arg(required = true, name = "NAME[~SCRIPT]")]
        name: String,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,

        /// Run sandbox in the background
        #[arg(short, long)]
        detach: bool,

        /// Execute a command within the sandbox
        #[arg(short, long)]
        exec: Option<String>,

        /// Additional arguments after `--`
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Open a shell in a sandbox
    #[command(name = "shell")]
    Shell {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Whether command should apply for a build sandbox
        #[arg(short, long)]
        build: bool,

        /// Name of the component
        #[arg(required = true)]
        name: String,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,

        /// Run sandbox in the background
        #[arg(short, long)]
        detach: bool,

        /// Additional arguments after `--`
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Create a temporary sandbox
    #[command(name = "tmp", alias = "t")]
    Tmp {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        image: bool,

        /// Name of the image
        #[arg(required = true, name = "NAME[~SCRIPT]")]
        name: String,

        /// Number of CPUs
        #[arg(long, alias = "cpu")]
        cpus: Option<u8>,

        /// Memory in MB
        #[arg(long)]
        memory: Option<u32>,

        /// Volume mappings, format: <host_path>:<container_path>
        #[arg(long = "volume", name = "VOLUME")]
        volumes: Vec<String>,

        /// Port mappings, format: <host_port>:<container_port>
        #[arg(long = "port", name = "PORT")]
        ports: Vec<String>,

        /// Environment variables, format: <key>=<value>
        #[arg(long = "env", name = "ENV")]
        envs: Vec<String>,

        /// Working directory
        #[arg(long)]
        workdir: Option<Utf8UnixPathBuf>,

        /// Network scope, options: local, public, any, none
        #[arg(long)]
        scope: Option<String>,

        /// Execute a command within the sandbox
        #[arg(short, long)]
        exec: Option<String>,

        /// Additional arguments after `--`
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Install a script from an image
    #[command(name = "install", alias = "i")]
    Install {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        image: bool,

        /// Name of the image
        #[arg(required = true, name = "NAME[~SCRIPT]")]
        name: String,

        /// Alias for the script
        #[arg()]
        alias: Option<String>,

        /// Number of CPUs
        #[arg(long, alias = "cpu")]
        cpus: Option<u8>,

        /// Memory in MB
        #[arg(long)]
        memory: Option<u32>,

        /// Volume mappings, format: <host_path>:<container_path>
        #[arg(long = "volume", name = "VOLUME")]
        volumes: Vec<String>,

        /// Port mappings, format: <host_port>:<container_port>
        #[arg(long = "port", name = "PORT")]
        ports: Vec<String>,

        /// Environment variables, format: <key>=<value>
        #[arg(long = "env", name = "ENV")]
        envs: Vec<String>,

        /// Working directory
        #[arg(long)]
        workdir: Option<Utf8UnixPathBuf>,

        /// Network scope, options: local, public, any, none
        #[arg(long)]
        scope: Option<String>,

        /// Execute a command within the sandbox
        #[arg(short, long)]
        exec: Option<String>,

        /// Additional arguments after `--`
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Uninstall a script
    #[command(name = "uninstall")]
    Uninstall {
        /// Script to uninstall
        script: Option<String>,
    },

    /// Start or stop project sandboxes based on configuration
    #[command(name = "apply")]
    Apply {
        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Start project sandboxes
    #[command(name = "up")]
    Up {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Whether command should apply for a build sandbox
        #[arg(short, long)]
        build: bool,

        /// Whether command should apply for a group
        #[arg(short, long)]
        group: bool,

        /// Names of components to start
        #[arg(required = true)]
        names: Vec<String>,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Stop project sandboxes
    #[command(name = "down")]
    Down {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Whether command should apply for a build sandbox
        #[arg(short, long)]
        build: bool,

        /// Whether command should apply for a group
        #[arg(short, long)]
        group: bool,

        /// Names of components to stop
        #[arg(required = true)]
        names: Vec<String>,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Show running status
    #[command(name = "status")]
    Status {
        /// Whether command should apply for a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Whether command should apply for a build sandbox
        #[arg(short, long)]
        build: bool,

        /// Whether command should apply for a group
        #[arg(short, long)]
        group: bool,

        /// Name of the component
        #[arg(required = true)]
        name: String,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Clean cached sandbox layers, metadata, etc.
    #[command(name = "clean")]
    Clean {
        /// Clean globally. This cleans $MICROSANDBOX_HOME
        #[arg(long)]
        global: bool,

        /// Clean all
        #[arg(long)]
        all: bool,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Build images
    #[command(name = "build")]
    Build {
        /// Build from build definition
        #[arg(short, long)]
        build: bool,

        /// Build from sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Build from group
        #[arg(short, long)]
        group: bool,

        /// Names of components to build
        #[arg(required = true)]
        names: Vec<String>,

        /// Create a snapshot
        #[arg(long)]
        snapshot: bool,
    },

    /// Pull an image
    #[command(name = "pull")]
    Pull {
        /// Whether command should apply for an image
        #[arg(short, long)]
        image: bool,

        /// Whether command should apply for an image group
        #[arg(short = 'G', long)]
        image_group: bool,

        /// Name of the image or image group
        #[arg(required = true)]
        name: Reference,

        /// Path to store the layer files
        #[arg(short = 'L', long)]
        layer_path: Option<PathBuf>,
    },

    /// Push an image
    #[command(name = "push")]
    Push {
        /// Whether command should apply for an image
        #[arg(short, long)]
        image: bool,

        /// Whether command should apply for an image group
        #[arg(short = 'G', long)]
        image_group: bool,

        /// Name of the image or image group
        #[arg(required = true)]
        name: String,
    },

    /// Manage microsandbox itself
    #[command(name = "self")]
    Self_ {
        /// Action to perform
        #[arg(value_enum)]
        action: SelfAction,
    },

    /// Start a server for orchestrating sandboxes
    #[command(name = "server")]
    Server {
        /// The subcommand to run
        #[command(subcommand)]
        subcommand: ServerSubcommand,
    },

    /// Version of microsandbox
    #[command(name = "version")]
    Version,
}

/// Subcommands for the server subcommand
#[derive(Debug, Parser)]
pub enum ServerSubcommand {
    /// Start the sandbox server
    Start {
        /// Port to listen on
        #[arg(long)]
        port: Option<u16>,

        /// Path to the namespace directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Disable default namespace
        #[arg(long, default_value_t = false)]
        disable_default: bool,

        /// Make server require a secure API key
        #[arg(long, default_value_t = false)]
        secure: bool,

        /// Set secret key for server. Automatically generated if not provided.
        #[arg(long)]
        key: Option<String>,

        /// Run server in the background``
        #[arg(short, long)]
        detach: bool,
    },

    /// Stop the sandbox server
    Stop,

    /// Generate a new API key
    #[command(name = "keygen")]
    Keygen {
        /// Token expiration duration. format: 1s, 2m, 3h, 4d, 5w, 6mo, 7y
        #[arg(long)]
        expire: Option<String>,
    },
}

/// Actions for the self subcommand
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SelfAction {
    /// Upgrade microsandbox
    Upgrade,

    /// Uninstall microsandbox
    Uninstall,
}

//-------------------------------------------------------------------------------------------------
// Functions: Helpers
//-------------------------------------------------------------------------------------------------

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;

    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
