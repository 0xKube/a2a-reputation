#!/usr/bin/env node
import { readFileSync } from 'node:fs';
import { fetchOperatorPreview, graphqlRequest, normalizeOperatorPreview, normalizeOperatorPreviewV2, selectPreviewPacket, VARA_GRAPHQL_ENDPOINT } from '../src/graphql-preview.mjs';

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

const first = Number(argValue('--first', '25'));
const subject = argValue('--subject');
const fixturePath = argValue('--fixture');
const one = hasArg('--one');
const v2 = hasArg('--v2');

let packets;
let source;
if (fixturePath) {
  const fixture = JSON.parse(readFileSync(fixturePath, 'utf8'));
  packets = v2 ? normalizeOperatorPreviewV2(fixture.data ?? fixture) : normalizeOperatorPreview(fixture.data ?? fixture);
  source = { mode: 'fixture', path: fixturePath };
} else {
  if (v2) {
    const data = await graphqlRequest({ endpoint: VARA_GRAPHQL_ENDPOINT, variables: { first } });
    packets = normalizeOperatorPreviewV2(data);
  } else {
    packets = await fetchOperatorPreview({ endpoint: VARA_GRAPHQL_ENDPOINT, first });
  }
  source = { mode: 'live', endpoint: VARA_GRAPHQL_ENDPOINT };
}

const output = one ? selectPreviewPacket(packets, { subject }) : { source: { ...source, format: v2 ? 'v2' : 'v1' }, count: packets.length, packets };
process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
