#[cfg(target_os = "linux")]
use procfs::ProcError;
use serde::Serialize;
use std::io;
use sysexits::ExitCode;
use thiserror::Error;

pub type Pid = u32;

#[derive(Debug, Serialize)]
pub struct ProcStats {
    pub pid: Pid,
    pub socket_descriptors: u32,
    pub file_descriptors: u32,
}

#[derive(Debug, Serialize)]
pub struct Failure<'a> {
    pub message: &'a str,
    pub details: &'a str,
}

#[derive(Error, Debug)]
pub enum FshcError {
    #[error("only pid numbers between 1 and 99999 are supported")]
    PidOutOfRange,
    #[error("could not locate a process for the given pid")]
    InvalidInput,
    #[error("insufficient permission to inspect file descriptors of the target process")]
    PermissionDenied,
    #[error("failed to fetch file descriptor details for the target process")]
    IoError,
    #[error("failed to fetch file descriptor details for the target process")]
    Other,
    #[cfg(target_os = "macos")]
    #[error("{0}")]
    Errno(String),
}

pub trait ExitCodeProvider {
    fn exit_code(&self) -> ExitCode {
        ExitCode::DataErr
    }
}

impl ExitCodeProvider for FshcError {
    fn exit_code(&self) -> ExitCode {
        match self {
            FshcError::PidOutOfRange => ExitCode::DataErr,
            FshcError::PermissionDenied => ExitCode::NoPerm,
            FshcError::IoError => ExitCode::IoErr,
            FshcError::InvalidInput => ExitCode::DataErr,
            FshcError::Other => ExitCode::OsErr,
            #[cfg(target_os = "macos")]
            FshcError::Errno(_) => ExitCode::OsErr,
        }
    }
}

impl ExitCodeProvider for io::Error {
    fn exit_code(&self) -> ExitCode {
        match self.kind() {
            io::ErrorKind::PermissionDenied => ExitCode::NoPerm,
            io::ErrorKind::NotFound => ExitCode::DataErr,
            io::ErrorKind::InvalidInput => ExitCode::DataErr,
            io::ErrorKind::BrokenPipe => ExitCode::IoErr,
            _ => ExitCode::DataErr,
        }
    }
}

#[cfg(target_os = "linux")]
impl ExitCodeProvider for ProcError {
    fn exit_code(&self) -> ExitCode {
        match self {
            ProcError::PermissionDenied(_) => ExitCode::NoPerm,
            ProcError::NotFound(_) => ExitCode::DataErr,
            ProcError::Io(_, _) => ExitCode::IoErr,
            _ => ExitCode::Unavailable,
        }
    }
}

pub type FshcResult = Result<ProcStats, FshcError>;

impl From<io::Error> for FshcError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::PermissionDenied => FshcError::PermissionDenied,
            io::ErrorKind::NotFound => FshcError::InvalidInput,
            io::ErrorKind::InvalidInput => FshcError::InvalidInput,
            io::ErrorKind::BrokenPipe => FshcError::IoError,
            _ => FshcError::Other,
        }
    }
}

#[cfg(target_os = "macos")]
impl From<String> for FshcError {
    fn from(value: String) -> Self {
        FshcError::Errno(value)
    }
}

#[cfg(target_os = "linux")]
impl From<procfs::ProcError> for FshcError {
    fn from(value: procfs::ProcError) -> Self {
        match value {
            ProcError::PermissionDenied(_) => FshcError::PermissionDenied,
            ProcError::NotFound(_) => FshcError::InvalidInput,
            ProcError::Incomplete(_) => FshcError::IoError,
            ProcError::Io(_, _) => FshcError::IoError,
            ProcError::Other(_) => FshcError::Other,
            ProcError::InternalError(_) => FshcError::Other,
        }
    }
}
