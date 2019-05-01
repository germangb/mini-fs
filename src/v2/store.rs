use std::io::{self, ErrorKind};
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
///
/// See [`Store::map_file`](trait.Store.html#method.map_file).
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
                    Err(ref err) if err.kind() == io::ErrorKind::NotFound => {},
                    Err(err) => return Err(err),
                }
                $(
                match $tail.open(path) {
                    Ok(file) => return Ok(file),
                    Err(ref err) if err.kind() == io::ErrorKind::NotFound => {},
                    Err(err) => return Err(err),
                }
                )+

                Err(io::Error::new(io::ErrorKind::NotFound, "File not found."))
            }
        }
        tuples!($($tail,)+);
    };
}

// implement for tuples of up to size 8
tuples! { A, B, C, D, E, F, G, H, }
