use std::collections::{BTreeMap, LinkedList};
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// Concrete file type
pub mod file;
/// File storage generic.
pub mod store;
/// Zip file storage.
#[cfg(feature = "zip")]
pub mod zip {}
/// Tar file storage.
#[cfg(feature = "tar")]
pub mod tar {}
/// Concrete error type.
pub mod err;

use self::store::Store;

struct Mount<F> {
    path: PathBuf,
    store: Box<dyn Store<File = F, Error = Box<dyn Error>>>,
}

/// Virtual filesystem.
pub struct MiniFs<F> {
    mount: LinkedList<Mount<F>>,
}

impl<F> MiniFs<F> {
    pub fn new() -> Self {
        Self {
            mount: LinkedList::new(),
        }
    }

    pub fn mount<P, E, S>(mut self, path: P, store: S) -> Self
    where
        P: Into<PathBuf>,
        S: Store<File = F, Error = E> + 'static,
        E: Into<Box<dyn Error>>,
    {
        let path = path.into();
        let store = Box::new(store.map_err(|e| e.into()));
        self.mount.push_back(Mount { path, store });
        self
    }

    pub fn umount<P>(&mut self, path: P) -> Option<Box<dyn Store<File = F, Error = Box<dyn Error>>>>
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
    type Error = err::Error;

    fn open(&self, path: &Path) -> Result<Self::File, Self::Error> {
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
    inner: BTreeMap<PathBuf, Vec<u8>>,
}

macro_rules! tuples {
    ($head:ident,) => {};
    ($head:ident, $($tail:ident,)+) => {
        impl<$head, $($tail,)+> Store for ($head, $($tail,)+)
        where
            $head: Store,
            $($tail: Store<File = $head::File, Error = $head::Error>,)+
        {
            type File = $head::File;
            type Error = $head::Error;
            #[allow(non_snake_case)]
            fn open(&self, path: &Path) -> Result<Self::File, Self::Error> {
                let ($head, $($tail,)+) = self;
                match $head.open(path) {
                    Ok(file) => return Ok(file),
                    Err(err) => return Err(err),
                }
                $(
                match $tail.open(path) {
                    Ok(file) => return Ok(file),
                    Err(err) => return Err(err),
                }
                )+

                // TODO Result<Option<File>, Error> where None is FileNotFound
                unimplemented!();
            }
        }
        tuples!($($tail,)+);
    };
}

// implement for tuples of up to size 8
tuples! { A, B, C, D, E, /*F,*/ G, H, I, }
