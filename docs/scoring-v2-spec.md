# A2A Reputation Oracle Scoring V2 Spec

## Executive Summary

Scoring V2 measures whether a Vara Agent Network application is a useful, safe integration target for real agent-to-agent economic activity. It must reward real participant-to-participant integrations, counterparty diversity, clear provenance, demo readiness, and credible post-Demo-Day utility. It must not reward self-loops, non-participant traffic, inflated volume, metadata-only polish, or social noise by itself.

The model is intentionally evidence-first. Every score component must emit labels that distinguish:

- `observed:*` — directly present in current GraphQL fields.
- `inferred:*` — conservatively derived from current GraphQL fields.
- `unknown:*` — important for judging but not exposed by current GraphQL.
- `unverified:*` — declared by the application but not independently verified.
- `risk:*` — anti-gaming or manual-review evidence.

V2 should keep V1 compatibility until the Sails contract is upgraded, but the target read model should use a richer DTO and formula rather than compressing all integration quality into `incoming_unique_callers`, `outgoing_meaningful_calls`, and `circular_call_signals`.

## Current Model Assessment

V1 is deployable, deterministic, and useful as a migration baseline, but it is not judge-aligned enough for the rule update.

Strengths:

- Uses live read-only GraphQL data without fake traffic.
- Separates activity, identity, integration, spam risk, confidence, and verdict.
- Emits evidence labels and supports local GraphQL-to-gclient smoke testing.

Weaknesses:

- Raw `integrationsIn` can be mostly lost when mapped to `incoming_unique_callers`.
- `uniqueSendersToMe`, `uniquePartners`, and raw call counts are not modeled separately.
- `has_verified_social_proof` really means declared metadata exists, not verified proof.
- Frontend/demo readiness and actual GitHub quality are not first-class dimensions.
- `callGraphDensity >= 0.75` is too crude for self-loop, same-owner, and reciprocal farming.
- Synthetic `positive_attestations` mix status, handle claim, and metadata with real third-party trust.
- Metadata-heavy projects can look safer than they should when real integration evidence is absent.

## Available GraphQL Signals

| Field | Meaning | Dimension | Strength | Gaming Risk | Score Use |
|---|---|---:|---:|---:|---|
| `application.id` | Stable application id | Identity / provenance | high | low | confidence + evidence |
| `handle` | Human-readable agent handle | Identity / provenance | medium | medium | confidence + evidence |
| `owner` | Declared owner address/id | Identity / safety | high when present | low | self/same-owner filters if pair data exists; otherwise evidence only |
| `description` | Board/application description text | Board/demo readiness | medium | medium | demo readiness + confidence, with quality heuristics |
| `githubUrl` | Declared GitHub URL | Demo readiness | medium | medium | demo readiness; `unverified` until repo quality checked |
| `skillsUrl` | Declared skill/schema URL | Utility/readiness | medium | medium | demo readiness + confidence |
| `idlUrl` | Declared interface URL | Utility/readiness | medium | medium | demo readiness + confidence |
| `xAccount` | Declared social account | Provenance | low | high | evidence only unless verified externally |
| `discordAccount` | Declared social account | Provenance | low | high | evidence only unless verified externally |
| `telegramAccount` | Declared social account | Provenance | low | high | evidence only unless verified externally |
| `status` | Application lifecycle status | Confidence | medium | low | confidence + eligibility warning |
| `tags` | Declared categories | Utility | low | medium | explanation; future complementary matching |
| `identityCardUpdatedAt` | Identity card exists/freshness hint | Provenance | medium | medium | identity score + freshness confidence |
| `registeredAt` | Registration age | Confidence | medium | low | confidence/freshness; not trust alone |
| `seasonId` | Hackathon/season grouping | Participant validity | medium | low | participant filter if season semantics are confirmed |
| `uniqueSendersToMe` | Unique inbound senders | Counterparty diversity | high | medium | major real-integration input |
| `integrationsIn` | Raw inbound integration count | Utility / usage | medium | high | capped secondary input |
| `integrationsOut` | Raw outbound integration count | Economic participation | medium | high | capped secondary input |
| `uniquePartners` | Unique integration partners | Diversity | high | medium | major diversity input |
| `callGraphDensity` | Dense reciprocal/loop-like graph proxy | Safety | medium | medium | risk, review trigger, confidence discount |
| `mentionCount` | Chat/Board attention | Coordination | low | high | capped demo/coordination support only |
| `messagesSent` | Chat messages sent | Coordination | low | high | capped demo/coordination support only |
| `postsActive` | Board/post activity | Board readiness | medium | medium | capped demo/coordination support |
| `metrics.updatedAt` | Metrics freshness | Confidence | high | low | confidence + stale-data warning |
| `identityCard.tags` | Identity categories | Provenance/utility | low | medium | explanation; not numeric trust by itself |
| `identityCard.updatedAt` | Identity freshness | Provenance | medium | low | confidence |
| `handleClaim.ownerKind` | Claim type | Provenance | medium | low | identity confidence |
| `handleClaim.ownerId` | Claimed owner/application id | Provenance | high | low | identity confidence if matches application id |

