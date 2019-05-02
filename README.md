# mini-fs

[![Cargo package](https://img.shields.io/crates/v/mini-fs.svg?style=flat-square)](https://crates.io/crates/mini-fs)
[![Build Status](https://img.shields.io/travis/germangb/mini-fs/master.svg?style=flat-square)](https://travis-ci.org/germangb/mini-fs)
[![docs.rs docs](https://docs.rs/mini-fs/badge.svg?style=flat-square)](https://docs.rs/mini-fs)
[![Master docs](https://img.shields.io/badge/docs-master-blue.svg?style=flat-square)](https://germangb.github.io/mini-fs/)

**mini-fs** is an extensible virtual filesystem for the application layer.

Supports reading from both the native filesystem, as well as Tar & Zip archives.

```toml
[dependencies]
mini-fs = "0.2"
```

An example showcasing the API:

```rust
use mini_fs::{Store, Local, Tar, MiniFs};

// Declare some file systems.
let local = Local::pwd()?;
let tar = Tar::open("archive.tar.gz")?;

// Mount them.
let mut fs = MiniFs::new()
    .mount("/data", local)
    .mount("/archived", tar);

// To open (read) files:
let file = fs.open("/data/example.gif")?;

// Unmount it when you're done (drops the file system)
fs.umount("/data");
```

## Merging

You can merge multiple file systems so they share the same mount point using a tuple. This allows you to override certain files from another location.

Example use cases:

* Config files with default fallbacks.
* Replace assets on a game (modding).

```rust
let a = Local::new("data/");
// |- example.txt

let b = Tar::open("archive.tar.gz")?;
// |- example.txt
// |- hello.txt

let files = MiniFs::new().mount("/files", (a, b));

assert!(files.open("/files/example.txt").is_ok()); // this "example.txt" is from "a"
assert!(files.open("/files/hello.txt").is_ok());
```

Note that if you tried to first mount `a`, followed by `b` on the same mount point, the first one would be shadowed by `b`.

## Extensible

It is possible to define a new file store so you can read files from an archive format that is not directly supported by this crate.

You'll need to:

1. Define two types: the store itselt, and the type of file that it returns.
2. Implement the `Store` trait on the first type, and `UserFile` on the second.

---

For example, say you want to implement storage based on a Zip archive (this crate already has an implementation, but let's say you want to improve it).

You'd need to implement something like the following:

First, define the types for the storage and the files that it returns.
```rust
use std::io;
use mini_fs::{Store, File, UserFile};

// This the type that will be mounted. It represents the Storage itself.
struct MyZip { /*...*/ }

// This example implementation of Zip will return a slice of bytes for each
// entry in the archive. This type must implement both io::Read and io::Seek
// before implementing the UserFile trait.
struct MyZipEntry(io::Cursor<Box<[u8]>>);

impl UserFile for MyZipEntry {}

// The file needs to implement IO (having the io::Cursor makes it easy)
impl io::Read for MyZipEntry {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl io::Seek for MyZipEntry { /*...*/ }
```

And then, implement the `Store` trait:

```rust
impl Store for MyZip {
    type File = MyZipEntry;
    
    fn open_path(&self, path: &Path) -> io::Result<MyZipEntry> {
        // Fetch file
        // ...
    }
}
```

---

And that is all, Now you can mount this store and/or use it as part of a tuple.

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