use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    FileNotFound,
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Error::FileNotFound,
            _ => Error::Io(err),
        }
    }
}
