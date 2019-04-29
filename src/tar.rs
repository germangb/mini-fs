//! ```should_panic
//! use mini_fs::tar::Tar;
//! # use mini_fs::err::Error;
//!
//! # fn main() -> Result<(), Error> {
//! let tar = Tar::open("example.tar")?;
//!
//! // gzip files also supported:
//! let targz = Tar::open("example.tar.gz")?;
//! # Ok(())
//! # }
//! ```
use std::cell::{Cell, RefCell};
use std::fs;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::ops::DerefMut;
use std::path::Path;

use flate2::read::GzDecoder;
use tar_::Archive;

use crate::err::{Error, Result};
use crate::file::File;
use crate::Store;

/// In-memory tar file.
pub struct Tar {
    inner: BigTar<Cursor<Box<[u8]>>>,
}

/// Tar archive.
pub struct BigTar<F: Read + Seek> {
    gzip: Cell<bool>,
    inner: RefCell<F>,
}

impl Store for Tar {
    fn open(&self, path: &Path) -> Result<File> {
        self.inner.open(path)
    }
}

impl<T: Read + Seek> Store for BigTar<T> {
    fn open(&self, path: &Path) -> Result<File> {
        if self.gzip.get() {
            let mut file = self.inner.borrow_mut();
            file.seek(SeekFrom::Start(0))?;
            self.open_read(path, GzDecoder::new(file.deref_mut()))
        } else {
            let mut file = self.inner.borrow_mut();
            file.seek(SeekFrom::Start(0))?;
            match self.open_read(path, file.deref_mut()) {
                Err(Error::FileNotFound) => Err(Error::FileNotFound),
                Err(_) => {
                    self.gzip.set(true);
                    drop(file);
                    self.open(path)
                }
                Ok(entry) => Ok(entry),
            }
        }
    }
}

impl BigTar<fs::File> {
    /// Open a file from the native filesystem.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = fs::OpenOptions::new().create(false).open(path)?;
        Ok(Self::new(file))
    }
}

impl<T: Read + Seek> BigTar<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: RefCell::new(inner),
            gzip: Cell::new(false),
        }
    }

    fn open_read<R: Read>(&self, path: &Path, read: R) -> Result<File> {
        let mut archive = Archive::new(read);
        for entry in archive.entries()? {
            let mut entry = entry?;
            if path == entry.path()? {
                let mut data = Vec::new();
                entry.read_to_end(&mut data)?;
                return Ok(File::from_ram(data));
            }
        }
        Err(Error::FileNotFound)
    }
}

impl Tar {
    /// Consume the reader until the end and keep the Tar file in memory for the
    /// lifetime of this object.
    pub fn from_reader<R: Read>(read: &mut R) -> Result<Self> {
        let mut ram = Vec::new();
        read.read_to_end(&mut ram)?;
        Ok(Self::new(ram))
    }

    /// Open a file and read all its content.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = fs::File::open(path)?;
        Self::from_reader(&mut file)
    }

    pub fn new<T: Into<Box<[u8]>>>(bytes: T) -> Self {
        let cursor = Cursor::new(bytes.into());
        Self {
            inner: BigTar::new(cursor),
        }
    }
}