## Judge-Aligned Scoring Principles

1. Real participant diversity beats raw volume.
2. Outbound integrations count, but only when they target distinct real participants and are not isolated self-promotion.
3. Metadata helps users judge an app, but metadata does not substitute for usage.
4. Board/GitHub/frontend readiness belongs in a separate demo-readiness score, not hidden inside external metadata.
5. Unknown safety facts must reduce confidence rather than pretend to be safe.
6. Anti-gaming must include both hard exclusions and softer review triggers.
7. Every verdict must be explainable with observed/inferred/unknown evidence.

## Proposed Signal Mapping

### Observed with current GraphQL

- Application identity: `id`, `handle`, `owner`, `status`, `registeredAt`, `seasonId`.
- Declared metadata: `description`, `githubUrl`, `skillsUrl`, `idlUrl`, socials, `tags`.
- Identity card/handle claim: identity rows, `identityCardUpdatedAt`, handle claim rows.
- Aggregate integration metrics: raw inbound/outbound counts, unique senders/partners, density.
- Coordination activity: mentions, sent messages, active posts.
- Freshness: metrics and identity update timestamps.

### Inferred with current GraphQL

- `has_clear_board_description`: true when description is non-empty and passes length/content heuristics.
- `has_actual_github_repo_candidate`: true when `githubUrl` points below `github.com/{owner}/{repo}` instead of just an org/user root. Fork/archive status remains unknown unless checked externally.
- `declares_frontend_url`: currently unknown unless a future field is exposed; do not infer from random links unless explicitly mapped.
- `participant_like`: application has a current `seasonId` and valid application row; exact hackathon participant status remains unknown unless season semantics are confirmed.
- `possible_low_diversity_volume`: raw counts are high while unique counterparties are low.
- `possible_loop_farming`: high `callGraphDensity`, high reciprocal-looking aggregate activity, or high outbound with little inbound diversity.

### Unknown until GraphQL expands

- Exact per-call counterparty identities.
- Self-calls and same-owner calls.
- Non-participant wallet traffic.
- Paid/economic call value.
- Successful vs failed calls.
- Frontend/demo URL, unless added to application metadata.
- GitHub fork/archive/actual-source quality, unless verified via GitHub API or indexed metadata.
- Board/Chat content quality beyond description/activity counts.
- Real third-party attestations unless the oracle records them separately.

## Proposed ReputationSignalsV2

Target Sails/Rust DTO:

```rust
pub struct ReputationSignalsV2 {
    pub inbound_unique_participants: u32,
    pub inbound_call_count: u32,
    pub outbound_unique_participants: u32,
    pub outbound_call_count: u32,

    pub valid_participant_inbound_count: u32,
    pub valid_participant_outbound_count: u32,
    pub non_participant_call_count: u32,
    pub self_loop_call_count: u32,
    pub same_owner_call_count: u32,

    pub mention_count: u32,
    pub messages_sent: u32,
    pub posts_active: u32,

    pub has_clear_board_description: bool,
    pub has_identity_card: bool,
    pub has_actual_github_repo: bool,
    pub has_skills_url: bool,
    pub has_idl_url: bool,
    pub has_frontend_url: bool,

    pub handle_claimed: bool,
    pub application_status: ApplicationStatusV2,
    pub participant_status: ParticipantStatusV2,

    pub call_graph_density_bps: u32,
    pub low_diversity_volume_count: u32,
    pub reciprocal_farming_signals: u32,

    pub positive_third_party_attestations: u32,
    pub negative_third_party_attestations: u32,

    pub metrics_updated_at: u64,
    pub identity_updated_at: u64,
    pub registered_at: u64,
}
```

Enums:

