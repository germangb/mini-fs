use std::io::{self, ErrorKind, Read, Seek, Write};
use std::path::Path;

pub trait StoreFile: Read + Write + Seek {}

/// Generic file storage.
pub trait Store {
    type File;
    fn open_path(&self, path: &Path) -> io::Result<Self::File>;

    fn open_write_path(&self, path: &Path) -> io::Result<Self::File> {
        self.open_path(path)
    }

    fn open<P>(&self, path: P) -> io::Result<Self::File>
    where
        P: AsRef<Path>,
        Self: Sized,
    {
        self.open_path(path.as_ref())
    }

    fn open_write<P>(&self, path: P) -> io::Result<Self::File>
    where
        P: AsRef<Path>,
        Self: Sized,
    {
        self.open_write_path(path.as_ref())
    }

    fn map_file<F, U>(self, f: F) -> MapFile<Self, F>
    where
        F: Fn(Self::File) -> U,
        Self: Sized,
    {
        MapFile::new(self, f)
    }
}

impl<T: Store> Store for Box<T> {
    type File = T::File;

    fn open_path(&self, path: &Path) -> io::Result<Self::File> {
        unimplemented!()
    }
}

/// Maps file type using a closure.
///
/// See [`Store::map_file`](trait.Store.html#method.map_file).
pub struct MapFile<S, F> {
    store: S,
    clo: F,
}

impl<S, F> MapFile<S, F> {
    fn new(store: S, closure: F) -> Self {
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

// implement for tuples of up to size 8
tuples! { A, B, C, D, E, F, G, H, }
