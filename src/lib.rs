//! **mini-fs** is a filesystem abstraction layer.
//!
//! Currently supported features include:
//!
//! - Access to the local (native) filesystem.
//! - In-memory filesystems.
//! - Tar, tar.gz, and Zip archives.
//! - Logically merging filesystems (useful for overriding some files)
//!
//! ## Merging example
//!
//! This feature is useful when you want to override some files from another
//! location:
//!
//! ```no_run
//! # fn main() -> Result<(), mini_fs::err::Error> {
//! use std::path::Path;
//! # use mini_fs::{Store, Local, Tar, MiniFs, err::Error};
//! let a = Local::new("data/");
//! // |- example.txt
//!
//! let b = Tar::open("archive.tar.gz")?;
//! // |- example.txt
//! // |- hello.txt
//!
//! let files = MiniFs::new().mount("/files", (a, b));
//!
//! assert!(files.open(Path::new("/files/example.txt")).is_ok()); // this "example.txt" is from "a"
//! assert!(files.open(Path::new("/files/hello.txt")).is_ok());
//! # Ok(())
//! # }
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

/// Error and result types.
pub mod err;
mod file;
/// Storage from a tarball.
///
/// *To use this module you must enable the "tar" feature.*
#[cfg(feature = "tar")]
pub mod tar;
/// v2 api work in progress.
///
/// *To use this module you must enable the "point_two" feature*
#[cfg(feature = "point_two")]
pub mod v2;
/// Storage from a Zip file.
///
/// *To use this module you must enable the "zip" feature.*
#[cfg(feature = "zip")]
pub mod zip;

/// Generic filesystem abstraction.
pub trait Store {
    /// Opens file in read-only mode.
    fn open(&self, path: &Path) -> Result<File>;

    /// Opens a file in write-only mode.
    #[allow(unused_variables)]
    fn open_write(&self, path: &Path) -> Result<File> {
        Err(Error::Io(file::write_support_err()))
    }
}

/// Local (native) filesystem.
pub struct Local {
    root: PathBuf,
}

impl Store for Local {
    fn open(&self, path: &Path) -> Result<File> {
        let file = fs::File::open(self.root.join(path))?;
        Ok(File::from_fs(file))
    }

    fn open_write(&self, path: &Path) -> Result<File> {
        let file = fs::OpenOptions::new()
            .write(true)
            // FileMut doesn't impl Read
            .read(false)
            .create(false)
            .open(path)?;

        Ok(File::from_fs(file))
    }
}

impl Local {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    /// Point to the current working directory.
    pub fn pwd() -> Result<Self> {
        Ok(Self::new(env::current_dir()?))
    }
}

/// In-memory file store.
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

/// Virtual filesystem.
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
