use mini_fs::{Store, Zip};
use std::io::Read;
use std::path::Path;

#[test]
fn zip() {
    let file = include_bytes!("archive.zip");

    let zip = Zip::new(&file[..]);

    let mut hello = zip.open(Path::new("hello.txt")).unwrap();
    let mut world = zip.open(Path::new("world.txt")).unwrap();
    let mut hello_content = String::new();
    let mut world_content = String::new();
    hello.read_to_string(&mut hello_content).unwrap();
    world.read_to_string(&mut world_content).unwrap();
    assert_eq!("hello\n", hello_content);
    assert_eq!("world!\n", world_content);
}