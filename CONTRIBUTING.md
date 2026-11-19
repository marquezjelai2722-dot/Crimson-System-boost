# Contributing

## Development setup

1. Install the stable Rust toolchain for Windows.
2. Install the `rustfmt` and `clippy` components.
3. Clone the repository and switch to `rust-rebuild/night-ops`.
4. Run the checks below before opening a pull request.

```powershell
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

## Change rules

- Keep changes small and explain the user-visible effect.
- Do not add a system tweak without a compatibility check, verified restore point, before-state snapshot, audit entry, and tested undo.
- Never disable Defender, Firewall, Windows Update, drivers, or essential Windows services.
- Do not add telemetry or network access without a clear purpose and explicit disclosure.
- Do not commit build output, local configuration, logs, recordings, tokens, or signing keys.
- Update documentation when behavior or supported Windows versions change.

## Pull requests

Describe the problem, implementation, risks, test results, and rollback behavior. Mark unimplemented functionality as planned; do not present mock readings or placeholders as working features.
