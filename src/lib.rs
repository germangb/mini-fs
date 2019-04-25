use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};

pub mod err;
#[cfg(feature = "tar")]
pub mod tar;
#[cfg(feature = "zip")]
pub mod zip;

use crate::err::Error::FileNotFound;
use crate::err::MountError::AlreadyMounted;
use err::{Error, MountError};
use std::collections::{HashMap, HashSet};

pub type File<'a> = Cursor<Cow<'a, [u8]>>;
pub type Result<T> = std::result::Result<T, Error>;

pub trait Store {
    fn open(&self, path: &Path) -> Result<File>;
}

pub struct Local {
    root: PathBuf,
}

impl Store for Local {
    fn open(&self, path: &Path) -> Result<File> {
        let file = fs::read(path)?;
        Ok(Cursor::new(Cow::Owned(file)))
    }
}

impl Local {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    pub fn pwd() -> Result<Self> {
        Ok(Self::new(env::current_dir()?))
    }
}

pub struct Ram;

pub struct Merge<A, B>(pub A, pub B);

impl<A: Store, B: Store> Store for Merge<A, B> {
    fn open(&self, path: &Path) -> Result<File> {
        self.0.open(path).or_else(|e| self.1.open(path))
    }
}

pub struct Config(Result<HashMap<PathBuf, Box<dyn Store>>>);
pub struct MiniFs(HashMap<PathBuf, Box<dyn Store>>);

impl Config {
    pub fn mount<P, F>(mut self, path: P, store: F) -> Self
    where
        P: Into<PathBuf>,
        F: Store + 'static,
    {
        let mut err = None;
        if let Ok(ref mut mnt) = self.0 {
            let path = path.into();
            if mnt.contains_key(&path) {
                err = Some(Error::Mount(AlreadyMounted(path)));
            } else {
                mnt.insert(path, Box::new(store));
            }
        }
        if let Some(err) = err {
            self.0 = Err(err);
        }
        self
    }

    pub fn build(self) -> Result<MiniFs> {
        Ok(MiniFs(self.0?))
    }
}

impl MiniFs {
    pub fn new() -> Config {
        Config(Ok(Default::default()))
    }
}

impl Store for MiniFs {
    // TODO Implement directory tree
    fn open(&self, path: &Path) -> Result<File> {
        for (mnt, fs) in self.0.iter() {
            if let Ok(stripped) = path.strip_prefix(mnt) {
                match fs.open(stripped) {
                    Err(Error::Io(ref e)) if e.kind() == io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e),
                    Ok(file) => return Ok(file),
                }
            }
        }
        Err(Error::FileNotFound(path.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
