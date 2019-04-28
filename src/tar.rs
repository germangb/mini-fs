use std::cell::Cell;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use flate2::read::GzDecoder;
use tar_::Archive;

use crate::err::{Error, Result};
use crate::file::File;
use crate::Store;

/// Tar archive store.
///
/// ```should_panic
/// use mini_fs::tar::Tar;
/// # use mini_fs::err::Error;
///
/// # fn main() -> Result<(), Error> {
/// let tar = Tar::open("example.tar")?;
///
/// // gzip files also supported:
/// let targz = Tar::open("example.tar.gz")?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Tar {
    gzip: Cell<bool>,
    bytes: Box<[u8]>,
}

impl Tar {
    fn open_read<R: Read>(&self, path: &Path, reader: R) -> Result<File> {
        let mut archive = Archive::new(reader);
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

impl Store for Tar {
    fn open(&self, path: &Path) -> Result<File> {
        if self.gzip.get() {
            self.open_read(path, GzDecoder::new(Cursor::new(&self.bytes)))
        } else {
            match self.open_read(path, Cursor::new(&self.bytes)) {
                Err(Error::FileNotFound) => Err(Error::FileNotFound),
                Err(_) => {
                    self.gzip.set(true);
                    self.open(path)
                }
                Ok(entry) => Ok(entry),
            }
        }
    }
}

impl Tar {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_reader(fs::File::open(path)?)
    }

    pub fn new<B: Into<Box<[u8]>>>(bytes: B) -> Self {
        Self {
            bytes: bytes.into(),
            gzip: Cell::new(false),
        }
    }

    pub fn from_reader<R: Read>(mut read: R) -> Result<Self> {
        let mut inner = Vec::new();
        read.read_to_end(&mut inner)?;
        Ok(Self::new(inner))
    }
}
