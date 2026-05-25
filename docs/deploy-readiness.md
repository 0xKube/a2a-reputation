# Deploy Readiness

This project uses a V1 read model with migration/export support plus a V2 query-scoring route.

## Migration surface

- `oracle_status()` returns `Active` or `ReadOnly`.
- `set_read_only(read_only)` toggles write availability.
- `ReadOnly` rejects operator changes, read-model writes, and attestations.
- Queries and exports continue while read-only.
- `export_migration_config()` returns owner, status, read-only flag, bounds, counts, report version, and timestamp.
- `export_operators()` returns operators in deterministic order.
- `export_read_models_chunk(cursor, limit)` returns deterministic read-model chunks.
- `export_attestations_chunk(cursor, limit)` returns deterministic attestation chunks.
- `export_stats()` returns count invariants.

## V2 scoring surface

- `score_agent_v2(subject, signals, evidence)` is present in the generated IDL/client.
- V2 packets separate observed fields, inferred risk, unknown fields, and unverified links.
- The fixture gclient smoke validates V1 packet write/read plus V2 generated-client payload compatibility.

## Usage prediction stake bounds

`open_usage_prediction` requires attached value between 10 and 10,000 VARA, using 12-decimal base units. Below-minimum and above-maximum stakes are rejected before position creation.

Usage predictions settle against window deltas: `predicted_delta_calls` is compared with `actual_delta_calls` for `[window_start_ms, window_end_ms]`, not with a cumulative/lifetime total. Standard windows are 3 hours. The contract rejects settlement before `window_end_ms`; after the keeper settles the closed window, the gclient smoke verifies rolling into the next 3-hour epoch for the same subject while preserving both positions in export state.

## Local release gate

Use Node 22.

```bash
npm ci
npm test
npm run sails:test
npm run deploy:wasm
```

Optional local Gear/gclient smoke:

```bash
GEAR_NODE_PATH='gear --dev' npm run smoke:graphql-gclient:full
```

A local candidate is clean when:

1. JS tests pass.
2. Sails/gtest passes.
3. `deploy:wasm` reports `deployReady: true`.
4. `.outputs/deploy/reputation_oracle.opt.wasm` and `.outputs/deploy/reputation_oracle.idl` are regenerated from current source.
5. Fixture gclient smoke passes, or clearly skips because local Gear prerequisites are unavailable.
6. `git diff --check` is clean.

## Local wallet smoke

`npm run smoke:local` uses `vara-wallet` against `ws://localhost:9944`.

```bash
npm run smoke:local
```

To execute the local wallet write path:

```bash
export VARA_WALLET_ACCOUNT=<funded-local-account-name>
npm run smoke:local -- --execute
```

The write path uploads `.outputs/deploy/reputation_oracle.opt.wasm`, queries `ExportMigrationConfig`, calls `SetReadOnly`, and queries `ExportMigrationConfig` again.
