use std::cell::RefCell;
use std::fs;
use std::io::{self, ErrorKind, Read, Seek, SeekFrom};
use std::ops::DerefMut;
use std::path::Path;

use zip_::ZipArchive;

use crate::file;
use crate::Store;

/// Zip archive store.
pub struct Zip<T: Read + Seek> {
    inner: RefCell<T>,
}

impl Zip<fs::File> {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(path)?;
        Ok(Self::new(file))
    }
}

impl<T: Read + Seek> Zip<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }
}

impl<T: Read + Seek> Store for Zip<T> {
    type File = file::File;
    fn open(&self, path: &Path) -> io::Result<file::File> {
        let mut file = self.inner.borrow_mut();
        file.seek(SeekFrom::Start(0))?;

        let mut archive = ZipArchive::new(file.deref_mut())?;
        let name = path.to_str().ok_or(io::Error::new(
            ErrorKind::Other,
            "Utf8 path conversion error.",
        ));
        let mut file = archive.by_name(name?)?;

        let mut v = Vec::new();
        file.read_to_end(&mut v)?;
        Ok(file::File::from_ram(v))
    }
}
