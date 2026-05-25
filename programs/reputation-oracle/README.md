## The **reputation-oracle** program

[![Build Status](https://github.com/gear-tech/reputation-oracle/workflows/CI/badge.svg)](https://github.com/gear-tech/reputation-oracle/actions)

Program **reputation-oracle** for [⚙️ Gear Protocol](https://github.com/gear-tech/gear) written in [⛵ Sails](https://github.com/gear-tech/sails) framework.

The program workspace includes the following packages:
- `reputation-oracle` is the package allowing to build WASM binary for the program and IDL file for it.
  The package also includes integration tests for the program in the `tests` sub-folder
- `reputation-oracle-app` is the package containing business logic for the program represented by the `ReputationOracle` structure.
- `reputation-oracle-client` is the package containing the client for the program allowing to interact with it from another program, tests, or off-chain client.

### 🏗️ Building

```bash
cargo build --release
```

The deployable artifact is `target/wasm32-gear/release/reputation_oracle.opt.wasm`
with `target/wasm32-gear/release/reputation_oracle.idl`. The Sails/Gear build
can still emit an `.opt.wasm` file when Binaryen's `wasm-opt` is missing, so the
repo-level gate is:

```bash
npm run deploy:wasm
```

That command runs the release build, writes `.outputs/deploy/preflight.json`, and
only copies deploy artifacts into `.outputs/deploy/` when `wasm-opt` is on `PATH`
and the build emitted no `wasm-opt optimizations error` warning.

### ✅ Testing

```bash
npm run sails:test
```

The repo-level helper runs `cargo test` in this workspace and exposes the
project-local npm Binaryen binary to Cargo when available. To pass through a
specific Cargo test target, use `npm run sails:test -- --test gtest`.

# License

The source code is licensed under the [MIT license](LICENSE).
