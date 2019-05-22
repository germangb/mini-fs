use std::collections::btree_set::BTreeSet;
use std::ffi::OsString;
use std::io;
use std::path::Path;

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
    /// arbitrary order.
    fn entries_path(&self, _: &Path) -> io::Result<Entries> {
        unimplemented!("entries_path is not implemented.")
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

// Implement tuples of up to 11 elements (12 or more looks bad on the rustdoc)
store_tuples! { A, B, C, D, E, F, G, H, I, J, K, }
