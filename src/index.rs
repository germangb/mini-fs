use crate::File;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

pub type Files<M> = BTreeMap<OsString, Node<M>>;
pub enum Node<M> {
    Empty,
    File { name: OsString, meta: M },
    Dir { name: OsString, ent: Files<M> },
}

/// Directory index.
pub struct DirIndex<M> {
    root: Node<M>,
}

impl<M> DirIndex<M> {
    pub fn new() -> Self {
        Self { root: Node::Empty }
    }

    pub fn insert<P>(&mut self, path: P, meta: M)
    where
        P: Into<PathBuf>,
    {
        unimplemented!()
    }

    pub fn get<P>(&self, path: P) -> Option<&M>
    where
        P: AsRef<Path>,
    {
        let path = normalize_path(path.as_ref());
        unimplemented!()
    }
}

// TODO errors
fn normalize_path(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                normal.pop();
            }
            Component::Normal(n) => normal.push(n),
            _ => unimplemented!(),
        }
    }
    normal
}
