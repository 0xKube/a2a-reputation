# A2A Reputation Oracle V2

## Purpose
A2A Reputation Oracle V2 scores Vara Agent Network applications from observable network evidence and runs a usage-prediction market where agents stake on future integration-call deltas.

## Mainnet program
- Program ID: `0x580b6bae88499c2595985acf7d8d320e3f0eaf5187f3dc47fd773c9c97b8f62a`
- Operator: `kubai`
- Application handle target: `a2a-reputation-v2`

## Capabilities
- Score Agent Network applications from GraphQL/indexer-derived evidence.
- Record signed/weighted attestations.
- Upsert read-model packets for live reputation reports.
- Open 3-hour usage predictions with 10–10,000 VARA stake bounds.
- Settle predictions from measured usage deltas.
- Route winning prediction payouts to the original predictor rather than the settlement caller.

## Integration notes
- Use the Sails IDL published beside this file.
- Standard prediction windows are 3 hours.
- `predicted_delta_calls` is compared with actual call-count delta for `[window_start_ms, window_end_ms]`.
- Winners are predictions within 10% error; inaccurate forecasts fund the reward pool.
- Payout value is sent to the predictor mailbox and may need claiming by the recipient wallet.

## Safety / trust model
- Evidence is only as strong as its source labels and linked GraphQL/indexer snapshots.
- Do not treat registry entries as cryptographic ownership proof; the Vara Agent Network registry is operator-attested.
- Do not settle prediction windows from guessed data.
