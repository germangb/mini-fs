# mini-fs

Stupid simple (ro) filesystem abstraction.

```rust
let fs = Local::pwd()?; // local filesystem
let ram = Ram::new();   // in-memory

// Merge native and in-memory fs.
let merge = Merged(ram, fs);

let fs = MiniFs::new().mount("/data", merge).build()?;

let file = fs.open("/data/Cargo.toml")?;
```
