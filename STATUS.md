# A2A Reputation Oracle — Status

_Last updated: 2026-05-29 01:35 MSK_

## Current focus

Public hackathon showcase is live: V2 payout routing is patched and the Vara Agent Markets frontend is available at https://vara.wecraft.fun/.

## Done

- Clean public repo surface is maintained; stale/internal submission docs were removed earlier.
- Usage prediction market now uses **window deltas**: `predicted_delta_calls` vs `actual_delta_calls`, not cumulative/lifetime totals.
- Stake bounds are implemented: **10–10,000 VARA**.
- Standard 3-hour windows now reject settlement before `window_end_ms`.
- Rollover path is verified locally: settled epoch 1 can coexist with open epoch 2 for the same subject; exports preserve both positions without ID conflict.
- Gates passed after the rollover work: `npm test`, `npm run sails:test` (50), `npm run deploy:wasm`, full fixture gclient economic/upgrade smoke, and `git diff --check`.
- V2 payout routing fix is implemented locally/publicly: settlement sends winning value to `position.predictor` instead of returning value to the settlement caller.
- Vara Agent Markets frontend added under `docs/`: static market board, wallet connect, `OpenUsagePrediction` write flow, V2 position refresh, and approximate win preview.
- Public demo frontend: https://vara.wecraft.fun/

## Current blockers / risks

- Mainnet/application/social/deployment claims should remain linked to proof; do not overclaim adoption beyond observable transactions/indexer data.
- Long/non-standard prediction windows are kept as compatibility/testing paths; public/demo language should describe standard 3-hour market windows only.

## Next 3 actions

1. Re-check frontend wallet signing from a real browser extension.
2. Keep `docs/data/markets.json` refreshed from V2 contract exports before major demos.
3. After prediction windows close, settle only with real GraphQL/indexer evidence.

## Runbook

```bash
npm install
npm test
npm run sails:test
npm run deploy:wasm
GCLIENT_SMOKE_UPGRADE=1 \
npm run smoke:graphql-gclient -- --fixture fixtures/graphql-preview-minimal.json --v2 --economic
git diff --check
```

## Public-copy constraints

- Do not include internal positioning/submission/outreach drafts in the public repo.
- Do not include local machine paths, private notes, secrets, wallet material, or personal/operator context.
- Do not claim mainnet deployment, Registry submission, Board identity card, live integrations, deployed frontend, external metadata, earnings, or exact self-loop/same-owner detection without linked proof.
