use std::cell::RefCell;
use std::fs;
use std::io::{self, Cursor, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use zip_::ZipArchive;

use crate::index::Index;
use crate::store::Store;
use crate::{Entries, Entry};

/// Zip archive store.
///
/// # Remarks
///
/// When used with a `std::fs::File`, the file will remain open for the lifetime
/// of the Zip.
pub struct ZipFs<T: Read + Seek> {
    inner: RefCell<T>,
    index: Option<Index<()>>,
}

/// Entry in the Zip archive.
pub struct ZipFsFile {
    inner: Cursor<Box<[u8]>>,
}

impl Read for ZipFsFile {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for ZipFsFile {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl ZipFs<fs::File> {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(path)?;
        Ok(Self::new(file))
    }
}

impl<T: Read + Seek> ZipFs<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: RefCell::new(inner),
            index: None,
        }
    }

    /// Index the contents of the archive.
    ///
    /// Having an index allows you to list the contents of the archive using the
    /// entries_path and entries methods.
    pub fn index(mut self) -> io::Result<Self> {
        let mut index = Index::new();
        let mut file = self.inner.borrow_mut();
        file.seek(SeekFrom::Start(0))?;
        let mut archive = ZipArchive::new(&mut *file)?;
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let path = file.sanitized_name();

            index.insert(path, ());
        }
        self.index = Some(index);
        drop(file);
        Ok(self)
    }
}

impl<T: Read + Seek> Store for ZipFs<T> {
    type File = ZipFsFile;
    fn open_path(&self, path: &Path) -> io::Result<Self::File> {
        let mut file = self.inner.borrow_mut();
        file.seek(SeekFrom::Start(0))?;

        let mut archive = ZipArchive::new(&mut *file)?;
        let name = path.to_str().ok_or(io::Error::new(
            ErrorKind::Other,
            "Utf8 path conversion error.",
        ));
        let mut file = archive.by_name(name?)?;

        let mut v = Vec::new();
        file.read_to_end(&mut v)?;
        Ok(ZipFsFile {
            inner: Cursor::new(v.into()),
        })
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
