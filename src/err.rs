use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    FileNotFound(PathBuf),
    Mount(MountError),
    Io(io::Error),
}

#[derive(Debug)]
pub enum MountError {
    AlreadyMounted(PathBuf),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
