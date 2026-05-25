import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import {
  parseProgramId,
  runLocalSmoke,
} from '../src/local-smoke.mjs';

function withArtifacts(callback) {
  const dir = mkdtempSync(path.join(tmpdir(), 'local-smoke-test-'));
  try {
    const wasmPath = path.join(dir, 'reputation_oracle.opt.wasm');
    const idlPath = path.join(dir, 'reputation_oracle.idl');
    writeFileSync(wasmPath, 'wasm');
    writeFileSync(idlPath, 'idl');
    return callback({ wasmPath, idlPath });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

test('parses program id from vara-wallet upload output', () => {
  assert.equal(parseProgramId('{"programId":"0xabc"}'), '0xabc');
  assert.equal(parseProgramId('programId: 0xdef'), '0xdef');
  assert.equal(parseProgramId(''), null);
});

test('skips local smoke when vara-wallet is unavailable', () => {
  const result = runLocalSmoke({
    runner() {
      return { status: null, error: new Error('spawn vara-wallet ENOENT') };
    },
  });

  assert.equal(result.status, 'SKIP');
  assert.match(result.reason, /vara-wallet CLI not found/);
});

test('skips local smoke when local node is unavailable', () => {
  const calls = [];
  const result = runLocalSmoke({
    runner(command, args) {
      calls.push({ command, args });
      if (args.includes('--help')) return { status: 0, stdout: 'help', stderr: '' };
      return { status: 1, stdout: '', stderr: 'connection refused' };
    },
  });

  assert.equal(result.status, 'SKIP');
  assert.match(result.reason, /local Vara node unavailable/);
  assert.deepEqual(calls[1].args, ['--network', 'local', 'node', 'info']);
});

test('does not perform local writes unless execute is enabled', () => {
  withArtifacts(({ wasmPath, idlPath }) => {
    const calls = [];
    const result = runLocalSmoke({
      wasmPath,
      idlPath,
      runner(command, args) {
        calls.push({ command, args });
        return { status: 0, stdout: '{}', stderr: '' };
      },
    });

    assert.equal(result.status, 'SKIP');
    assert.match(result.reason, /write smoke is disabled/);
    assert.equal(calls.some(({ args }) => args.includes('upload')), false);
  });
});

test('execute path deploys and checks migration read-only routes', () => {
  withArtifacts(({ wasmPath, idlPath }) => {
    const calls = [];
    const result = runLocalSmoke({
      account: 'local-smoke',
      execute: true,
      wasmPath,
      idlPath,
      runner(command, args) {
        calls.push({ command, args });
        if (args.includes('upload')) {
          return { status: 0, stdout: '{"programId":"0xprogram"}', stderr: '' };
        }
        return { status: 0, stdout: '{}', stderr: '' };
      },
    });

    assert.equal(result.status, 'PASS');
    assert.equal(result.programId, '0xprogram');
    assert.deepEqual(calls[2].args.slice(0, 6), [
      '--network',
      'local',
      '--account',
      'local-smoke',
      'program',
      'upload',
    ]);
    assert.ok(calls[3].args.includes('ReputationOracle/ExportMigrationConfig'));
    assert.ok(calls[4].args.includes('ReputationOracle/SetReadOnly'));
    assert.ok(calls[5].args.includes('ReputationOracle/ExportMigrationConfig'));
  });
});
