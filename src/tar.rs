use std::cell::{Cell, RefCell};
use std::fs;
use std::io::{self, Cursor, ErrorKind, Read, Seek, SeekFrom};
use std::ops::DerefMut;
use std::path::Path;

use flate2::read::GzDecoder;
use tar_::Archive;

use crate::store::Store;

/// Tar archive.
pub struct Tar<F: Read + Seek> {
    gzip: Cell<bool>,
    inner: RefCell<F>,
}

/// Entry in the Tar archive.
pub struct TarEntry {
    inner: Cursor<Box<[u8]>>,
}

impl Read for TarEntry {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for TarEntry {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<T: Read + Seek> Store for Tar<T> {
    type File = TarEntry;
    fn open_path(&self, path: &Path) -> io::Result<Self::File> {
        if self.gzip.get() {
            let mut file = self.inner.borrow_mut();
            file.seek(SeekFrom::Start(0))?;
            self.open_read(path, GzDecoder::new(file.deref_mut()))
        } else {
            let mut file = self.inner.borrow_mut();
            file.seek(SeekFrom::Start(0))?;
            match self.open_read(path, file.deref_mut()) {
                Err(e) => {
                    if e.kind() == ErrorKind::NotFound {
                        return Err(io::Error::from(ErrorKind::NotFound));
                    } else {
                        self.gzip.set(true);
                        drop(file);
                        self.open_path(path)
                    }
                }
                Ok(entry) => Ok(entry),
            }
        }
    }
}

impl Tar<fs::File> {
    /// Open a file from the native filesystem.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(path)?;
        Ok(Self::new(file))
    }
}

impl<T: Read + Seek> Tar<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: RefCell::new(inner),
            gzip: Cell::new(false),
        }
    }

    fn open_read<R: Read>(&self, path: &Path, read: R) -> io::Result<TarEntry> {
        let mut archive = Archive::new(read);
        for entry in archive.entries()? {
            let mut entry = entry?;
            if path == entry.path()? {
                let mut data = Vec::new();
                entry.read_to_end(&mut data)?;
                return Ok(TarEntry {
                    inner: Cursor::new(data.into()),
                });
            }
        }
        Err(io::Error::from(ErrorKind::NotFound))
    }
}
