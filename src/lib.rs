//! Stupid simple filesystem-like storage.
//!
//! Supports reading from the local filesystem and from in-memory tar and zip
//! archives.
//!
//! # Example
//!
//! ```
//! use mini_fs::{Local, MiniFs};
//!
//! let a = Local::new("/core/res");
//! let b = Local::new("/user/res");
//!
//! // You can use tuples to merge stores.
//! let res = (b, a);
//!
//! let files = MiniFs::new().mount("/res", res);
//! ```
use std::collections::BTreeMap;
use std::collections::LinkedList;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use err::{Error, Result};
pub use file::File;
#[cfg(feature = "tar")]
pub use tar::Tar;
#[cfg(feature = "zip")]
pub use zip::Zip;

/// Error types.
pub mod err;
mod file;
/// Storage from a tarball.
///
/// *To use this module you must enable the "tar" feature.*
#[cfg(feature = "tar")]
pub mod tar;
/// Storage from a Zip file.
///
/// *To use this module you must enable the "zip" feature.*
#[cfg(feature = "zip")]
pub mod zip;

/// Generic filesystem abstraction.
pub trait Store {
    fn open(&self, path: &Path) -> Result<File>;
}

/// Local filesystem store.
pub struct Local {
    root: PathBuf,
}

impl Store for Local {
    fn open(&self, path: &Path) -> Result<File> {
        let file = fs::File::open(self.root.join(path))?;
        Ok(File::from_fs(file))
    }
}

impl Local {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    pub fn pwd() -> Result<Self> {
        Ok(Self::new(env::current_dir()?))
    }
}

/// In-memory data store.
#[derive(Clone)]
pub struct Ram {
    inner: BTreeMap<PathBuf, Vec<u8>>,
}

impl Store for Ram {
    fn open(&self, path: &Path) -> Result<File> {
        self.inner
            .get(path)
            .map(|b| File::from_ram(b))
            .ok_or_else(|| Error::FileNotFound)
    }
}

impl Ram {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn touch<P, F>(&mut self, path: P, file: F)
    where
        P: Into<PathBuf>,
        F: Into<Vec<u8>>,
    {
        self.inner.insert(path.into(), file.into());
    }
}

struct Mount {
    path: PathBuf,
    store: Box<dyn Store>,
}

/// Filesystem-like data storage.
pub struct MiniFs {
    inner: LinkedList<Mount>,
}

impl MiniFs {
    pub fn new() -> Self {
        Self {
            inner: LinkedList::new(),
        }
    }

    pub fn mount<P, F>(mut self, path: P, store: F) -> Self
    where
        P: Into<PathBuf>,
        F: Store + Send + 'static,
    {
        let path = path.into();
        let store = Box::new(store);
        self.inner.push_back(Mount { path, store });
        self
    }

    /// Unmounts store mounted at the given location.
    ///
    /// Returns the unmounted store if the given path was a valid mounting
    /// point. Returns `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// # use mini_fs::{Local, MiniFs};
    /// let a = Local::new("/");
    /// let b = Local::new("/etc");
    ///
    /// let mut fs = MiniFs::new().mount("/", a).mount("/etc", b);
    ///
    /// assert!(fs.umount("/etc").is_some());
    /// assert!(fs.umount("/etc").is_none());
    /// ```
    pub fn umount<P: AsRef<Path>>(&mut self, path: P) -> Option<Box<dyn Store>> {
        let path = path.as_ref();
        if let Some(p) = self.inner.iter().rposition(|p| p.path == path) {
            let mut tail = self.inner.split_off(p);
            let fs = tail.pop_front().map(|m| m.store);
            self.inner.append(&mut tail);
            fs
        } else {
            None
        }
    }
}

impl Store for MiniFs {
    fn open(&self, path: &Path) -> Result<File> {
        let next = self.inner.iter().rev().find_map(|mnt| {
            if let Ok(np) = path.strip_prefix(&mnt.path) {
                Some((np, &mnt.store))
            } else {
                None
            }
        });
        if let Some((np, store)) = next {
            store.open(np)
        } else {
            Err(Error::FileNotFound)
        }
    }
}

macro_rules! tuples {
    ($head:ident,) => {};
    ($head:ident, $($tail:ident,)+) => {
        impl<$head, $($tail,)+> Store for ($head, $($tail,)+)
        where
            $head: Store,
            $($tail: Store,)+
        {
            #[allow(non_snake_case)]
            fn open(&self, path: &Path) -> Result<File> {
                let ($head, $($tail,)+) = self;
                match $head.open(path) {
                    Ok(file) => return Ok(file),
                    Err(Error::FileNotFound) => {}
                    Err(err) => return Err(err),
                }
                $(
                match $tail.open(path) {
                    Ok(file) => return Ok(file),
                    Err(Error::FileNotFound) => {}
                    Err(err) => return Err(err),
                }
                )+
                Err(Error::FileNotFound)
            }
        }
        tuples!($($tail,)+);
    };
}

// implement for tuples of up to size 8
tuples! { A, B, C, D, E, F, G, H, }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