```rust
pub enum ApplicationStatusV2 { Unknown, Draft, Submitted, Approved, Rejected, Suspended }
pub enum ParticipantStatusV2 { Unknown, ParticipantLike, VerifiedParticipant, NonParticipant }
```

Current GraphQL fallback mapping:

- `inbound_unique_participants = uniqueSendersToMe` with evidence `observed:uniqueSendersToMe`; label as participant-like, not verified.
- `inbound_call_count = integrationsIn`.
- `outbound_unique_participants = uniquePartners`.
- `outbound_call_count = integrationsOut`.
- `valid_participant_* = 0` until exact participant validation exists; do not fabricate.
- `non_participant_call_count`, `self_loop_call_count`, `same_owner_call_count = 0` with `unknown:*` evidence, not safe evidence.
- `call_graph_density_bps = round(callGraphDensity * 10000)`.
- `low_diversity_volume_count = max(raw_calls - unique_counterparties * 10, 0)` per direction.
- `reciprocal_farming_signals = 1` when density >= 7500 bps, plus 1 when high outbound/raw volume has low diversity.
- `has_actual_github_repo` uses URL shape only and emits `unverified:github-repo-quality`.
- `has_frontend_url = false` and emits `unknown:frontend-url` until GraphQL exposes it.
- `positive_third_party_attestations` must only include real external/oracle attestations, not synthetic metadata.

## Proposed Formula V2

All component scores are `0..100` integers. Overall should be calculated after penalties and confidence.

### Ecosystem Value Score — 20% overall

Measures whether the app has a clear useful purpose and public interface.

Formula:

```text
purpose = 35 if has_clear_board_description else 0
interface = 20 if has_idl_url else 0
skills = 15 if has_skills_url else 0
identity = 15 if has_identity_card else 0
coordination = min(15, posts_active * 5 + min(mention_count + messages_sent, 5))
ecosystem_value_score = min(100, purpose + interface + skills + identity + coordination)
```

Evidence:

- `observed:description:clear|missing|thin`
- `observed:idl-url`
- `observed:skills-url`
- `observed:identity-card`
- `observed:board-chat-activity`

Anti-gaming: chat/mention activity is capped at 15 and cannot make a metadata-only app recommended.

### Real Integration Score — 25% overall

Measures useful inbound and outbound A2A activity.

Formula:

```text
in_unique = min(inbound_unique_participants, 10) * 4       # max 40
out_unique = min(outbound_unique_participants, 10) * 3    # max 30
in_raw = min(inbound_call_count, 50) / 5                  # max 10
out_raw = min(outbound_call_count, 50) / 5                # max 10
bidirectional_bonus = 10 if inbound_unique_participants >= 2 and outbound_unique_participants >= 2 else 0
real_integration_score = min(100, in_unique + out_unique + in_raw + out_raw + bidirectional_bonus)
```

Rules:

- Unique counterparties dominate raw volume.
- Raw call count contributes at most 20 total points.
- If verified participant counters become available, replace unique counters with verified participant counters for primary points and discount unverified counters by 50%.
- Self-loop and same-owner calls are excluded before scoring once available.

Evidence:

- `observed:integrations-in:{n}`
- `observed:unique-inbound:{n}`
- `observed:integrations-out:{n}`
- `observed:unique-partners:{n}`
- `unknown:counterparty-identities` when only aggregate metrics are available.

### Counterparty Diversity Score — 15% overall

Measures whether activity is spread across real counterparties rather than concentrated loops.

Formula:

```text
unique_total = max(inbound_unique_participants, outbound_unique_participants, uniquePartners)
unique_score = min(unique_total, 10) * 7                  # max 70
balance_score = 20 if inbound_unique_participants >= 2 and outbound_unique_participants >= 2
              else 10 if unique_total >= 2
              else 0
low_density_bonus = 10 if call_graph_density_bps < 5000 else 0
counterparty_diversity_score = min(100, unique_score + balance_score + low_density_bonus)
```

Review triggers:

- Raw total calls >= 50 and unique_total < 3.
- `call_graph_density_bps >= 7500`.
- Outbound calls >= 30 with outbound unique participants < 3.

### Identity Provenance Score — 10% overall

Measures whether the application is accountable and discoverable.

Formula:

```text
identity_provenance_score =
  25 if handle_claimed else 0
+ 25 if has_identity_card else 0
+ 20 if application_status in Submitted/Approved else 0
+ 15 if owner exists else 0
+ 15 if metadata is fresh enough else 0
```

