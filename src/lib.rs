//! **mini-fs** is a filesystem abstraction layer.
//!
//! Currently supported features include:
//!
//! - Access to the local (native) filesystem.
//! - In-memory filesystems.
//! - Tar, tar.gz, and Zip archives.
//! - Logically merging filesystems (useful for overriding some files)
//!
//! ## Case sensitivity
//!
//! Paths defined by the virtual filesystem are **case sensitive**¹.
//!
//! ¹ Except when you use `Local` which uses `std::fs` internally.
use std::collections::{BTreeMap, LinkedList};
use std::env;
use std::fs;
use std::io::{self, Cursor, Error, ErrorKind, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[cfg(feature = "s3")]
pub use s3::S3;
use std::any::Any;
pub use store::{MapFile, Store};
#[cfg(feature = "tar")]
pub use tar::Tar;
#[cfg(feature = "zip")]
pub use zip::Zip;

#[cfg(feature = "tar")]
use tar::TarEntry;
#[cfg(feature = "zip")]
use zip::ZipEntry;

/// S3 bucket from Aws.
#[cfg(feature = "s3")]
pub mod s3;
/// File storage generic.
mod store;
/// Tar file storage.
#[cfg(feature = "tar")]
pub mod tar;
/// Zip file storage.
#[cfg(feature = "zip")]
pub mod zip;

/// File you can Read bytes from.
pub enum File {
    Local(fs::File),
    Ram(Cursor<Rc<[u8]>>),

    #[cfg(feature = "tar")]
    Zip(ZipEntry),
    #[cfg(feature = "zip")]
    Tar(TarEntry),

    #[cfg(feature = "s3")]
    S3(()),

    // A layer of dynamic typing is added for custom Stores.
    // That way, this cost is only paid for external implementations of Store.
    Custom(Box<dyn Custom + Send>),
}

/// Custom file type.
pub trait Custom: Any + Read + Seek {}

impl From<fs::File> for File {
    fn from(file: fs::File) -> Self {
        File::Local(file)
    }
}

impl From<Cursor<Rc<[u8]>>> for File {
    fn from(file: Cursor<Rc<[u8]>>) -> Self {
        File::Ram(file)
    }
}

#[cfg(feature = "tar")]
impl From<TarEntry> for File {
    fn from(file: TarEntry) -> Self {
        File::Tar(file)
    }
}

#[cfg(feature = "zip")]
impl From<ZipEntry> for File {
    fn from(file: ZipEntry) -> Self {
        File::Zip(file)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            File::Local(ref mut file) => file.read(buf),
            File::Ram(ref mut file) => file.read(buf),
            #[cfg(feature = "zip")]
            File::Zip(ref mut file) => file.read(buf),
            #[cfg(feature = "tar")]
            File::Tar(ref mut file) => file.read(buf),
            File::Custom(ref mut file) => file.read(buf),
        }
    }
}

struct Mount {
    path: PathBuf,
    store: Box<dyn Store<File = File>>,
}

/// Virtual filesystem.
pub struct MiniFs {
    mount: LinkedList<Mount>,
}

impl Store for MiniFs {
    type File = File;

    fn open_path(&self, path: &Path) -> io::Result<File> {
        let next = self.mount.iter().rev().find_map(|mnt| {
            if let Ok(np) = path.strip_prefix(&mnt.path) {
                Some((np, &mnt.store))
            } else {
                None
            }
        });
        if let Some((np, store)) = next {
            store.open_path(np)
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    }
}

impl MiniFs {
    pub fn new() -> Self {
        Self {
            mount: LinkedList::new(),
        }
    }

    pub fn mount<P, S, T>(mut self, path: P, store: S) -> Self
    where
        P: Into<PathBuf>,
        S: Store<File = T> + 'static,
        T: Into<File>,
    {
        let path = path.into();
        let store = Box::new(store.map_file(|file| file.into()));
        self.mount.push_back(Mount { path, store });
        self
    }

    pub fn umount<P>(&mut self, path: P) -> Option<Box<dyn Store<File = File>>>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if let Some(p) = self.mount.iter().rposition(|p| p.path == path) {
            let mut tail = self.mount.split_off(p);
            let fs = tail.pop_front().map(|m| m.store);
            self.mount.append(&mut tail);
            fs
        } else {
            None
        }
    }
}

/// Native file store.
pub struct Local {
    root: PathBuf,
}

impl Store for Local {
    type File = fs::File;

    fn open_path(&self, path: &Path) -> io::Result<fs::File> {
        fs::OpenOptions::new()
            .create(false)
            .read(true)
            .write(false)
            .open(self.root.join(path))
    }

    fn open_write_path(&self, path: &Path) -> io::Result<fs::File> {
        fs::OpenOptions::new()
            .create(false)
            .read(false)
            .write(true)
            .open(self.root.join(path))
    }
}

impl Local {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    /// Point to the current working directory.
    pub fn pwd() -> io::Result<Self> {
        Ok(Self::new(env::current_dir()?))
    }
}

/// In-memory file storage
pub struct Ram {
    inner: BTreeMap<PathBuf, Rc<[u8]>>,
}

impl Store for Ram {
    type File = Cursor<Rc<[u8]>>;

    fn open_path(&self, path: &Path) -> io::Result<Self::File> {
        match self.inner.get(path) {
            Some(file) => Ok(Cursor::new(Rc::clone(file))),
            None => Err(Error::from(ErrorKind::NotFound)),
        }
    }
}

impl Ram {
    pub fn new() -> Self {
        Ram {
            inner: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn rm<P: AsRef<Path>>(&mut self, path: P) -> Option<Rc<[u8]>> {
        self.inner.remove(path.as_ref())
    }

    pub fn touch<P, F>(&mut self, path: P, file: F)
    where
        P: Into<PathBuf>,
        F: Into<Rc<[u8]>>,
    {
        self.inner.insert(path.into(), file.into());
    }
}
