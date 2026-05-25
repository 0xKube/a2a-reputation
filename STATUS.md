# A2A Reputation Oracle — Status

_Last updated: 2026-05-25 22:28 MSK_

## Current focus

Private hardening pass for the usage-prediction market: window-delta semantics, stake bounds, and standard 3-hour rollover mechanics before any renewed public/deploy claims.

## Done

- Clean private repo surface is maintained; stale/internal submission docs were removed earlier and the repo remains private.
- Usage prediction market now uses **window deltas**: `predicted_delta_calls` vs `actual_delta_calls`, not cumulative/lifetime totals.
- Stake bounds are implemented: **10–10,000 VARA**.
- Standard 3-hour windows now reject settlement before `window_end_ms`.
- Rollover path is verified locally: settled epoch 1 can coexist with open epoch 2 for the same subject; exports preserve both positions without ID conflict.
- Gates passed after the rollover work: `npm test`, `npm run sails:test` (50), `npm run deploy:wasm`, full fixture gclient economic/upgrade smoke, and `git diff --check`.

## Current blockers / risks

- Repo is still private; reopening/public submission needs Jacob approval.
- Mainnet/application/social/deployment claims still require linked proof before appearing in public copy.
- Long/non-standard prediction windows are kept as compatibility/testing paths; public/demo language should describe standard 3-hour market windows only.

## Next 3 actions

1. Commit and push the private hardening changes.
2. Re-check public README/docs for exact claim boundaries after push.
3. Prepare the deploy/public approval checklist: wallet/operator, mainnet deploy, Registry/Board/Chat, proof capture, repo visibility.

## Runbook

```bash
cd /Users/ys/clawd/projects/a2a-reputation-oracle
export PATH="/opt/homebrew/opt/node@22/bin:/Users/ys/clawd/bin:$PATH"
npm test
npm run sails:test
npm run deploy:wasm
GEAR_NODE_PATH='/Users/ys/clawd/bin/gear 2' \
GCLIENT_SMOKE_UPGRADE=1 \
npm run smoke:graphql-gclient -- --fixture fixtures/graphql-preview-minimal.json --v2 --economic
git diff --check
```

## Public-copy constraints

- Do not include internal positioning/submission/outreach drafts in the public repo.
- Do not include local machine paths, private notes, secrets, wallet material, or personal/operator context.
- Do not claim mainnet deployment, Registry submission, Board identity card, live integrations, deployed frontend, external metadata, earnings, or exact self-loop/same-owner detection without linked proof.
- Do not imply public repo visibility until Jacob explicitly approves reopening.
