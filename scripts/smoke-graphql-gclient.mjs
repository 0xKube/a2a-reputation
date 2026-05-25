#!/usr/bin/env node
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { fetchOperatorPreview, graphqlRequest, normalizeOperatorPreview, normalizeOperatorPreviewV2, selectPreviewPacket, VARA_GRAPHQL_ENDPOINT } from '../src/graphql-preview.mjs';

const repoRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');

function argValue(name, fallback = null) {
  const prefix = `${name}=`;
  const inline = process.argv.find((arg) => arg.startsWith(prefix));
  if (inline) return inline.slice(prefix.length);
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : fallback;
}

const first = Number(argValue('--first', '25'));
const subject = argValue('--subject');
const fixturePath = argValue('--fixture');
const emitV2 = process.argv.includes('--v2');
const runEconomicSmoke = process.argv.includes('--economic');
let source;
let packets;
let packetsV2 = null;

if (fixturePath) {
  const fixture = JSON.parse(readFileSync(fixturePath, 'utf8'));
  const data = fixture.data ?? fixture;
  packets = normalizeOperatorPreview(data);
  if (emitV2) packetsV2 = normalizeOperatorPreviewV2(data);
  source = `fixture:${fixturePath}`;
} else {
  if (emitV2) {
    const data = await graphqlRequest({ endpoint: VARA_GRAPHQL_ENDPOINT, variables: { first } });
    packets = normalizeOperatorPreview(data);
    packetsV2 = normalizeOperatorPreviewV2(data);
  } else {
    packets = await fetchOperatorPreview({ endpoint: VARA_GRAPHQL_ENDPOINT, first });
  }
  source = `live:${VARA_GRAPHQL_ENDPOINT}`;
}

const packet = selectPreviewPacket(packets, { subject });
const packetV2 = emitV2 ? selectPreviewPacket(packetsV2, { subject }) : null;
const dir = mkdtempSync(path.join(tmpdir(), 'a2a-graphql-gclient-'));
const packetPath = path.join(dir, 'preview-packet.json');
writeFileSync(packetPath, `${JSON.stringify(packet, null, 2)}\n`);
if (packetV2) writeFileSync(path.join(dir, 'preview-packet-v2.json'), `${JSON.stringify(packetV2, null, 2)}\n`);

console.log(`graphql preview source: ${source}`);
console.log(`graphql preview packet: ${packet.subject} -> ${packetPath}`);
if (packetV2) console.log(`graphql preview packet v2: ${packetV2.subject} -> ${path.join(dir, 'preview-packet-v2.json')} verdict=${packetV2.scoresV2.verdict} overall=${packetV2.scoresV2.overall_score}`);

try {
  const result = spawnSync('cargo', ['run', '--manifest-path', 'programs/reputation-oracle/gclient-smoke/Cargo.toml'], {
    cwd: repoRoot,
    env: {
      ...process.env,
      GRAPHQL_PREVIEW_PACKET_PATH: packetPath,
      ...(packetV2 ? { GRAPHQL_PREVIEW_PACKET_V2_PATH: path.join(dir, 'preview-packet-v2.json') } : {}),
      ...(runEconomicSmoke ? { GCLIENT_SMOKE_ECONOMIC: '1' } : {}),
    },
    stdio: 'inherit',
  });
  if (result.error) throw result.error;
  process.exitCode = result.status ?? 1;
} finally {
  rmSync(dir, { recursive: true, force: true });
}