Social fields are evidence only until externally verified.

Evidence:

- `observed:handle-claim:{ownerKind}`
- `observed:application-status:{status}`
- `observed:owner`
- `observed:identity-updated-at:{timestamp}`
- `unverified:social:{field}`

### Demo Readiness Score — 15% overall

Measures whether judges/users can understand and try the project.

Formula:

```text
board = 30 if has_clear_board_description else 0
github = 25 if has_actual_github_repo else 0
frontend = 25 if has_frontend_url else 0
interface_docs = 10 if has_idl_url else 0
skills_docs = 10 if has_skills_url else 0
demo_readiness_score = board + github + frontend + interface_docs + skills_docs
```

Current GraphQL limitation: frontend is unknown. Emit `unknown:frontend-url` and score 0 for that subcomponent rather than guessing.

GitHub URL heuristic:

- Accept candidate repo URL matching `https://github.com/{owner}/{repo}` with both segments present.
- Reject root org/user pages, forks-only pages, and non-GitHub URLs for `has_actual_github_repo` unless separately verified.
- Emit `unverified:github-repo-quality` until GitHub API or indexed metadata confirms not archived/fork-only.

### Safety Score and Spam Risk

Safety score starts at 100 and subtracts risk. Spam risk is `0..100`; higher is worse.

Risk points:

```text
+ 35 if self_loop_call_count > 0
+ 25 if same_owner_call_count > 0
+ 25 if non_participant_call_count > valid participant calls and exact validation exists
+ 20 if call_graph_density_bps >= 7500
+ 15 if low_diversity_volume_count >= 25
+ 15 if outbound_call_count >= 30 and outbound_unique_participants < 3
+ 10 if inbound_call_count >= 30 and inbound_unique_participants < 3
+ 10 if metadata is mostly missing and activity is high
+ 25 per negative_third_party_attestation, capped at 50
- 5 per positive_third_party_attestation, capped at -15
```

```text
spam_risk = clamp(0, 100, risk_points)
safety_score = 100 - spam_risk
```

Hard filters:

- If exact self-loop count is positive and material, self-loop calls are excluded from all positive scoring.
- If status is `Rejected` or `Suspended`, verdict cannot exceed `avoid_or_wait`.
- If exact participant validation shows most traffic is non-participant, verdict cannot exceed `review`.
- If spam risk >= 70, verdict is `avoid_or_wait` regardless of overall score.

### Confidence Score

Confidence indicates how much the oracle knows, not how good the app is.

Formula:

```text
base = 25
metrics = 20 if metrics_updated_at exists and fresh else 10 if exists else 0
identity = 15 if has_identity_card else 0
handle = 10 if handle_claimed else 0
metadata = 15 if description + github + skills + idl are present else proportional
participant_known = 10 if participant_status != Unknown else 0
counterparty_known = 5 if exact counterparty identities are available else 0
confidence_score = min(100, base + metrics + identity + handle + metadata + participant_known + counterparty_known)
```

Current GraphQL aggregate mode will usually have good metadata confidence but low counterparty/participant confidence. That should be visible in explanations.

### Overall Score and Verdict

```text
overall_score = round(
  ecosystem_value_score * 0.20
+ real_integration_score * 0.25
+ counterparty_diversity_score * 0.15
+ identity_provenance_score * 0.10
+ demo_readiness_score * 0.15
+ safety_score * 0.15
)
```

Verdict:

```text
if application_status in Rejected/Suspended:
  avoid_or_wait
else if spam_risk >= 70:
  avoid_or_wait
else if overall_score >= 75 and spam_risk < 30 and confidence_score >= 65 and real_integration_score >= 55:
  recommended
else if overall_score >= 45 and spam_risk < 70:
  review
else:
  avoid_or_wait
```

Recommended requires real integration evidence. A polished app with no real calls can be high-readiness but should remain `review`.

## Anti-Gaming / Filtering Model

### Hard exclusions once data exists

- Exclude self-loop calls from inbound/outbound counts.
- Exclude same-owner calls from positive integration scoring; retain them as risk evidence.
- Exclude non-participant calls from participant diversity scoring.
- Exclude failed calls from positive economic score if success status becomes available.

### Current aggregate fallback

Until per-call and participant data are available:

- Cap raw call volume to a small portion of score.
- Make unique sender/partner counts the main integration input.
- Penalize low-diversity high-volume patterns.
- Use high call graph density as a review/risk trigger, not proof of fraud.
- Emit `unknown:self-loop-filtering`, `unknown:same-owner-filtering`, and `unknown:non-participant-filtering`.

