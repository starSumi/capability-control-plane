# Schema lifecycle

The Rust types in `capability-protocol` are the only authored semantic source.
Generate or verify all checked client projections with:

```powershell
cargo run -p xtask --locked -- codegen
cargo run -p xtask --locked -- codegen --check
```

JSON Schema and TypeScript outputs live under `generated/`. They are checked in
for direct client consumption, but all edits must originate in Rust. The check
fails on byte drift, missing projections, or unexpected generated files.
Protocol compatibility is also tested through strict Serde decoding and the
fixtures in `fixtures/`.
