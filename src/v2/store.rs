use std::marker::PhantomData;
use std::path::Path;

/// Generic file storage.
pub trait Store {
    type File;
    type Error;
    fn open(&self, path: &Path) -> Result<Self::File, Self::Error>;

    fn map_file<F, U>(self, f: F) -> MapFile<Self, F, U>
    where
        F: Fn(Self::File) -> U,
        Self: Sized,
    {
        MapFile {
            store: self,
            closure: f,
            _file: PhantomData,
        }
    }

    fn map_err<F, E>(self, f: F) -> MapErr<Self, F, E>
    where
        F: Fn(Self::Error) -> E,
        Self: Sized,
    {
        MapErr {
            store: self,
            closure: f,
            _error: PhantomData,
        }
    }
}

/// File type adapter.
pub struct MapFile<S, F, U> {
    store: S,
    closure: F,
    _file: PhantomData<U>,
}

impl<S, F, U> Store for MapFile<S, F, U>
where
    S: Store,
    F: Fn(S::File) -> U,
{
    type File = U;
    type Error = S::Error;
    #[inline]
    fn open(&self, path: &Path) -> Result<Self::File, Self::Error> {
        match self.store.open(path) {
            Ok(file) => Ok((self.closure)(file)),
            Err(err) => Err(err),
        }
    }
}

/// Error type adapter.
pub struct MapErr<S, F, E> {
    store: S,
    closure: F,
    _error: PhantomData<E>,
}

impl<S, F, E> Store for MapErr<S, F, E>
where
    S: Store,
    F: Fn(S::Error) -> E,
{
    type File = S::File;
    type Error = E;

    fn open(&self, path: &Path) -> Result<Self::File, Self::Error> {
        match self.store.open(path) {
            Ok(file) => Ok(file),
            Err(err) => Err((self.closure)(err)),
        }
    }
}
