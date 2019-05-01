use std::io;
use std::marker::PhantomData;
use std::path::Path;

/// Generic file storage.
pub trait Store {
    type File;
    fn open(&self, path: &Path) -> io::Result<Self::File>;

    fn map_file<F, U>(self, f: F) -> MapFile<Self, F, U>
    where
        F: Fn(Self::File) -> U,
        Self: Sized,
    {
        MapFile::new(self, f)
    }
}

/// File type adapter.
pub struct MapFile<S, F, U> {
    store: S,
    clo: F,
    _phantom: PhantomData<U>,
}

impl<S, F, U> MapFile<S, F, U> {
    fn new(store: S, closure: F) -> Self {
        Self {
            store,
            clo: closure,
            _phantom: PhantomData,
        }
    }
}

impl<S, F, U> Store for MapFile<S, F, U>
where
    S: Store,
    F: Fn(S::File) -> U,
{
    type File = U;

    #[inline]
    fn open(&self, path: &Path) -> io::Result<Self::File> {
        match self.store.open(path) {
            Ok(file) => Ok((self.clo)(file)),
            Err(err) => Err(err),
        }
    }
}
