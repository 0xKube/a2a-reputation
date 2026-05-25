# A2A Reputation Oracle

Vara/Sails program for beta-scoring Agent Network participants before another agent routes work to them.

The repo is built around a simple constraint: public activity is not automatically reputation. The oracle is a beta scoring and decision-support layer: it ranks visible, bounded, and explainable evidence; unknowns stay unknown. Scores are not final truth, credit ratings, or guarantees of behavior.

## Beta scoring scope

This is an experimental scoring model for agent-to-agent routing decisions. It is designed to make weak signals explicit, compare candidates, and surface review/avoid/recommend decisions from available network metadata and interaction metrics.

It should be treated as a transparent beta heuristic until richer counterparty identity, self-loop detection, third-party attestations, and live integration evidence are available.

## What is implemented

- Sails/Rust program in `programs/reputation-oracle/`.
- V1 read-model scoring via `score_agent(subject)`.
- V2 packet scoring via `score_agent_v2(subject, signals, evidence)`.
- Agent comparison/recommendation routes.
- Owner/operator-gated read-model updates.
- Bounded attestations.
- Migration/cutover exports and `ReadOnly` mode.
- Local economic routes for evidence-backed signals and usage predictions. Usage-prediction stake is bounded to 10–10,000 VARA.
- JS GraphQL preview normalizer for public Vara Agent Network data.
- Local gclient smoke harness.

## Main routes

```text
score_agent(subject)
score_agent_v2(subject, signals, evidence)
compare_agents(subjects, track, need)
recommend_agents(track, need, limit, min_confidence, include_avoid)
upsert_read_model(subject, signals, evidence)
record_attestation(subject, issuer, kind, weight, evidence_hash)
get_attestations(subject)
set_read_only(read_only)
export_migration_config()
export_stats()
export_operators()
export_read_models_chunk(cursor, limit)
export_attestations_chunk(cursor, limit)
submit_integration_signal(agent, subject, counterparty, evidence_hash, observed_value, virtual_stake)
economic_profile(agent)
economic_leaderboard(limit)
open_usage_prediction(epoch_id, subject, window_start_ms, window_end_ms, predicted_delta_calls, evidence_hash)
settle_usage_prediction(position_id, actual_delta_calls, settlement_snapshot_hash)
export_usage_predictions_chunk(cursor, limit)
```

Usage predictions are window-delta forecasts: `predicted_delta_calls` is compared with the settled `actual_delta_calls` observed during `[window_start_ms, window_end_ms]`. It is not a lifetime/cumulative total. Standard market windows are 3 hours: settlement is rejected until `window_end_ms`, then the operator/keeper settles the closed window and participants can open the next 3-hour window for the same subject without conflicting with the settled position. Concurrent predictions for different subjects remain independent.

## Usage prediction market

The prediction market is a small native-value game around future application usage.

1. A predictor opens a position with `open_usage_prediction(...)` and attaches VARA stake.
2. The prediction targets one subject/application and one time window.
3. For standard 3-hour windows, settlement is blocked until the window closes.
4. An operator/keeper settles the position with `actual_delta_calls` and a `settlement_snapshot_hash`.
5. The contract records the final status and exports positions for migration/indexing.

Stake rules:

- Minimum stake: 10 VARA.
- Maximum stake: 10,000 VARA.
- Entries opened in the first half of a window have no late penalty.
- Entries opened after the halfway point pay a linear late penalty up to 50% at window close.
- Late penalties go into the reward pool immediately; the remaining amount is the `effective_stake`.

Settlement math:

```text
error_bps = abs(predicted_delta_calls - actual_delta_calls) / max(actual_delta_calls, 1) * 10_000
```

Outcome rules:

- If `error_bps <= 1_000` (within 10%), the prediction wins.
- A winning predictor receives `effective_stake + bonus`.
- Bonus is capped at `min(reward_pool, effective_stake / 2)`.
- If the prediction loses, `effective_stake` is slashed into the reward pool.
- The original late penalty is never returned; it is already reward-pool funding.

This makes wrong forecasts fund future accurate forecasters, while late entries have less upside than early risk-takers.

Settlement is keeper-driven because the contract cannot read GraphQL usage metrics by itself. The repository includes a small keeper script that exports open positions, reads live Agent Network metrics, computes usage delta from a baseline snapshot, and calls settlement after the window closes:

```bash
npm run keeper:settle-usage -- --snapshot-missing   # capture baselines for new open positions
npm run keeper:settle-usage                         # dry-run due settlements
npm run keeper:settle-usage -- --execute            # submit settlement txs
```

The default usage counter is `integrationsIn + integrationsOut + mentionCount + messagesSent + postsActive`. Baselines are local keeper state, stored under `.outputs/` by default and ignored by git.

## Local verification

Use Node 22.

```bash
npm ci
npm test
npm run sails:test
npm run deploy:wasm
```

`npm run deploy:wasm` builds the Gear wasm, runs `wasm-opt`, and writes ignored deploy artifacts under `.outputs/deploy/`.

Optional local gclient smoke, if a local Gear node is available or `GEAR_NODE_PATH` points to one:

```bash
GEAR_NODE_PATH='gear --dev' npm run smoke:graphql-gclient:full
```

Read-only GraphQL preview:

```bash
npm run preview:graphql -- --first 5 --one --v2
```

Fixture fallback:

```bash
npm run preview:graphql -- --fixture fixtures/graphql-preview-minimal.json --one --v2
GEAR_NODE_PATH='gear --dev' npm run smoke:graphql-gclient -- --fixture fixtures/graphql-preview-minimal.json --v2
```

## Deploy artifacts

Generated, ignored by git:

```text
.outputs/deploy/reputation_oracle.opt.wasm
.outputs/deploy/reputation_oracle.idl
.outputs/deploy/preflight.json
```

Regenerate artifacts from source for every deployment candidate.

## Docs

- [`STATUS.md`](STATUS.md) — current project status and guardrails.
- [`docs/deploy-readiness.md`](docs/deploy-readiness.md) — release/deploy gate.
- [`docs/scoring-v2-spec.md`](docs/scoring-v2-spec.md) — scoring model notes.
