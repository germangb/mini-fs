use mini_fs::{Store, Tar};
use std::io::{Cursor, Read};
use std::path::Path;

#[test]
fn tar() {
    let file = include_bytes!("archive.tar");
    let tar = Tar::new(Cursor::new(&file[..]));
    for _ in 0..4 {
        let mut a = tar.open(Path::new("a.txt")).unwrap();
        let mut b = tar.open(Path::new("b.txt")).unwrap();
        let mut a_content = String::new();
        let mut b_content = String::new();
        a.read_to_string(&mut a_content).unwrap();
        b.read_to_string(&mut b_content).unwrap();
        assert_eq!("hello\n", a_content);
        assert_eq!("world!\n", b_content);
    }
}

#[test]
fn tar_gz() {
    let file = include_bytes!("archive.tar.gz");
    let tar = Tar::new(Cursor::new(&file[..]));

    for _ in 0..4 {
        let mut a = tar.open(Path::new("a.txt")).unwrap();
        let mut b = tar.open(Path::new("b.txt")).unwrap();
        let mut a_content = String::new();
        let mut b_content = String::new();
        a.read_to_string(&mut a_content).unwrap();
        b.read_to_string(&mut b_content).unwrap();
        assert_eq!("hello\n", a_content);
        assert_eq!("world!\n", b_content);
    }
}
