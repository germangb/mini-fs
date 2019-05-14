use std::collections::btree_set::BTreeSet;
use std::ffi::OsString;
use std::io::{self, Read, Seek, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// File or directory entry.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entry {
    pub name: OsString,
    pub kind: EntryKind,
}

/// Type of file entry.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntryKind {
    File,
    Dir,
    /*TODO symlinks
     *Sym, */
}

/// Iterator of file entries.
pub struct Entries<'a> {
    inner: Box<dyn Iterator<Item = io::Result<Entry>> + 'a>,
}

impl<'a> Entries<'a> {
    pub(crate) fn empty() -> Self {
        Self::new(std::iter::empty())
    }

    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = <Entries<'a> as Iterator>::Item>,
        <I as IntoIterator>::IntoIter: 'a,
    {
        Self {
            inner: Box::new(iter.into_iter()),
        }
    }
}

impl Iterator for Entries<'_> {
    type Item = io::Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
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
    fn entries_path(&self, path: &Path) -> io::Result<Entries> {
        Ok(Entries::empty())
    }
}

/// Convenient methods on top of Store.
pub trait StoreExt: Store {
    fn entries<P: AsRef<Path>>(&self, path: P) -> io::Result<Entries> {
        <Self as Store>::entries_path(self, &crate::index::normalize_path(path.as_ref()))
    }

    fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<Self::File> {
        <Self as Store>::open_path(self, &crate::index::normalize_path(path.as_ref()))
    }
}

impl<T: Store> StoreExt for T {}

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

    #[inline]
    fn entries_path(&self, path: &Path) -> io::Result<Entries> {
        self.store.entries_path(path)
    }
}

// iterator + set to take care of repeating elements.
// TODO consider other data structures for the set.
struct TupleEntries<I> {
    inner: I,
    set: BTreeSet<OsString>,
}

impl<I> TupleEntries<I> {
    fn new(inner: I) -> Self {
        Self {
            inner,
            set: BTreeSet::new(),
        }
    }
}

impl<I> Iterator for TupleEntries<I>
where
    I: Iterator<Item = io::Result<Entry>>,
{
    type Item = <I as Iterator>::Item;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                None => return None,
                Some(Err(e)) => return Some(Err(e)),
                Some(Ok(file)) => {
                    if self.set.insert(file.name.clone()) {
                        return Some(Ok(file));
                    }
                }
            }
        }
    }
}

macro_rules! entries {
    ($self:ident, $path:expr, $head:ident,) => { $self.0.entries_path($path)? };
    ($self:ident, $path:expr, $head:ident, $($tail:ident,)+) => {
        $self.0.entries_path($path)?.chain(entries!($self, $path, $($tail,)+) )
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
                match $head.open_path(path) {
                    Ok(file) => return Ok(file.into()),
                    Err(ref err) if err.kind() == io::ErrorKind::NotFound => {},
                    Err(err) => return Err(err),
                }
                $(
                match $tail.open_path(path) {
                    Ok(file) => return Ok(file.into()),
                    Err(ref err) if err.kind() == io::ErrorKind::NotFound => {},
                    Err(err) => return Err(err),
                }
                )+

                Err(io::Error::from(io::ErrorKind::NotFound))
            }

            fn entries_path(&self, path: &Path) -> io::Result<Entries> {
                // chain all elements from the tuple
                let raw = entries!(self, path, $head, $($tail,)+);
                Ok(Entries::new(TupleEntries::new(raw)))
            }
        }
        tuples!($($tail,)+);
    };
}

// Implement tuples of up to 11 elements (12 or more looks bad on the rustdoc)
tuples! { A, B, C, D, E, F, G, H, I, J, K, }
