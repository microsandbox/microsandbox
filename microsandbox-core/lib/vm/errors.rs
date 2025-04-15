#[derive(Debug, Error)]
pub enum InvalidMicroVMConfigError {
    /// The root path for the MicroVm does not exist.
    #[error("The root path {0} does not exist")]
    RootPathDoesNotExist(PathBuf),

    /// The specified memory is zero.
    #[error("The specified memory is zero")]
    MemoryIsZero,

    /// The specified executable path does not exist.
    #[error("The executable path {0} does not exist")]
    ExecutablePathDoesNotExist(Utf8UnixPathBuf),

    /// The command line string contains invalid characters.
    #[error("The command line string '{0}' contains invalid characters")]
    InvalidCommandLineString(String),

    /// The specified guest paths conflict.
    #[error("The guest paths {0} and {1} conflict")]
    ConflictingGuestPaths(String, String),
}
