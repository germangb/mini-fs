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
//! Paths defined by the virtual filesystem are **case sensitive**.
use std::collections::{BTreeMap, LinkedList};
use std::env;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub use file::File;
#[cfg(feature = "s3")]
pub use s3::S3;
pub use store::Store;
#[cfg(feature = "tar")]
pub use tar::Tar;
#[cfg(feature = "zip")]
pub use zip::Zip;

/// Concrete file type
mod file;
/// S3 bucket from Aws.
#[cfg(feature = "s3")]
pub mod s3;
/// File storage generic.
pub mod store;
/// Tar file storage.
#[cfg(feature = "tar")]
pub mod tar;
/// Zip file storage.
#[cfg(feature = "zip")]
pub mod zip;

struct Mount<F> {
    path: PathBuf,
    store: Box<dyn Store<File = F>>,
}

/// Virtual filesystem.
pub struct MiniFs<F> {
    mount: LinkedList<Mount<F>>,
}

impl<F> Store for MiniFs<F> {
    type File = F;

    fn open_path(&self, path: &Path) -> io::Result<F> {
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

impl<F> MiniFs<F> {
    pub fn new() -> Self {
        Self {
            mount: LinkedList::new(),
        }
    }

    pub fn mount<P, S>(mut self, path: P, store: S) -> Self
    where
        P: Into<PathBuf>,
        S: Store<File = F> + 'static,
    {
        let path = path.into();
        let store = Box::new(store);
        self.mount.push_back(Mount { path, store });
        self
    }

    pub fn umount<P>(&mut self, path: P) -> Option<Box<dyn Store<File = F>>>
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
    type File = file::File;

    fn open_path(&self, path: &Path) -> io::Result<file::File> {
        let file = fs::File::open(self.root.join(path))?;
        Ok(file::File::from_std(file))
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
    type File = file::File;

    fn open_path(&self, path: &Path) -> io::Result<file::File> {
        match self.inner.get(path) {
            Some(file) => Ok(file::File::from_ram(Rc::clone(file))),
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
