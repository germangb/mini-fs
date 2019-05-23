//! This module contains a caseless filesystem.

use std::ffi::OsString;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::index::normalize_path;
use crate::prelude::*;
use crate::store::Entries;

/// Caseless filesystem wrapping an inner filesystem.
#[derive(Clone, Debug)]
pub struct CaselessFs<T> {
    /// Inner filesystem store.
    inner: T,
}

impl<T: Store> CaselessFs<T> {
    /// Creates a new caseless filesystem with the provided inner filesystem.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Moves the inner filesystem out of the caseless filesystem.
    /// Inspired by std::io::Cursor.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Gets a reference to the inner filesystem.
    /// Inspired by std::io::Cursor.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the inner filesystem.
    /// Inspired by std::io::Cursor.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Finds paths that match the caseless path.
    pub fn find<P: AsRef<Path>>(&self, path: P) -> Vec<PathBuf> {
        let path = normalize_path(path.as_ref());
        let mut paths = vec![PathBuf::new()];
        for component in path.components() {
            paths = find_next_ascii_lowercase(&self.inner, &component, paths);
            if paths.len() == 0 {
                return paths;
            }
        }
        paths
    }
}

impl<T: Store> Store for CaselessFs<T> {
    type File = T::File;

    /// Opens the file of the caseless path.
    fn open_path(&self, path: &Path) -> io::Result<Self::File> {
        // real path
        if let Ok(file) = self.inner.open_path(path) {
            return Ok(file);
        }
        // caseless path
        for path in self.find(path) {
            return self.inner.open_path(&path);
        }
        Err(io::ErrorKind::NotFound.into())
    }

    /// Iterates over the entries of the inner filesystem.
    fn entries_path(&self, path: &Path) -> io::Result<Entries> {
        self.inner.entries_path(path)
    }
}

/// Finds the next path candidates.
fn find_next_ascii_lowercase<S: Store>(
    fs: &S,
    component: &Component,
    paths: Vec<PathBuf>,
) -> Vec<PathBuf> {
    let mut next = Vec::new();
    let target: OsString = match component {
        Component::Normal(os_s) => (*os_s).to_owned(),
        Component::RootDir => {
            // nothing can go before the root
            next.push(Path::new("/").to_owned());
            return next;
        }
        _ => {
            panic!(format!("unexpected path component {:?}", component));
        }
    };
    if let Some(t_s) = target.to_str() {
        // compare utf8
        for path in paths {
            if let Ok(entries) = fs.entries(&path) {
                for e in entries {
                    if let Ok(entry) = e {
                        if let Some(e_s) = entry.name.to_str() {
                            if t_s.to_ascii_lowercase() == e_s.to_ascii_lowercase() {
                                let mut path = path.to_owned();
                                path.push(&entry.name);
                                next.push(path);
                            }
                        }
                    }
                }
            }
        }
    } else {
        // compare raw
        for path in paths {
            if let Ok(entries) = fs.entries(&path) {
                for e in entries {
                    if let Ok(entry) = e {
                        if &entry.name == &target {
                            let mut path = path.to_owned();
                            path.push(&entry.name);
                            next.push(path);
                        }
                    }
                }
            }
        }
    }
    next
}
