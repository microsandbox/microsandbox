use std::path::PathBuf;

use crate::{cli::styles, oci::Reference};
use clap::Parser;
use typed_path::Utf8UnixPathBuf;

//-------------------------------------------------------------------------------------------------
// Types
//-------------------------------------------------------------------------------------------------

/// `monocore` is a tool for managing lightweight virtual machines and images
#[derive(Debug, Parser)]
#[command(name = "monocore", author, styles=styles::styles())]
pub struct MonocoreArgs {
    /// The subcommand to run
    #[command(subcommand)]
    pub subcommand: Option<MonocoreSubcommand>,

    /// Enable verbose logging
    #[arg(short = 'V', long)]
    pub verbose: bool,

    /// Show version
    #[arg(short = 'v', long)]
    pub version: bool,
}

/// Available subcommands for managing services
#[derive(Debug, Parser)]
pub enum MonocoreSubcommand {
    /// Initialize a new monocore project
    #[command(name = "init")]
    Init {
        /// Specifies the directory to initialize the project in
        ///
        /// This argument is mutually exclusive with the `--path` flag
        #[arg(required = false, conflicts_with = "path_with_flag", name = "PATH")]
        path: Option<PathBuf>,

        /// Specifies the directory to initialize the project in
        ///
        /// This flag is mutually exclusive with the positional argument
        #[arg(short, long = "path", conflicts_with = "path", name = "PATH")]
        path_with_flag: Option<PathBuf>,
    },

    /// Add a new build, sandbox, or group component to the project
    #[command(name = "add")]
    Add {
        /// Add a sandbox
        #[arg(short, long, default_value_t = true)]
        sandbox: bool,

        /// Add a build
        #[arg(short, long)]
        build: bool,

        /// Add a group
        #[arg(short, long)]
        group: bool,

        /// Names of components to add
        #[arg(required = true)]
        names: Vec<String>,

        /// Image to use
        #[arg(short, long)]
        image: Option<String>,

        /// Number of CPUs
        #[arg(long)]
        cpus: Option<u32>,

        /// RAM in MB
        #[arg(long)]
        ram: Option<u32>,

        /// Volume mappings
        #[arg(long)]
        volumes: Vec<String>,

        /// Port mappings
        #[arg(long)]
        ports: Vec<String>,

        /// Environment variables
        #[arg(long)]
        envs: Vec<String>,

        /// Groups to join
        #[arg(long)]
        groups: Vec<String>,

        /// Working directory
        #[arg(long)]
        workdir: Option<Utf8UnixPathBuf>,

        /// Shell to use
        #[arg(long)]
        shell: Option<String>,

        /// Scripts to add
        #[arg(long)]
        scripts: Vec<String>,

        /// Files to import
        #[arg(long)]
        imports: Vec<String>,

        /// Files to export
        #[arg(long)]
        exports: Vec<String>,

        /// Network configuration
        #[arg(long)]
        network: Option<String>,
    },

    /// Remove a build, sandbox, or group component from the project
    #[command(name = "remove")]
    Remove {
        /// Remove a sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Remove a build
        #[arg(short, long)]
        build: bool,

        /// Remove a group
        #[arg(short, long)]
        group: bool,

        /// Names of components to remove
        #[arg(required = true)]
        names: Vec<String>,
    },

    /// List build, sandbox, or group components in the project
    #[command(name = "list")]
    List {
        /// List sandboxes
        #[arg(short, long)]
        sandbox: bool,

        /// List builds
        #[arg(short, long)]
        build: bool,

        /// List groups
        #[arg(short, long)]
        group: bool,
    },

    /// Show logs of a running build, sandbox, or group
    #[command(name = "log")]
    Log {
        /// Show sandbox logs
        #[arg(short, long)]
        sandbox: bool,

        /// Name of the component
        #[arg(required = true)]
        name: String,

        /// Follow the logs
        #[arg(long)]
        follow: bool,

        /// Don't use a pager
        #[arg(long)]
        no_pager: bool,

        /// Number of lines to show from the end
        #[arg(long)]
        tail: Option<usize>,

        /// Number of lines to show
        #[arg(long)]
        count: Option<usize>,

        /// Log level
        #[arg(short = 'L')]
        level: Option<String>,
    },

    /// Show tree of layers that make up a build, sandbox, or group component
    #[command(name = "tree")]
    Tree {
        /// Show sandbox tree
        #[arg(short, long)]
        sandbox: bool,

        /// Show build tree
        #[arg(short, long)]
        build: bool,

        /// Show group tree
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
    #[command(name = "run")]
    Run {
        /// Specifies the sandbox and script to run
        #[arg(conflicts_with = "sandbox_script_with_flag", name = "SANDBOX~SCRIPT")]
        sandbox_script: Option<String>,

        /// Specifies the sandbox and script to run
        #[arg(
            short,
            long = "sandbox",
            conflicts_with = "sandbox_script",
            name = "SANDBOX~SCRIPT"
        )]
        sandbox_script_with_flag: Option<String>,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,

        /// Additional arguments after `--`
        #[arg(last = true)]
        args: Vec<String>,

