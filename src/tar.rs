use std::cell::RefCell;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use tar_::Archive;

use crate::err::Error;
use crate::file::File;
use crate::{Result, Store};

pub struct Tar {
    inner: Vec<u8>,
}

impl Store for Tar {
    fn open(&self, path: &Path) -> Result<File> {
        let mut archive = Archive::new(Cursor::new(&self.inner));
        for entry in archive.entries()? {
            let mut entry = entry?;
            if path == entry.path()? {
                let mut data = Vec::new();
                let _ = entry.read_to_end(&mut data)?;
                return Ok(File::from_ram(data));
            }
        }
        Err(Error::FileNotFound)
    }
}

impl Tar {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_reader(fs::File::open(path)?)
    }

    pub fn from_reader<R: Read>(mut read: R) -> Result<Self> {
        let mut inner = Vec::new();
        read.read_to_end(&mut inner)?;
        Ok(Self { inner })
    }
}
