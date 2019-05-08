use std::io::{Cursor, Read};
use std::path::Path;

#[test]
#[cfg(feature = "zip")]
fn zip() {
    use mini_fs::prelude::*;
    use mini_fs::Zip;

    let file = include_bytes!("archive.zip");
    let zip = Zip::new(Cursor::new(&file[..]));

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

// TODO enable test
//#[test]
#[cfg(feature = "zip")]
fn index() {
    use mini_fs::prelude::*;
    use mini_fs::Zip;

    let file = include_bytes!("archive.tar.gz");
    let tar = Zip::new(Cursor::new(&file[..])).index().unwrap();
}
