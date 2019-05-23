//! `mini-fs` is an extensible virtual filesystem for the application layer.
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
//! All implementations of [`Store`] from this crate use **case sensitive**¹
//! paths. However, you are free to implement custom stores where paths are case
//! insensitive.
//!
//! ¹ Except maybe [`LocalFs`], which uses [`std::fs`] internally and is subject
//! to the underlying OS.
//!
//! ## Example
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use mini_fs::prelude::*;
//! use mini_fs::{LocalFs, MiniFs, ZipFs};
//!
//! let gfx = LocalFs::new("./res/images");
//! let sfx = ZipFs::open("archive.zip")?;
//!
//! let assets = MiniFs::new().mount("/gfx", gfx).mount("/sfx", sfx);
//!
//! let root = MiniFs::new().mount("/assets", assets);
//!
//! let file = root.open("/assets/gfx/trash.gif")?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Security
//!
//! Don't use this crate in applications where security is a critical factor.
//! [`LocalFs`] in particular might be vulnerable to [directory traversal
//! attacks][dir], so it's best not to use it directly in a static file server,
//! for example.
//!
//! [`std::fs`]: https://doc.rust-lang.org/std/fs/index.html
//! [`Store`]: ./trait.Store.html
//! [`LocalFs`]: ./struct.LocalFs.html
//! [dir]: https://en.wikipedia.org/wiki/Directory_traversal_attack
#![deny(warnings)]
use std::any::Any;
use std::collections::LinkedList;
use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{env, fs};

pub use caseless::CaselessFs;
//pub use index::{Index, IndexEntries};
pub use store::{Entries, Entry, EntryKind, Store, StoreExt};
#[cfg(feature = "tar")]
pub use tar::TarFs;
#[cfg(feature = "zip")]
pub use zip::ZipFs;

include!("macros.rs");

pub mod caseless;
/// Directory index.
#[doc(hidden)]
pub mod index;
mod store;
/// Tar file storage.
#[cfg(feature = "tar")]
pub mod tar;
/// Zip file storage.
#[cfg(feature = "zip")]
pub mod zip;
/// Convenient library imports.
pub mod prelude {
    pub use crate::store::{Store, StoreExt};
}

impl_file! {
    /// File you can seek and read from.
    pub enum File {
        Local(fs::File),
        Ram(RamFile),
        #[cfg(feature = "zip")]
        Zip(zip::ZipFsFile),
        #[cfg(feature = "tar")]
        Tar(tar::TarFsFile),
        // External types are dynamic
        User(Box<dyn UserFile>),
    }
}

/// Custom file type.
pub trait UserFile: Any + Read + Seek + Send {}

impl<T: UserFile> From<T> for File {
    fn from(file: T) -> Self {
        File::User(Box::new(file))
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

    fn open_path(&self, path: &Path) -> Result<File> {
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

    fn entries_path(&self, path: &Path) -> Result<Entries> {
        // FIXME creating a new PathBuf because otherwise I'm getting lifetime errors.
        let path = path.to_path_buf();

        Ok(Entries::new(
            self.mount
                .iter()
                .rev()
                .find(|m| path.strip_prefix(&m.path).is_ok())
                .into_iter()
                .flat_map(move |m| match path.strip_prefix(&m.path) {
                    Ok(np) => m.store.entries_path(np).unwrap(),
                    Err(_) => Entries::new(None),
                }),
        ))
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
pub struct LocalFs {
    root: PathBuf,
}

impl Store for LocalFs {
    type File = fs::File;

    fn open_path(&self, path: &Path) -> Result<fs::File> {
        fs::OpenOptions::new()
            .create(false)
            .read(true)
            .write(false)
            .open(self.root.join(path))
    }

    fn entries_path(&self, path: &Path) -> Result<Entries> {
        // FIXME cloned because lifetimes.
        //let root = self.root.clone();

        let entries = fs::read_dir(self.root.join(path))?.map(move |ent| {
            let entry = ent?;
            let path = entry
                .path()
                .strip_prefix(&self.root)
                .map(Path::to_path_buf)
                .expect("Error striping path suffix.");
            let file_type = entry.file_type()?;

            // TODO synlinks
            let kind = if file_type.is_dir() {
                EntryKind::Dir
            } else if file_type.is_symlink() {
                EntryKind::File
            } else {
                EntryKind::File
            };

            Ok(Entry {
                name: path.into_os_string(),
                kind,
            })
        });

        Ok(Entries::new(entries))
    }
}

impl LocalFs {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    /// Point to the current working directory.
    pub fn pwd() -> Result<Self> {
        Ok(Self::new(env::current_dir()?))
    }
}

/// In-memory file storage
pub struct RamFs {
    index: index::Index<Rc<[u8]>>,
}

/// In-memory file.
pub struct RamFile(Cursor<Rc<[u8]>>);

impl Read for RamFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl Seek for RamFile {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.0.seek(pos)
    }
}

impl Store for RamFs {
    type File = RamFile;

    fn open_path(&self, path: &Path) -> Result<Self::File> {
        match self.index.get(path) {
            Some(file) => Ok(RamFile(Cursor::new(Rc::clone(file)))),
            None => Err(Error::from(ErrorKind::NotFound)),
        }
    }

    fn entries_path(&self, path: &Path) -> Result<Entries> {
        Ok(Entries::new(self.index.entries(path).map(|ent| {
            Ok(Entry {
                name: ent.name.to_os_string(),
                kind: ent.kind,
            })
        })))
    }
}

impl RamFs {
    pub fn new() -> Self {
        Self {
            index: index::Index::new(),
        }
    }

    pub fn clear(&mut self) {
        self.index.clear();
    }

    pub fn rm<P: AsRef<Path>>(&mut self, path: P) -> Option<Rc<[u8]>> {
        self.index.remove(path)
    }

    pub fn touch<P, F>(&mut self, path: P, file: F)
    where
        P: Into<PathBuf>,
        F: Into<Rc<[u8]>>,
    {
        self.index.insert(path.into(), file.into());
    }

    pub fn index(self) -> Self {
        self
    }
}
