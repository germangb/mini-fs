use std::io;

use failure::Fail;
#[cfg(feature = "zip")]
use zip_::result::ZipError;

/// Custom result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types used
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "File not found.")]
    FileNotFound,
    #[fail(display = "IO error: {}", 0)]
    Io(io::Error),
    /// Utf-8 conversion error,
    #[fail(display = "UTF8 conversion error.")]
    Utf8,
    #[cfg(feature = "zip")]
    #[fail(display = "Unsupported Zip compression algorithm.")]
    UnsupportedZip,
    #[cfg(feature = "zip")]
    #[fail(display = "File is likely not a Zip archive.")]
    InvalidZip,
}

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