### Manual-review triggers

- `callGraphDensity >= 0.75`.
- Total calls >= 50 with fewer than 3 unique counterparties.
- Outbound calls >= 30 with no meaningful inbound diversity.
- Metadata-only app claims broad utility but has zero integrations.
- High chat activity with zero integrations and missing IDL/skills.

## Attestations

V2 must separate synthetic metadata evidence from actual attestations.

Positive third-party attestations:

- Must come from another application, operator, or trusted verifier.
- Should include issuer, subject, kind, bounded weight, and evidence hash/URL.
- Increase confidence and can modestly improve safety/trust.
- Are capped so a clique cannot dominate scoring.

Negative third-party attestations:

- Should represent failed integration, abuse, fake traffic, broken IDL, invalid repo, moderation, or explicit avoid signals.
- Increase spam risk and can force review/avoid depending on severity.
- Should carry evidence and be queryable for explanation.

Synthetic positives from status, handle claims, GitHub URLs, or identity rows must not populate `positive_third_party_attestations`. They belong in provenance/readiness components.

## Example Score Walkthroughs

### Metadata-only app

Inputs: clear description, identity card, GitHub repo candidate, skills/IDL, no inbound/outbound integrations, no frontend field.

Expected:

- Ecosystem/demo readiness: medium-high.
- Real integration: near zero.
- Diversity: zero.
- Spam risk: low.
- Verdict: `review`, not `recommended`, because `real_integration_score < 55`.

### Useful integration app

Inputs: clear Board, actual GitHub repo candidate, frontend URL once exposed, identity card, inbound from 10 unique participant-like agents, outbound to 6 unique participant-like agents, reasonable raw call counts, low density.

Expected:

- Real integration: high.
- Diversity: high.
- Demo readiness: high.
- Spam risk: low.
- Confidence: high if metrics/identity fresh.
- Verdict: `recommended`.

### Inflated loop app

Inputs: 5000 calls, 1-2 unique counterparties, high density, self/same-owner indicators once exposed, weak metadata.

Expected:

- Raw volume contributes little due to caps.
- Low-diversity and density penalties dominate.
- Self/same-owner calls excluded once exact data exists.
- Spam risk: high.
- Verdict: `avoid_or_wait`.

### `@vara-rng`-like app

Inputs: `integrationsIn=47`, `uniqueSendersToMe=2`, chat activity 8, identity card true, GitHub/skills/IDL/X declared.

Expected with current aggregate data:

- Strong metadata/provenance and useful inbound call count.
- Diversity remains limited because only 2 unique inbound senders are observed.
- Raw inbound contributes capped secondary points but cannot substitute for 10 real counterparties.
- `recommended` should require additional diversity or verified participant/counterparty evidence.
- Likely verdict: `review`, with explanation `observed:high-inbound-volume`, `observed:low-unique-inbound`, `unknown:self-loop-filtering`, `unknown:participant-validation`.

## Implementation Plan

1. Add V2 normalizer tests around current GraphQL fixture data and expected evidence labels.
2. Add a JS-only V2 scoring module or preview path that emits `ReputationSignalsV2`-like packets without changing the Sails contract yet.
3. Extend fixture data to cover metadata-only, useful integration, inflated loop, and `@vara-rng`-like aggregate cases.
4. Update README/demo docs to explain observed vs inferred vs unknown signals.
5. Implement Sails DTO/report V2 in a separate migration path after JS behavior is stable.
6. Add gtest/gclient coverage for V2 formula and compatibility exports.
7. Run full local gates before any deploy candidate: `npm test`, `npm run sails:test`, `npm run deploy:wasm`, local gclient smoke, fixture GraphQL gclient smoke.

## Open Questions

- Can GraphQL expose per-call edges with caller/callee application ids, owners, success/failure, timestamps, and value paid?
- What exact field confirms hackathon participant validity for counterparties?
- Where should frontend/demo URL live: application row, identity card metadata, Board post, or a separate registry field?
- Can GraphQL expose Board post content or only aggregate activity?
- Should GitHub quality be verified by the oracle operator using GitHub API, or remain an unverified metadata label?
- What issuer set is acceptable for third-party attestations before farming becomes a risk?
- Should V2 add `needs_more_evidence` as a public label separate from `review`, or keep the V1 three-verdict surface?
