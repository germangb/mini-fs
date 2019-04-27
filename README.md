# mini-fs

[![Build Status](https://travis-ci.org/germangb/mini-fs.svg?branch=master)](https://travis-ci.org/germangb/mini-fs)
[![Master docs](https://img.shields.io/badge/docs-master-blue.svg?style=flat-square)](https://germangb.github.io/mini-fs/)

Stupid simple (ro) filesystem abstraction.

Supports reading files from the local filesystem and tar & zip archives.

```rust
use std::path::Path;
use mini_fs::{Store, Local, Tar, merge};

let tar = Tar::open("archive.tar")?;
let local = Local::pwd()?;

// Merge the two filesystems into one.
let merge = merge!(tar, local);

let fs = MiniFs::new().mount("/data", merge);

let file = fs.open(Path::new("/data/hello.txt"))?;
```
