# mini-fs

[![Cargo package](https://img.shields.io/crates/v/mini-fs.svg?style=flat-square)](https://crates.io/crates/mini-fs)
[![Build Status](https://img.shields.io/travis/germangb/mini-fs/master.svg?style=flat-square)](https://travis-ci.org/germangb/mini-fs)
[![docs.rs docs](https://docs.rs/mini-fs/badge.svg?style=flat-square)](https://docs.rs/mini-fs)
[![Master docs](https://img.shields.io/badge/docs-master-blue.svg?style=flat-square)](https://germangb.github.io/mini-fs/)

Filesystem abstraction.

Supports opening files from the local filesystem, as well as tar & zip archives.

```toml
[dependencies]
mini-fs = "0.1"
```

An example showcasing the API:

```rust
use std::path::Path;
use mini_fs::{Store, Local, Tar, MiniFs};

// Declare some file systems.
let local = Local::pwd()?;
let tar = Tar::open("archive.tar.gz")?;

// Mount them.
let mut fs = MiniFs::new()
    .mount("/data", local)
    .mount("/archived", tar);

// To open (read) files:
let file = fs.open(Path::new("/data/example.gif"))?;

// Unmount it when you're done (drops the file system)
fs.umount("/data");
```

## Merging

You can merge multiple file systems so they share the same mount point using a tuple.

```rust
let a = Local::new("data/");
// |- example.txt

let b = Tar::open("archive.tar.gz")?;
// |- example.txt
// |- hello.txt

let files = MiniFs::new().mount("/files", (a, b));

assert!(files.open(Path::new("/files/example.txt")).is_ok()); // this "example.txt" is from "a"
assert!(files.open(Path::new("/files/hello.txt")).is_ok());
```

Note that if you tried to first mount `a`, followed by `b` on the same mount point, the first one would be shadowed by `b`.

## License

```
MIT License

Copyright (c) 2019 German Gomez Bajo

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```