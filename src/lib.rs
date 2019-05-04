//! **mini-fs** is an extensible virtual filesystem for the application layer.
//!
//! Currently supported features include:
//!
//! - Access to the local (native) filesystem.
//! - In-memory filesystems.
//! - Read from tar, tar.gz, and zip archives.
//! - Filesystem overlays.
//!
//! ## Case sensitivity
//!
//! Paths defined by the virtual filesystem are **case sensitive**¹.
//!
//! ¹ Except when you use `Local` which uses `std::fs` internally.
//!
//! ## Example
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use mini_fs::{Local, MiniFs, Store, Zip};
//!
//! let gfx = Local::new("./res/images");
//! let sfx = Zip::open("archive.tar.gz")?;
//!
//! let fs = MiniFs::new()
//!     .mount("/assets/gfx", gfx)
//!     .mount("/assets/sfx", sfx);
//!
//! let file = fs.open("/assets/gfx/trash.gif")?;
//! # Ok(())
//! # }
//! ```
use std::collections::{BTreeMap, LinkedList};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use std::any::Any;
pub use store::Store;
#[cfg(feature = "tar")]
pub use tar::Tar;
#[cfg(feature = "zip")]
pub use zip::Zip;

use std::ffi::OsString;
#[cfg(feature = "tar")]
use tar::TarEntry;
#[cfg(feature = "zip")]
use zip::ZipEntry;

/// File storage generic.
mod store;
/// Tar file storage.
#[cfg(feature = "tar")]
pub mod tar;
/// Zip file storage.
#[cfg(feature = "zip")]
pub mod zip;

/// File you can seek and read from.
pub enum File {
    Local(fs::File),
    Ram(RamFile),
    #[cfg(feature = "tar")]
    Zip(ZipEntry),
    #[cfg(feature = "zip")]
    Tar(TarEntry),
    // External types are dynamic
    User(Box<dyn UserFile>),
}

/// Custom file type.
pub trait UserFile: Any + io::Read + io::Seek + Send {}

impl<T: UserFile> From<T> for File {
    fn from(file: T) -> Self {
        File::User(Box::new(file))
    }
}

impl From<fs::File> for File {
    fn from(file: fs::File) -> Self {
        File::Local(file)
    }
}

impl From<RamFile> for File {
    fn from(file: RamFile) -> Self {
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

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            File::Local(ref mut file) => file.read(buf),
            File::Ram(ref mut file) => file.read(buf),
            #[cfg(feature = "zip")]
            File::Zip(ref mut file) => file.read(buf),
            #[cfg(feature = "tar")]
            File::Tar(ref mut file) => file.read(buf),
            File::User(ref mut file) => file.read(buf),
        }
    }
}

impl io::Seek for File {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match self {
            File::Local(ref mut file) => file.seek(pos),
            File::Ram(ref mut file) => file.seek(pos),
            #[cfg(feature = "zip")]
            File::Zip(ref mut file) => file.seek(pos),
            #[cfg(feature = "tar")]
            File::Tar(ref mut file) => file.seek(pos),
            File::User(ref mut file) => file.seek(pos),
        }
    }
}

/// File or directory entry.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entry {
    pub file_name: OsString,
    pub path: PathBuf,
    pub directory: bool,
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
            Err(io::Error::from(io::ErrorKind::NotFound))
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
        let store = Box::new(store::MapFile::new(store, |file: T| file.into()));
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

/// In-memory file.
pub struct RamFile(io::Cursor<Rc<[u8]>>);

impl io::Read for RamFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl io::Seek for RamFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}

impl Store for Ram {
    type File = RamFile;

    fn open_path(&self, path: &Path) -> io::Result<Self::File> {
        match self.inner.get(path) {
            Some(file) => Ok(RamFile(io::Cursor::new(Rc::clone(file)))),
            None => Err(io::Error::from(io::ErrorKind::NotFound)),
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
