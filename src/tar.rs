use std::cell::{Cell, RefCell};
use std::fs;
use std::io::{self, ErrorKind, Read, Seek, SeekFrom};
use std::ops::DerefMut;
use std::path::Path;

use flate2::read::GzDecoder;
use tar_::Archive;

use crate::file;
use crate::Store;

/// Tar archive.
pub struct Tar<F: Read + Seek> {
    gzip: Cell<bool>,
    inner: RefCell<F>,
}

impl<T: Read + Seek> Store for Tar<T> {
    type File = file::File;
    fn open(&self, path: &Path) -> io::Result<file::File> {
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
                        self.open(path)
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

    fn open_read<R: Read>(&self, path: &Path, read: R) -> io::Result<file::File> {
        let mut archive = Archive::new(read);
        for entry in archive.entries()? {
            let mut entry = entry?;
            if path == entry.path()? {
                let mut data = Vec::new();
                entry.read_to_end(&mut data)?;
                return Ok(file::File::from_ram(data));
            }
        }
        Err(io::Error::from(ErrorKind::NotFound))
    }
}
