#!/usr/bin/env node
import { runLocalSmoke } from '../src/local-smoke.mjs';

const args = new Set(process.argv.slice(2));
const execute = args.has('--execute') || process.env.LOCAL_SMOKE_ALLOW_WRITES === '1';
const result = runLocalSmoke({ execute });

console.log(JSON.stringify(result, null, 2));

if (result.status === 'FAIL') {
  process.exitCode = 1;
}
