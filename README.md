# mini-fs

[![Build Status](https://travis-ci.org/germangb/mini-fs.svg?branch=master)](https://travis-ci.org/germangb/mini-fs)
[![Master docs](https://img.shields.io/badge/docs-master-blue.svg?style=flat-square)](https://germangb.github.io/mini-fs/)

Stupid simple (read only) filesystem abstraction.

Supports reading files from the local filesystem, as well as tar & zip archives.

```rust
use std::path::Path;
use mini_fs::{Store, Local, Tar};

// Declare a file system.
let local = Local::pwd().unwrap();

// Mount it.
let mut fs = MiniFs::new().mount("/data", local);

// Read files.
let file = fs.open(Path::new("/data/example.gif")).unwrap();

// Unmount it when you're done (drop the file system)
fs.unmount("/data").unwrap();
```

## Merging

You can merge multiple file systems so they share the same mounting point using a tuple:

```rust
let a = Local::new("data/");
// - example.txt

let b = Tar::open("archive.tar.gz")?;
// - hello.txt

let files = MiniFs::new().mount("/files", (a, b));

assert!(files.open("/files/example.txt").is_ok());
assert!(files.open("/files/hello.txt").is_ok());
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