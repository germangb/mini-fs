# mini-fs

[![Build Status](https://travis-ci.org/germangb/mini-fs.svg?branch=master)](https://travis-ci.org/germangb/mini-fs)
[![Master docs](https://img.shields.io/badge/docs-master-blue.svg?style=flat-square)](https://germangb.github.io/mini-fs/)

Stupid simple (ro) filesystem abstraction.

```rust
let fs = Local::pwd()?; // local filesystem
let ram = Ram::new();   // in-memory

// Merge native and in-memory fs.
let merge = Merged(ram, fs);

let fs = MiniFs::new().mount("/data", merge).build()?;

let file = fs.open("/data/Cargo.toml")?;
```