        /// Run sandbox in the background
        #[arg(long)]
        detach: bool,
    },

    /// Start a sandbox
    #[command(name = "start")]
    Start {
        /// Specifies the sandbox
        #[arg(conflicts_with = "sandbox_with_flag", name = "SANDBOX")]
        sandbox: Option<String>,

        /// Specifies the sandbox
        #[arg(short, long = "sandbox", conflicts_with = "sandbox", name = "SANDBOX")]
        sandbox_with_flag: Option<String>,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,

        /// Additional arguments
        #[arg(last = true)]
        args: Vec<String>,

        /// Run sandbox in the background
        #[arg(long)]
        detach: bool,
    },

    /// Open a shell in a sandbox
    #[command(name = "shell")]
    Shell {
        /// Specifies the sandbox
        #[arg(conflicts_with = "sandbox_with_flag", name = "SANDBOX")]
        sandbox: Option<String>,

        /// Specifies the sandbox
        #[arg(short, long = "sandbox", conflicts_with = "sandbox", name = "SANDBOX")]
        sandbox_with_flag: Option<String>,

        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Config path
        #[arg(short, long)]
        config: Option<String>,

        /// Additional arguments
        #[arg(last = true)]
        args: Vec<String>,

        /// Run sandbox in the background
        #[arg(long)]
        detach: bool,
    },

    /// Create a temporary sandbox
    #[command(name = "tmp")]
    Tmp {
        /// Specifies the image and script to run
        #[arg(conflicts_with = "image_script_with_flag", name = "IMAGE~SCRIPT")]
        image_script: Option<String>,

        /// Specifies the image and script to run
        #[arg(
            short,
            long = "image",
            conflicts_with = "image_script",
            name = "IMAGE~SCRIPT"
        )]
        image_script_with_flag: Option<String>,

        /// Number of CPUs
        #[arg(long)]
        cpus: Option<u8>,

        /// RAM in MB
        #[arg(long)]
        ram: Option<u32>,

        /// Volume mappings
        #[arg(long)]
        volumes: Vec<String>,

        /// Port mappings
        #[arg(long)]
        ports: Vec<String>,

        /// Environment variables
        #[arg(long)]
        envs: Vec<String>,

        /// Working directory
        #[arg(long)]
        workdir: Option<Utf8UnixPathBuf>,
    },

    /// Install a script from an image
    #[command(name = "install")]
    Install {
        /// Whether to install from an image
        #[arg(short, long, default_value_t = true)]
        image: bool,

        /// Whether to install from an image group
        #[arg(short = 'G', long)]
        image_group: bool,

        /// Name of the image or image group
        name: String,

        /// Script to install
        script: Option<String>,

        /// New name for the script
        rename: Option<String>,
    },

    /// Uninstall a script
    #[command(name = "uninstall")]
    Uninstall {
        /// Script to uninstall
        script: Option<String>,

        /// Whether to uninstall from an image
        #[arg(short, long, default_value_t = true)]
        image: bool,

        /// Whether to uninstall from an image group
        #[arg(short = 'G', long)]
        image_group: bool,

        /// Name of the image or image group
        name: String,
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
        /// Target sandboxes
        #[arg(short, long, default_value_t = true)]
        sandbox: bool,

        /// Target group
        #[arg(short, long)]
        group: bool,

        /// Names of components to start
        names: Vec<String>,
    },

    /// Stop project sandboxes
    #[command(name = "down")]
    Down {
        /// Target sandboxes
        #[arg(short, long, default_value_t = true)]
        sandbox: bool,

        /// Target group
        #[arg(short, long)]
        group: bool,

        /// Names of components to stop
        names: Vec<String>,
    },

    /// Show running status
    #[command(name = "status")]
    Status {
        /// Target sandboxes
        #[arg(short, long, default_value_t = true)]
        sandbox: bool,

        /// Target group
        #[arg(short, long)]
        group: bool,

        /// Names of components to check
        names: Vec<String>,
    },

    /// Clean project data
    #[command(name = "clean")]
    Clean,

    /// Build images
    #[command(name = "build")]
    Build {
        /// Build from build definition
        #[arg(short, long)]
        build: bool,

        /// Build from sandbox
        #[arg(short, long, default_value_t = true)]
        sandbox: bool,

        /// Build from group
        #[arg(short, long)]
        group: bool,

        /// Name of the component
        #[arg(required = true)]
        name: String,

        /// Create a snapshot
        #[arg(long)]
        snapshot: bool,
    },

    /// Pull an image
    #[command(name = "pull")]
    Pull {
        /// Whether to pull an image
        #[arg(short, long, default_value_t = true)]
        image: bool,

        /// Whether to pull an image group
        #[arg(short = 'G', long)]
        image_group: bool,

        /// Name of the image or image group
        name: Reference,

        /// Path to store the layer files
        #[arg(short = 'L', long)]
        layer_path: Option<PathBuf>,
    },

    /// Push an image
    #[command(name = "push")]
    Push {
        /// Image to push
        #[arg(short, long)]
        image: String,
    },

    /// Manage monocore itself
    #[command(name = "self")]
    Self_ {
        /// Action to perform
        #[arg(value_enum)]
        action: SelfAction,
    },

    /// Deploy to cloud
    #[command(name = "deploy")]
    Deploy {
        /// Deploy sandbox
        #[arg(short, long)]
        sandbox: bool,

        /// Deploy group
        #[arg(short, long)]
        group: bool,

        /// Name of component to deploy
        name: Option<String>,
    },

    /// Start a server for orchestrating sandboxes
    #[command(name = "server")]
    Server {
        /// Port to listen on
        #[arg(long)]
        port: Option<u16>,

        /// Daemon control
        #[arg(long)]
        daemon: Option<String>,
    },

    /// Version of monocore
    #[command(name = "version")]
    Version,
}

/// Actions for the self subcommand
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SelfAction {
    /// Upgrade monocore
    Upgrade,

    /// Uninstall monocore
    Uninstall,
}

//-------------------------------------------------------------------------------------------------
// Methods
//-------------------------------------------------------------------------------------------------
