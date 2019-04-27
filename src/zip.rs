use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use zip_::ZipArchive;

use crate::err::Error;
use crate::{File, Result, Store};

/// Zip archive store.
pub struct Zip {
    bytes: Box<[u8]>,
}

impl Zip {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_reader(fs::File::open(path)?)
    }

    pub fn from_reader<R: Read>(mut read: R) -> Result<Self> {
        let mut inner = Vec::new();
        read.read_to_end(&mut inner)?;
        Ok(Self::new(inner))
    }

    pub fn new<B: Into<Box<[u8]>>>(bytes: B) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }
}

impl Store for Zip {
    fn open(&self, path: &Path) -> Result<File> {
        let mut archive = ZipArchive::new(Cursor::new(&self.bytes))?;
        let name = path.to_str().ok_or(Error::Utf8)?;
        let mut file = archive.by_name(name)?;

        let mut v = Vec::new();
        file.read_to_end(&mut v)?;
        Ok(File::from_ram(v))
    }
}
