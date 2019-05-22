use std::io::{Cursor, Read};
use std::path::Path;

#[test]
#[cfg(feature = "zip")]
fn zip() {
    use mini_fs::prelude::*;
    use mini_fs::ZipFs;

    let file = include_bytes!("archive.zip");
    let zip = ZipFs::new(Cursor::new(&file[..]));

    for _ in 0..4 {
        let mut hello = zip.open("hello.txt").unwrap();
        let mut world = zip.open("world.txt").unwrap();
        let mut hello_content = String::new();
        let mut world_content = String::new();
        hello.read_to_string(&mut hello_content).unwrap();
        world.read_to_string(&mut world_content).unwrap();
        assert_eq!("hello\n", hello_content);
        assert_eq!("world!\n", world_content);
    }
}

#[test]
#[cfg(feature = "zip")]
fn zip_entries() {
    use mini_fs::prelude::*;
    use mini_fs::ZipFs;

    let file = include_bytes!("archive2.zip");
    let zip = ZipFs::new(Cursor::new(&file[..])).index().unwrap();

    assert_eq!(2, zip.entries("nested").unwrap().collect::<Vec<_>>().len());
    assert_eq!(3, zip.entries(".").unwrap().collect::<Vec<_>>().len());
}
