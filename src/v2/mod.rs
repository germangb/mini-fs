use std::collections::{BTreeMap, LinkedList};
use std::env;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use store::Store;

/// Concrete file type
pub mod file;
/// File storage generic.
pub mod store;
/// Zip file storage.
#[cfg(feature = "zip")]
pub mod zip {}
/// Tar file storage.
#[cfg(feature = "tar")]
pub mod tar;

struct Mount<F> {
    path: PathBuf,
    store: Box<dyn Store<File = F>>,
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

    fn open(&self, path: &Path) -> io::Result<file::File> {
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

    fn open(&self, path: &Path) -> io::Result<file::File> {
        match self.inner.get(path) {
            Some(file) => Ok(file::File::from_ram(Rc::clone(file))),
            None => Err(Error::new(ErrorKind::NotFound, "File not found.")),
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

macro_rules! tuples {
    ($head:ident,) => {};
    ($head:ident, $($tail:ident,)+) => {
        impl<$head, $($tail,)+> Store for ($head, $($tail,)+)
        where
            $head: Store,
            $($tail: Store<File = $head::File>,)+
        {
            type File = $head::File;
            #[allow(non_snake_case)]
            fn open(&self, path: &Path) -> io::Result<Self::File> {
                let ($head, $($tail,)+) = self;
                match $head.open(path) {
                    Ok(file) => return Ok(file),
                    Err(ref err) if err.kind() == ErrorKind::NotFound => {},
                    Err(err) => return Err(err),
                }
                $(
                match $tail.open(path) {
                    Ok(file) => return Ok(file),
                    Err(ref err) if err.kind() == ErrorKind::NotFound => {},
                    Err(err) => return Err(err),
                }
                )+

                Err(Error::new(ErrorKind::NotFound, "File not found."))
            }
        }
        tuples!($($tail,)+);
    };
}

// implement for tuples of up to size 8
tuples! { A, B, C, D, E, /*F,*/ G, H, I, }
