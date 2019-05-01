use std::fmt;
use std::io;

use failure::Fail;
#[cfg(feature = "zip")]
use zip_::result::ZipError;

/// Error types used
#[derive(Debug)]
pub enum Error {
    FileNotFound,
    Io(io::Error),
    /// Utf-8 conversion error,
    Utf8,
    #[cfg(feature = "zip")]
    UnsupportedZip,
    #[cfg(feature = "zip")]
    InvalidZip,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::FileNotFound => write!(f, "File not found"),
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Utf8 => write!(f, "Utf8 conversion error"),
            Error::UnsupportedZip => write!(f, "Unsupported Zip compression algorithm."),
            Error::InvalidZip => write!(f, "Invalid Zip file"),
        }
    }
}

impl std::error::Error for Error {}

#[doc(hidden)]
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Error::FileNotFound,
            _ => Error::Io(err),
        }
    }
}

#[doc(hidden)]
#[cfg(feature = "zip")]
impl From<ZipError> for Error {
    fn from(e: ZipError) -> Self {
        match e {
            ZipError::Io(e) => Error::Io(e),
            ZipError::FileNotFound => Error::FileNotFound,
            ZipError::InvalidArchive(_) => Error::InvalidZip,
            ZipError::UnsupportedArchive(_) => Error::UnsupportedZip,
        }
    }
}
