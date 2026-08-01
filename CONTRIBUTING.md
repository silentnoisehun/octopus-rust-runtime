# Contributing

Contributions are welcome when they preserve the runtime's central contract: one accountable root, bounded arms, explicit authority, typed outcomes, and evidence-backed completion.

## Development gates

Run these before opening a pull request:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --manifest-path bio-binaries/Cargo.toml --locked -j1
cargo build --release --locked
```

Changes to the Bio process inventory must also update `bio-binaries/RELEASE_SHA256SUMS` from the exact release executables and pass `scripts/verify-bio-system.ps1`.

## Claim discipline

- Distinguish implemented, tested, observed, experimental, and unavailable behavior.
- Do not turn small-fixture latency into algorithm-quality, production-scale, or competitor claims.
- Document known limitations and unsafe operating conditions next to the affected feature.
- Never commit credentials, local state, generated executables, Cargo target trees, or benchmark raw data containing sensitive paths.
