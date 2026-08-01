---
title: Bio-Binaries
---

## Description
33 bio-inspired system utilities in Rust. Biological and quantum names are software metaphors, not physical claims.

## Usage
High-level, distributed system management via binary protocol (BioMessage).

## Input
CLI commands, `omega-master` commands, binary data packets.

## Output
Binary system state, logs, or executed bio-metaphorical operation.

## Example
`cargo run --release --bin omega-master -- start --port 8888`

## Dependencies
Rust, Cargo, tokio, bincode, blake3.

## Notes
Rust implementation with a binary BioMessage v2 wire format; selected CLI, registry, configuration, and legacy bridge surfaces use text or JSON. The subsystem contains 33 executable targets and integrates with Octopus across a separate native process boundary.
See `README.md`, `CAPABILITIES.md`, and `PHILOSOPHY.md` for full documentation.
