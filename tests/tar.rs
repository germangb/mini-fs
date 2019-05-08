use std::io::{Cursor, Read};
use std::path::Path;

#[test]
#[cfg(feature = "tar")]
fn tar() {
    use mini_fs::prelude::*;
    use mini_fs::{Store, Tar};

    let file = include_bytes!("archive.tar");
    let tar = Tar::new(Cursor::new(&file[..]));
    for _ in 0..4 {
        let mut a = tar.open("a.txt").unwrap();
        let mut b = tar.open("b.txt").unwrap();
        let mut a_content = String::new();
        let mut b_content = String::new();

        a.read_to_string(&mut a_content).unwrap();
        b.read_to_string(&mut b_content).unwrap();

        assert_eq!("hello\n", a_content);
        assert_eq!("world!\n", b_content);
        assert!(tar.open("nope").is_err());
    }
}

#[test]
#[cfg(feature = "tar")]
fn tar_gz() {
    use mini_fs::prelude::*;
    use mini_fs::Tar;

    let file = include_bytes!("archive.tar.gz");
    let tar = Tar::new(Cursor::new(&file[..]));
    for _ in 0..4 {
        let mut a = tar.open("a.txt").unwrap();
        let mut b = tar.open("b.txt").unwrap();
        let mut a_content = String::new();
        let mut b_content = String::new();

        a.read_to_string(&mut a_content).unwrap();
        b.read_to_string(&mut b_content).unwrap();

        assert_eq!("hello\n", a_content);
        assert_eq!("world!\n", b_content);
        assert!(tar.open("nope").is_err());
    }
}

// TODO enable test
//#[test]
#[cfg(feature = "tar")]
fn index() {
    use mini_fs::prelude::*;
    use mini_fs::Tar;

    let file = include_bytes!("archive.tar.gz");
    let tar = Tar::new(Cursor::new(&file[..])).index().unwrap();
}
