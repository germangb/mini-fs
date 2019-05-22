use crate::EntryKind;

use std::borrow::Cow;
use std::collections::btree_map::{BTreeMap, Iter};
use std::collections::vec_deque::VecDeque;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// Directory tree node.
/// Contains a list of file entries (leaf nodes), and directories (child nodes).
struct Node<M> {
    files: BTreeMap<OsString, M>,
    dirs: BTreeMap<OsString, Node<M>>,
}
impl<M> Node<M> {
    fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            dirs: BTreeMap::new(),
        }
    }
}

/// Directory index, implemented as a trie.
pub struct Index<M> {
    // TODO optimize tree (merge consecutive nodes with a single directory entry)
    // TODO remove extra unnecessary allocations.
    root: Node<M>,
}

/// Index entries iterator.
pub struct Entries<'a, M> {
    files: Option<Iter<'a, OsString, M>>,
    dirs: Option<Iter<'a, OsString, Node<M>>>,
}

impl<'a, M> Entries<'a, M> {
    // Polls the next file entry
    fn next_file(&mut self) -> Option<<Self as Iterator>::Item> {
        if let Some(ref mut iter) = self.files {
            iter.next().map(|(n, e)| Entry {
                name: n.as_os_str(),
                meta: Some(e),
                kind: EntryKind::File,
            })
        } else {
            None
        }
    }

    // Polls the next directory entry
    fn next_dir(&mut self) -> Option<<Self as Iterator>::Item> {
        if let Some(ref mut iter) = self.dirs {
            iter.next().map(|(n, _)| Entry {
                name: n.as_os_str(),
                meta: None,
                kind: EntryKind::File,
            })
        } else {
            None
        }
    }
}

impl<'a, M> Iterator for Entries<'a, M> {
    type Item = Entry<'a, M>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_file().or_else(|| self.next_dir())
    }
}

/// An entry in the directory index.
///
/// Can represent either a directory, or a file with metadata.
pub struct Entry<'a, M> {
    pub name: &'a OsStr,
    pub meta: Option<&'a M>,
    pub kind: EntryKind,
}

impl<M> Index<M> {
    pub fn new() -> Self {
        Self { root: Node::new() }
    }

    pub fn entries<P: AsRef<Path>>(&self, path: P) -> Entries<M> {
        let path = normalize_path(path.as_ref());
        entries(path.into_iter().collect(), &self.root)
    }

    pub fn remove<P: AsRef<Path>>(&self, _path: P) -> Option<M> {
        unimplemented!()
    }

    pub fn insert<P: Into<PathBuf>>(&mut self, path: P, meta: M) {
        let path = path.into();
        let path = normalize_path(&path);
        insert(path.into_iter().collect(), &mut self.root, meta)
    }

    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<&M> {
        let path = normalize_path(path.as_ref());
        get(path.into_iter().collect(), &self.root)
    }

    pub fn contains<P: AsRef<Path>>(&self, path: P) -> bool {
        self.get(path).is_some()
    }

    pub fn clear(&mut self) {
        self.root.files.clear();
        self.root.dirs.clear();
    }
}

fn entries<'a, M>(mut parts: VecDeque<&OsStr>, node: &'a Node<M>) -> Entries<'a, M> {
    let f0 = parts.pop_front();
    match (f0, parts.front()) {
        (None, _) => Entries {
            files: Some(node.files.iter()),
            dirs: Some(node.dirs.iter()),
        },
        (Some(dir), _) => {
            if let Some(node) = node.dirs.get(dir) {
                entries(parts, node)
            } else {
                Entries {
                    files: None,
                    dirs: None,
                }
            }
        }
    }
}

fn insert<M>(mut parts: VecDeque<&OsStr>, node: &mut Node<M>, meta: M) {
    let f0 = parts.pop_front();
    match (f0, parts.front()) {
        (None, _) => {}
        (Some(file), None) => {
            if node.dirs.get(file).is_none() {
                node.files.insert(file.to_os_string(), meta);
            }
        }
        (Some(dir), Some(_)) => {
            node.files.remove(dir);
            if let Some(dir) = node.dirs.get_mut(dir) {
                insert(parts, dir, meta)
            } else {
                let name = dir.to_os_string();
                let mut new_node = Node::new();
                insert(parts, &mut new_node, meta);
                node.dirs.insert(name, new_node);
            }
        }
    }
}

fn get<'a, M>(mut parts: VecDeque<&OsStr>, node: &'a Node<M>) -> Option<&'a M> {
    let f0 = parts.pop_front();
    match (f0, parts.front()) {
        (None, _) => None,
        (Some(file), None) => node.files.get(file),
        (Some(dir), Some(_)) => {
            if let Some(dir) = node.dirs.get(dir) {
                get(parts, dir)
            } else {
                None
            }
        }
    }
}

/// Normalizes path by removing any references to the parent (`..`) and the
/// current (`.`) directory.
///
/// ```
/// use mini_fs::index::normalize_path;
/// use std::path::Path;
///
/// assert_eq!(Path::new("/"), normalize_path(Path::new("/a/b/c/../../..")));
/// assert_eq!(Path::new("foo"), normalize_path(Path::new("./foo")));
/// ```
#[doc(hidden)]
pub fn normalize_path(path: &Path) -> Cow<Path> {
    use std::path::Component::*;
    if path.components().any(|c| match c {
        CurDir | ParentDir => true,
        _ => false,
    }) {
        let mut normal = PathBuf::new();
        for comp in path.components() {
            match comp {
                CurDir => {}
                ParentDir => {
                    normal.pop();
                }
                Normal(n) => normal.push(n),
                RootDir => normal.push("/"),
                _ => {}
            }
        }
        Cow::Owned(normal)
    } else {
        Cow::Borrowed(path)
    }
}
