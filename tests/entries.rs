use mini_fs::prelude::*;
use mini_fs::{EntryKind, LocalFs, MiniFs};
use std::ffi::OsStr;
use std::io::Result;

#[test]
fn mini_fs_entries() {
    let local = LocalFs::new("./tests/local");

    let files = MiniFs::new().mount("/files", local);

    assert!(files.entries("/nope").unwrap().next().is_none());
    assert!(files.entries("/files").unwrap().next().is_some());

    let mut entries = files
        .entries("/files")
        .unwrap()
        .collect::<Result<Vec<_>>>()
        .unwrap();

    assert_eq!(3, entries.len());

    entries.sort_by_key(|e| e.name.clone());

    assert_eq!(OsStr::new("bar"), entries[0].name);
    assert_eq!(OsStr::new("baz"), entries[1].name);
    assert_eq!(OsStr::new("foo"), entries[2].name);
}

#[test]
fn overriden_entries_0() {
    let f0 = LocalFs::new("./tests/local/");
    let f1 = LocalFs::new("./tests/local/baz");

    let files = MiniFs::new().mount("/f0", f0).mount("/f0", f1);

    let mut entries = files
        .entries("/f0")
        .unwrap()
        .collect::<Result<Vec<_>>>()
        .unwrap();

    assert_eq!(1, entries.len());
}

#[test]
fn overriden_entries_1() {
    let f0 = LocalFs::new("./tests/local/baz");
    let f1 = LocalFs::new("./tests/local/");

    let files = MiniFs::new().mount("/f0", f0).mount("/f0", f1);

    let mut entries = files
        .entries("/f0")
        .unwrap()
        .collect::<Result<Vec<_>>>()
        .unwrap();

    assert_eq!(3, entries.len());
}

#[test]
fn tuple_no_repeats() {
    let a = LocalFs::new("./tests/local");
    let b = LocalFs::new("./tests/local");

    let files = (a, b);
    let entries = files
        .entries("./")
        .unwrap()
        .collect::<Result<Vec<_>>>()
        .unwrap();

    assert_eq!(3, entries.len())
}

#[test]
fn local_trait_object_entries() {
    use mini_fs::prelude::*;
    use mini_fs::{LocalFs, Store};
    use std::path::Path;

    let local: Box<dyn Store<File = <LocalFs as Store>::File>> =
        Box::new(LocalFs::new("./tests/local"));

    let mut entries = local
        .entries_path(Path::new("./././"))
        .expect("entry iterator")
        .collect::<Result<Vec<_>>>()
        .expect("iterator result");

    entries.sort_by_key(|e| e.name.clone());

    assert_eq!(3, entries.len());

    assert_eq!(OsStr::new("bar"), entries[0].name);
    assert_eq!(OsStr::new("baz"), entries[1].name);
    assert_eq!(OsStr::new("foo"), entries[2].name);

    assert_eq!(EntryKind::File, entries[0].kind);
    assert_eq!(EntryKind::Dir, entries[1].kind);
    assert_eq!(EntryKind::File, entries[2].kind);
}

#[test]
fn local_entries() {
    let local = LocalFs::new("./tests/local");

    let mut entries = local
        .entries("./")
        .expect("entry iterator")
        .collect::<Result<Vec<_>>>()
        .expect("iterator result");

    entries.sort_by_key(|e| e.name.clone());

    assert_eq!(3, entries.len());

    assert_eq!(OsStr::new("bar"), entries[0].name);
    assert_eq!(OsStr::new("baz"), entries[1].name);
    assert_eq!(OsStr::new("foo"), entries[2].name);

    assert_eq!(EntryKind::File, entries[0].kind);
    assert_eq!(EntryKind::Dir, entries[1].kind);
    assert_eq!(EntryKind::File, entries[2].kind);

    let mut entries = local.entries("./baz").expect("entry iterator");

    assert_eq!(
        OsStr::new("baz/foobar"),
        entries.next().unwrap().map(|e| e.name).unwrap()
    );
}
