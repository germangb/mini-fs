use std::cell::RefCell;
use std::fs;
use std::io::{self, Cursor, ErrorKind, Read, Seek, SeekFrom, Write};
use std::ops::DerefMut;
use std::path::Path;

use zip_::ZipArchive;

use crate::store::Store;
use std::rc::Rc;

/// Zip archive store.
pub struct Zip<T: Read + Seek> {
    inner: RefCell<T>,
}

/// Entry in the Zip archive.
pub struct ZipEntry {
    inner: Cursor<Box<[u8]>>,
}

impl Read for ZipEntry {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for ZipEntry {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
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

    /// Build a file index to be able to call the "entries" trait method on this
    /// archive.
    pub fn index(self) -> io::Result<Self> {
        unimplemented!()
    }
}

impl<T: Read + Seek> Store for Zip<T> {
    type File = ZipEntry;
    fn open_path(&self, path: &Path) -> io::Result<Self::File> {
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
        Ok(ZipEntry {
            inner: Cursor::new(v.into()),
        })
    }
}
