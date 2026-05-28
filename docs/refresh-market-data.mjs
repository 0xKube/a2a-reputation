#!/usr/bin/env node
import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  VARA_GRAPHQL_ENDPOINT,
  graphqlRequest,
  normalizeOperatorPreviewV2,
} from '../src/graphql-preview.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const outPath = resolve(__dirname, 'data/markets.json');
const wallet = process.env.VARA_WALLET_BIN || 'vara-wallet';
const account = process.env.VARA_ACCOUNT || 'kubai';
const idl = resolve(__dirname, '../.outputs/deploy/reputation_oracle.idl');
const programs = [
  {
    label: 'V1',
    programId: '0x3c006c9daf828aa6dd237c012ea683335ffb2d455e443d7d9ab3593612f30775',
  },
  {
    label: 'V2',
    programId: '0x580b6bae88499c2595985acf7d8d320e3f0eaf5187f3dc47fd773c9c97b8f62a',
  },
];

function varaCall(programId, method, args) {
  const raw = execFileSync(wallet, [
    '--account', account,
    '--network', 'mainnet',
    '--json',
    'call', programId, method,
    '--args', JSON.stringify(args),
    '--idl', idl,
  ], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  return JSON.parse(raw);
}

function statusKind(status) {
  if (!status) return 'Unknown';
  if (typeof status === 'string') return status;
  return status.kind || Object.keys(status)[0] || 'Unknown';
}

function vara(raw) {
  return Number(BigInt(raw || '0')) / 1_000_000_000_000;
}

function evidenceFlags(packet) {
  const flags = [];
  const e = packet.evidenceV2 || [];
  if (e.some((x) => x.includes('github'))) flags.push('github');
  if (e.some((x) => x.includes('identity'))) flags.push('identity');
  if (e.some((x) => x.includes('handle'))) flags.push('registry');
  if (packet.signalsV2?.inbound_call_count > 0) flags.push('calls');
  if (packet.signalsV2?.mention_count > 0) flags.push('mentions');
  if (packet.signalsV2?.posts_active > 0) flags.push('board');
  return flags.slice(0, 3).join(' • ') || 'thin evidence';
}

const gql = await graphqlRequest({ endpoint: VARA_GRAPHQL_ENDPOINT, variables: { first: 100 } });
const packets = normalizeOperatorPreviewV2(gql)
  .sort((a, b) => b.scoresV2.overall_score - a.scoresV2.overall_score);
const leaderboard = packets.slice(0, 10).map((packet, index) => ({
  rank: index + 1,
  handle: packet.subject,
  score: packet.scoresV2.overall_score,
  verdict: packet.scoresV2.verdict.replaceAll('_', ' '),
  flags: evidenceFlags(packet),
  avg3hCallsLast12h: Number(((packet.signalsV2?.inbound_call_count || 0) / 4).toFixed(1)),
  market: 'none',
}));

const rawPositions = [];
for (const program of programs) {
  try {
    const exported = varaCall(program.programId, 'ReputationOracle/ExportUsagePredictionsChunk', [0, 100]);
    for (const item of exported.result?.items || []) rawPositions.push({ ...item, program: program.label, programId: program.programId });
  } catch (error) {
    rawPositions.push({ error: String(error.message || error), program: program.label, programId: program.programId });
  }
}

const openPositions = rawPositions.filter((p) => statusKind(p.status) === 'Open');
const settledPositions = rawPositions.filter((p) => ['Won', 'Lost'].includes(statusKind(p.status)));
const recentWinners = settledPositions
  .filter((p) => statusKind(p.status) === 'Won')
  .sort((a, b) => Number(b.opened_at_ms || 0) - Number(a.opened_at_ms || 0))
  .slice(0, 12);
const bySubject = new Map();
for (const position of openPositions) {
  const key = position.subject || '@unknown';
  const current = bySubject.get(key) || {
    handle: key,
    forecast: `+${position.predicted_delta_calls || 0} calls`,
    stake: 0,
    time: 'open',
    positions: 0,
    pool: '0 VARA',
    sourceProgram: position.program,
    positionIds: [],
  };
  current.stake += vara(position.effective_stake || position.stake || '0');
  current.positions += 1;
  current.positionIds.push(position.position_id);
  current.forecast = `+${position.predicted_delta_calls || 0} calls`;
  bySubject.set(key, current);
}

const marketHandles = new Set(bySubject.keys());
for (const row of leaderboard) row.market = marketHandles.has(row.handle) ? 'open' : 'none';

for (const row of leaderboard.slice(0, 5)) {
  if (!bySubject.has(row.handle)) {
    bySubject.set(row.handle, {
      handle: row.handle,
      forecast: 'new market available',
      stake: 0,
      time: 'none',
      positions: 0,
      pool: '0 VARA',
      sourceProgram: 'V2',
      positionIds: [],
    });
  }
}

const markets = [...bySubject.values()].map((market) => {
  const packet = packets.find((candidate) => candidate.subject === market.handle);
  return {
    ...market,
    stake: Number(market.stake.toFixed(3)),
    score: packet?.scoresV2?.overall_score ?? 0,
    verdict: packet?.scoresV2?.verdict?.replaceAll('_', ' ') ?? 'unscored',
    note: market.positions
      ? `${market.positions} open position${market.positions === 1 ? '' : 's'} on ${market.sourceProgram}; keeper-settled from GraphQL usage delta.`
      : 'No current stake in this epoch; eligible for a new usage market.',
  };
}).sort((a, b) => b.stake - a.stake || b.score - a.score);

const payload = {
  generatedAt: new Date().toISOString(),
  endpoint: VARA_GRAPHQL_ENDPOINT,
  programs,
  markets,
  leaderboard,
  openPositions: openPositions.map((p) => ({
    position_id: p.position_id,
    predictor: p.predictor,
    epoch_id: p.epoch_id,
    subject: p.subject,
    window_start_ms: p.window_start_ms,
    window_end_ms: p.window_end_ms,
    predicted_delta_calls: p.predicted_delta_calls,
    evidence_hash: p.evidence_hash,
    stake: p.stake,
    effective_stake: p.effective_stake,
    status: p.status,
    payout: p.payout,
    program: p.program,
    programId: p.programId,
  })),
  recentWinners: recentWinners.map((p) => ({
    position_id: p.position_id,
    predictor: p.predictor,
    epoch_id: p.epoch_id,
    subject: p.subject,
    window_start_ms: p.window_start_ms,
    window_end_ms: p.window_end_ms,
    predicted_delta_calls: p.predicted_delta_calls,
    actual_delta_calls: p.actual_delta_calls,
    error_bps: p.error_bps,
    evidence_hash: p.evidence_hash,
    settlement_snapshot_hash: p.settlement_snapshot_hash,
    stake: p.stake,
    effective_stake: p.effective_stake,
    status: p.status,
    payout: p.payout,
    program: p.program,
    programId: p.programId,
  })),
  rawOpenPositionCount: openPositions.length,
};

mkdirSync(dirname(outPath), { recursive: true });
writeFileSync(outPath, `${JSON.stringify(payload, null, 2)}\n`);
console.log(outPath);
console.log(`markets=${markets.length} leaderboard=${leaderboard.length} openPositions=${openPositions.length}`);
