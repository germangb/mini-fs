use std::collections::LinkedList;
use std::error::Error;
use std::marker::PhantomData;
use std::path::Path;

/// Concrete file type
pub mod file;
/// File storage generics.
pub mod store;
/// Zip file storage.
#[cfg(feature = "zip")]
pub mod zip {}
/// Tar file storage.
#[cfg(feature = "tar")]
pub mod tar {}

pub use crate::err;

use store::Store;

/// Virtual filesystem.
pub struct MiniFs<F> {
    mount: LinkedList<Box<dyn Store<File = F, Error = Box<dyn Error>>>>,
}

impl<F> MiniFs<F> {
    pub fn mount<E, S>(mut self, store: S) -> Self
    where
        E: Into<Box<dyn Error>>,
        S: Store<File = F, Error = E>,
    {
        self
    }
}
