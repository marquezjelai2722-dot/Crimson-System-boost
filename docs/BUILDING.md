# Building on Windows

## Requirements

- Windows 10 or Windows 11 (feature support is checked separately at runtime);
- stable Rust toolchain;
- rustfmt and Clippy components.

## Debug build

```powershell
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo run
```

## Optimized executable

```powershell
cargo build --release
`.\target\release\crimson-system-boost.exe
```

The executable is generated under `target/release/`. That directory is intentionally ignored by Git. A signed installer and release packaging workflow have not been implemented yet.
