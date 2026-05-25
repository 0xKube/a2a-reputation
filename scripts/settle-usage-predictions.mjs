#!/usr/bin/env node
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { graphqlRequest, toNonNegativeInt, VARA_GRAPHQL_ENDPOINT } from '../src/graphql-preview.mjs';

const repoRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');

function argValue(name, fallback = null) {
  const prefix = `${name}=`;
  const inline = process.argv.find((arg) => arg.startsWith(prefix));
  if (inline) return inline.slice(prefix.length);
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : fallback;
}

function hasArg(name) {
  return process.argv.includes(name);
}

const varaWalletBin = argValue('--vara-wallet-bin', process.env.VARA_WALLET_BIN ?? (existsSync('/Users/ys/clawd/bin/vara-wallet') ? '/Users/ys/clawd/bin/vara-wallet' : 'vara-wallet'));

function run(command, args, { json = false, allowFailure = false } = {}) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    env: process.env,
  });
  if (result.error) throw result.error;
  if (result.status !== 0 && !allowFailure) {
    throw new Error(`${command} ${args.join(' ')} failed\n${result.stdout}\n${result.stderr}`);
  }
  if (json) return JSON.parse(result.stdout);
  return result;
}

function usageCounter(metric = {}) {
  return (
    toNonNegativeInt(metric.integrationsIn) +
    toNonNegativeInt(metric.integrationsOut) +
    toNonNegativeInt(metric.mentionCount) +
    toNonNegativeInt(metric.messagesSent) +
    toNonNegativeInt(metric.postsActive)
  );
}

function subjectToHandle(subject) {
  return String(subject ?? '').trim().replace(/^@+/, '');
}

async function metricForSubject(subject, first) {
  const handle = subjectToHandle(subject);
  const data = await graphqlRequest({ endpoint: VARA_GRAPHQL_ENDPOINT, variables: { first } });
  const app = data.allApplications.nodes.find((candidate) => candidate.handle === handle);
  if (!app) throw new Error(`subject not found in live GraphQL applications: ${subject}`);
  const metric = data.allAppMetrics.nodes.find(
    (candidate) => candidate.applicationId?.toLowerCase() === app.id.toLowerCase(),
  ) ?? {};
  return { app, metric, counter: usageCounter(metric) };
}

function readBaselines(file) {
  if (!existsSync(file)) return {};
  return JSON.parse(readFileSync(file, 'utf8'));
}

function writeBaselines(file, baselines) {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, `${JSON.stringify(baselines, null, 2)}\n`);
}

const programId = argValue('--program-id', process.env.A2A_REPUTATION_PROGRAM_ID ?? '0x3c006c9daf828aa6dd237c012ea683335ffb2d455e443d7d9ab3593612f30775');
const idl = argValue('--idl', process.env.A2A_REPUTATION_IDL ?? '.outputs/deploy/reputation_oracle.idl');
const account = argValue('--account', process.env.VARA_ACCOUNT ?? 'kubai');
const network = argValue('--network', process.env.VARA_NETWORK ?? 'mainnet');
const baselineFile = argValue('--baseline-file', '.outputs/usage-prediction-baselines.json');
const first = Number(argValue('--first', '200'));
const execute = hasArg('--execute');
const createMissingBaselines = hasArg('--snapshot-missing');
const positionFilter = argValue('--position-id');
const nowMs = Number(argValue('--now-ms', Date.now().toString()));

const exportResult = run(varaWalletBin, [
  '--account', account,
  '--network', network,
  '--json',
  'call', programId,
  'ReputationOracle/ExportUsagePredictionsChunk',
  '--args', '[0,64]',
  '--idl', idl,
], { json: true });

const positions = exportResult.result?.items ?? [];
const baselines = readBaselines(path.resolve(repoRoot, baselineFile));
const actions = [];

for (const position of positions) {
  if (positionFilter && position.position_id !== positionFilter) continue;
  if (position.status?.kind !== 'Open') continue;

  const { app, metric, counter } = await metricForSubject(position.subject, first);
  const baseline = baselines[position.position_id];
  if (!baseline) {
    const snapshot = {
      subject: position.subject,
      applicationId: app.id,
      counter,
      metric,
      capturedAtMs: nowMs,
      note: 'Baseline captured by settle keeper. Positions opened before this snapshot should not be auto-settled from it.',
    };
    if (createMissingBaselines) {
      baselines[position.position_id] = snapshot;
      actions.push({ positionId: position.position_id, action: 'snapshot-baseline', counter, subject: position.subject });
    } else {
      actions.push({ positionId: position.position_id, action: 'missing-baseline', counter, subject: position.subject });
    }
    continue;
  }

  const due = nowMs >= Number(position.window_end_ms);
  const actualDelta = Math.max(0, counter - Number(baseline.counter ?? 0));
  const settlementSnapshotHash = `graphql:${app.id}:${baseline.counter}->${counter}:updated:${metric.updatedAt ?? 'unknown'}`.slice(0, 96);
  const planned = {
    positionId: position.position_id,
    subject: position.subject,
    due,
    predictedDelta: Number(position.predicted_delta_calls),
    baselineCounter: Number(baseline.counter),
    currentCounter: counter,
    actualDelta,
    settlementSnapshotHash,
  };

  if (!due) {
    actions.push({ ...planned, action: 'not-due' });
    continue;
  }

  if (!execute) {
    actions.push({ ...planned, action: 'dry-run-settle' });
    continue;
  }

  const settle = run(varaWalletBin, [
    '--account', account,
    '--network', network,
    '--json',
    'call', programId,
    'ReputationOracle/SettleUsagePrediction',
    '--args', JSON.stringify([position.position_id, actualDelta, settlementSnapshotHash]),
    '--idl', idl,
  ], { json: true });
  actions.push({ ...planned, action: 'settled', txHash: settle.txHash, blockNumber: settle.blockNumber, result: settle.result });
}

if (createMissingBaselines) writeBaselines(path.resolve(repoRoot, baselineFile), baselines);

console.log(JSON.stringify({
  execute,
  nowMs,
  programId,
  varaWalletBin,
  baselineFile,
  positionsSeen: positions.length,
  actions,
}, null, 2));
