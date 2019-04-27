use std::io;

#[cfg(feature = "zip")]
use zip_::result::ZipError;

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
