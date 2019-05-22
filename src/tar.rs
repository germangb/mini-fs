use std::cell::{Cell, RefCell};
use std::fs;
use std::io::{self, Cursor, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use flate2::read::GzDecoder;
use tar_::Archive;

use crate::index::Index;
use crate::store::Store;
use crate::{Entries, Entry};

/// Tar archive.
///
/// # Remarks
///
/// When used with a `std::fs::File`, the file will remain open for the lifetime
/// of the Tar.
pub struct TarFs<F: Read + Seek> {
    gzip: Cell<bool>,
    inner: RefCell<F>,
    index: Option<Index<SeekFrom>>,
}

/// Entry in the Tar archive.
pub struct TarFsFile {
    inner: Cursor<Box<[u8]>>,
}

impl Read for TarFsFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for TarFsFile {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<T: Read + Seek> Store for TarFs<T> {
    type File = TarFsFile;

    fn open_path(&self, path: &Path) -> io::Result<Self::File> {
        if self.gzip.get() {
            let mut file = self.inner.borrow_mut();
            file.seek(SeekFrom::Start(0))?;
            self.open_read(path, GzDecoder::new(&mut *file))
        } else {
            let mut file = self.inner.borrow_mut();
            file.seek(SeekFrom::Start(0))?;
            match self.open_read(path, &mut *file) {
                Ok(entry) => Ok(entry),
                Err(ref e) if e.kind() == ErrorKind::NotFound => {
                    Err(io::Error::from(ErrorKind::NotFound))
                }
                Err(_) => {
                    self.gzip.set(true);
                    drop(file);
                    self.open_path(path)
                }
            }
        }
    }

    fn entries_path(&self, path: &Path) -> io::Result<Entries> {
        if let Some(ref idx) = self.index {
            Ok(Entries::new(idx.entries(path).map(|ent| {
                let name = ent.name.to_os_string();
                let kind = ent.kind;
                Ok(Entry { name, kind })
            })))
        } else {
            panic!("You have to call the `Zip::index` method on this zip archive before you can list its entries.")
        }
    }
}

impl TarFs<fs::File> {
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

impl<T: Read + Seek> TarFs<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: RefCell::new(inner),
            gzip: Cell::new(false),
            index: None,
        }
    }

    fn open_read<R: Read>(&self, path: &Path, read: R) -> io::Result<TarFsFile> {
        let mut archive = Archive::new(read);
        for entry in archive.entries()? {
            let mut entry = entry?;
            if path == entry.path()? {
                let mut data = Vec::new();
                entry.read_to_end(&mut data)?;
                return Ok(TarFsFile {
                    inner: Cursor::new(data.into()),
                });
            }
        }
        Err(io::Error::from(ErrorKind::NotFound))
    }

    /// Index the contents of the archive.
    ///
    /// Having an index allows you to list the contents of the archive using the
    /// entries_path and entries methods.
    pub fn index(self) -> io::Result<Self> {
        unimplemented!("TarFs indexing hasn't been implemented yet.");
    }
}
