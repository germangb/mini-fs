use std::io::{self, Read, Seek, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// File or directory entry.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entry {
    pub path: PathBuf,
    pub entry_type: EntryKind,
}

/// Type of file entry.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntryKind {
    File,
    Directory,
}

/// Iterator of file entries.
pub struct Entries<'a> {
    inner: Box<dyn Iterator<Item = io::Result<Entry>>>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Entries<'a> {
    fn empty() -> Self {
        Self::new(std::iter::empty())
    }

    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = <Entries<'a> as Iterator>::Item>,
        <I as IntoIterator>::IntoIter: 'static,
    {
        Self {
            inner: Box::new(iter.into_iter()),
            _phantom: PhantomData,
        }
    }
}

impl Iterator for Entries<'_> {
    type Item = io::Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

/// Generic file storage.
pub trait Store {
    type File;

    fn open_path(&self, path: &Path) -> io::Result<Self::File>;

    /// Iterate over the entries of the Store.
    ///
    /// Order is not defined, so it may be depth first, breadth first, or any
    /// arbitrary order. The provided implementation returns an empty
    /// iterator.
    fn entries(&self) -> Entries {
        Entries::empty()
    }

    // The methods below aren't part of the trait object.

    fn open<P>(&self, path: P) -> io::Result<Self::File>
    where
        P: AsRef<Path>,
        Self: Sized,
    {
        self.open_path(path.as_ref())
    }
}

pub(crate) struct MapFile<S, F> {
    store: S,
    clo: F,
}

impl<S, F> MapFile<S, F> {
    pub(crate) fn new(store: S, closure: F) -> Self {
        Self {
            store,
            clo: closure,
        }
    }
}

impl<U, S, F> Store for MapFile<S, F>
where
    S: Store,
    F: Fn(S::File) -> U,
{
    type File = U;
    #[inline]
    fn open_path(&self, path: &Path) -> io::Result<Self::File> {
        match self.store.open_path(path) {
            Ok(file) => Ok((self.clo)(file)),
            Err(err) => Err(err),
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
            $head::File: Into<$crate::File>,
            $($tail::File: Into<$crate::File>,)+
        {
            type File = $crate::File;
            #[allow(non_snake_case)]
            fn open_path(&self, path: &Path) -> io::Result<Self::File> {
                let ($head, $($tail,)+) = self;
                match $head.open(path) {
                    Ok(file) => return Ok(file.into()),
                    Err(ref err) if err.kind() == io::ErrorKind::NotFound => {},
                    Err(err) => return Err(err),
                }
                $(
                match $tail.open(path) {
                    Ok(file) => return Ok(file.into()),
                    Err(ref err) if err.kind() == io::ErrorKind::NotFound => {},
                    Err(err) => return Err(err),
                }
                )+

                Err(io::Error::from(io::ErrorKind::NotFound))
            }
        }
        tuples!($($tail,)+);
    };
}

tuples! { A, B, C, D, E, F, G, H, I, J, K, }
