use std::io::prelude::*;

use mini_fs::prelude::*;
use mini_fs::{CaselessFs, RamFs};

#[cfg(test)]
#[test]
fn caseless() {
    let mut ram = RamFs::new();
    ram.touch("/a.txt", b"low a".to_vec());
    ram.touch("/A.TXT", b"high a".to_vec());
    ram.touch("/b/b.txt", b"low b".to_vec());
    ram.touch("/B/B.TXT", b"high b".to_vec());
    let mut caseless = CaselessFs::new(ram);

    // open with exact path
    let mut txt = String::new();
    let mut file = caseless.open("/a.txt").unwrap();
    file.read_to_string(&mut txt).unwrap();
    assert_eq!("low a", txt);

    let mut txt = String::new();
    let mut file = caseless.open("/A.TXT").unwrap();
    file.read_to_string(&mut txt).unwrap();
    assert_eq!("high a", txt);

    let mut txt = String::new();
    let mut file = caseless.open("/b/b.txt").unwrap();
    file.read_to_string(&mut txt).unwrap();
    assert_eq!("low b", txt);

    let mut txt = String::new();
    let mut file = caseless.open("/B/B.TXT").unwrap();
    file.read_to_string(&mut txt).unwrap();
    assert_eq!("high b", txt);

    // add with get_mut
    caseless.get_mut().touch("/c.txt", b"c".to_vec());
    let mut txt = String::new();
    let mut file = caseless.open("/c.txt").unwrap();
    file.read_to_string(&mut txt).unwrap();
    assert_eq!("c", txt);

    // find with caseless path
    assert_eq!(caseless.find("/A.txt").len(), 2);

    assert_eq!(caseless.find("/b/B.txt").len(), 2);

    // open with caseless path
    let mut txt = String::new();
    let mut file = caseless.open("/A.tXt").unwrap();
    file.read_to_string(&mut txt).unwrap();
    assert_eq!(["low a", "high a"].iter().any(|s| s == &txt), true);

    let mut txt = String::new();
    let mut file = caseless.open("/b/B.tXt").unwrap();
    file.read_to_string(&mut txt).unwrap();
    assert_eq!(["low b", "high b"].iter().any(|s| s == &txt), true);
}
