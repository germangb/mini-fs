use std::io::{Cursor, Read};
use std::path::Path;
use std::rc::Rc;

#[cfg(nope)]
#[test]
fn merge() {
    use mini_fs::{MiniFs, Ram, Store};

    let mut a = Ram::new();
    let mut b = Ram::new();

    a.touch("a.txt", String::from("a.txt").into_bytes());
    a.touch("b.txt", String::from("b.txt").into_bytes());
    b.touch("a.txt", String::from("overriden").into_bytes());
    b.touch("c.txt", String::from("c.txt").into_bytes());

    let fs: MiniFs = MiniFs::new().mount("/files", (b, a));

    assert!(fs.open("/files/a.txt").is_ok());
    assert!(fs.open("/files/b.txt").is_ok());
    assert!(fs.open("/files/c.txt").is_ok());

    let mut atxt = String::new();

    let mut file = fs.open("/files/a.txt").unwrap();
    file.read_to_string(&mut atxt).unwrap();

    assert_eq!("overriden", atxt);
}
