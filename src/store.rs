use std::io::{self, ErrorKind};
use std::path::Path;

/// Generic file storage.
pub trait Store {
    type File;
    fn open(&self, path: &Path) -> io::Result<Self::File>;

    fn open_read(&self, path: &Path) -> io::Result<Self::File> {
        self.open(path)
    }

    fn open2<P>(&self, path: P) -> io::Result<Self::File>
    where
        P: AsRef<Path>,
        Self: Sized,
    {
        self.open(path.as_ref())
    }

    fn open_read2<P>(&self, path: P) -> io::Result<Self::File>
    where
        P: AsRef<Path>,
        Self: Sized,
    {
        self.open_read(path.as_ref())
    }

    fn map_file<F, U>(self, f: F) -> MapFile<Self, F>
    where
        F: Fn(Self::File) -> U,
        Self: Sized,
    {
        MapFile::new(self, f)
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

                Err(io::Error::from(io::ErrorKind::NotFound))
            }
        }
        tuples!($($tail,)+);
    };
}

// implement for tuples of up to size 8
tuples! { A, B, C, D, E, F, G, H, }
