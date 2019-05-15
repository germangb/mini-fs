use mini_fs::index::*;
use std::path::Path;

#[test]
fn index_insert_get() {
    let mut index = Index::new();

    index.insert("a/foo.txt", 1);
    index.insert("a/b/c.txt", 2);
    index.insert("bar.txt", 4);
    index.insert("baz.txt", 8);

    #[rustfmt::skip]
    //   |- a
    //   |  |- foo.txt
    //   |  |- b
    //   |     |- c.txt
    //   |
    //   |- bar.txt
    //   |- baz.txt

    assert_eq!(Some(&1), index.get("a/foo.txt"));
    assert_eq!(Some(&2), index.get("a/b/c.txt"));
    assert_eq!(Some(&4), index.get("bar.txt"));
    assert_eq!(Some(&8), index.get("baz.txt"));

    assert_eq!(Some(&1), index.get("./a/foo.txt"));
    assert_eq!(Some(&2), index.get("./a/b/c.txt"));
    assert_eq!(Some(&4), index.get("./bar.txt"));
    assert_eq!(Some(&8), index.get("./baz.txt"));

    assert_eq!(None, index.get("nope"));

    assert_eq!(None, index.get("/a/foo.txt"));
    assert_eq!(None, index.get("/a/b/c.txt"));
    assert_eq!(None, index.get("/bar.txt"));
    assert_eq!(None, index.get("/baz.txt"));
}

#[test]
fn index_entries_2() {
    let mut index = Index::new();

    index.insert("nested/", ());
    index.insert("nested/hello.txt", ());
    index.insert("nested/world.txt", ());
    index.insert("nested", ());
    index.insert("hello.txt", ());
    index.insert("world.txt", ());

    #[rustfmt::skip]
    //   |- nested
    //   |  |- hello.txt
    //   |  |- world.txt
    //   |
    //   |- hello.txt
    //   |- world.txt

    assert_eq!(2, index.entries("nested").collect::<Vec<_>>().len());
    assert_eq!(3, index.entries(".").collect::<Vec<_>>().len());
}

#[test]
fn index_entries() {
    let mut index = Index::new();

    index.insert("a/foo.txt", ());
    index.insert("a/b/c.txt", ());
    index.insert("a/b/d.txt", ());
    index.insert("foo.txt", ());
    index.insert("bar.txt", ());
    index.insert("baz.txt", ());

    #[rustfmt::skip]
    //   |- a
    //   |  |- foo.txt
    //   |  |- b
    //   |     |- c.txt
    //   |     |- d.txt
    //   |
    //   |- foo.txt
    //   |- bar.txt
    //   |- baz.txt

    assert_eq!(4, index.entries(".").collect::<Vec<_>>().len());
    assert_eq!(2, index.entries("a").collect::<Vec<_>>().len());
    assert_eq!(2, index.entries("a/b/").collect::<Vec<_>>().len());
}

#[test]
fn test_normal_path() {
    assert_eq!(
        Path::new("smb:///a/b/c"),
        normalize_path(Path::new("smb:///a/b/c"))
    );
    assert_eq!(
        Path::new("/a/b/c"),
        normalize_path(Path::new("/a/b/c/../c"))
    );
    assert_eq!(Path::new("/a/b"), normalize_path(Path::new("/a/b/c/.././")));
    assert_eq!(
        Path::new("/"),
        normalize_path(Path::new("/a/b/c/.././../../"))
    );
}
