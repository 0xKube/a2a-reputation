import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import {
  assessDeployReadiness,
  buildCargoEnv,
  buildArtifactInventory,
  detectWasmOpt,
  resolveWasmOptCommand,
  stripAnsi,
} from '../src/deploy-wasm.mjs';
import { runSailsCargoTests } from '../src/sails-test.mjs';

test('assesses deployable artifacts only when wasm-opt ran cleanly', () => {
  const artifacts = {
    wasm: { exists: true, sizeBytes: 107060, path: '/tmp/reputation_oracle.wasm' },
    optimizedWasm: { exists: true, sizeBytes: 79697, path: '/tmp/reputation_oracle.opt.wasm' },
    idl: { exists: true, sizeBytes: 3101, path: '/tmp/reputation_oracle.idl' },
  };

  const summary = assessDeployReadiness({
    artifacts,
    wasmOpt: { found: true, version: 'wasm-opt version 123' },
    buildOutput: 'Finished release profile',
  });

  assert.equal(summary.deployReady, true);
  assert.deepEqual(summary.blockers, []);
  assert.equal(summary.checks.optimizedWasmNonEmpty, true);
});

test('blocks deploy-ready status when Sails reports missing wasm-opt', () => {
  const summary = assessDeployReadiness({
    artifacts: {
      wasm: { exists: true, sizeBytes: 107060, path: '/tmp/reputation_oracle.wasm' },
      optimizedWasm: { exists: true, sizeBytes: 79697, path: '/tmp/reputation_oracle.opt.wasm' },
      idl: { exists: true, sizeBytes: 3101, path: '/tmp/reputation_oracle.idl' },
    },
    wasmOpt: { found: false, version: null },
    buildOutput: '\u001b[33mwarning\u001b[0m: wasm-opt optimizations error: wasm-opt not found!',
  });

  assert.equal(summary.deployReady, false);
  assert.ok(summary.blockers.includes('Binaryen wasm-opt is not on PATH'));
  assert.ok(summary.blockers.includes('Sails/Gear build reported wasm-opt optimization did not run'));
  assert.equal(summary.checks.buildHadWasmOptWarning, true);
});

test('inventories expected Sails deploy artifacts', () => {
  const dir = mkdtempSync(path.join(tmpdir(), 'deploy-wasm-test-'));
  try {
    writeFileSync(path.join(dir, 'reputation_oracle.wasm'), 'wasm');
    writeFileSync(path.join(dir, 'reputation_oracle.opt.wasm'), 'optimized');
    const inventory = buildArtifactInventory({ releaseDir: dir });

    assert.equal(inventory.wasm.exists, true);
    assert.equal(inventory.optimizedWasm.sizeBytes, 9);
    assert.equal(inventory.idl.exists, false);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('strips ansi escape sequences from cargo output', () => {
  assert.equal(stripAnsi('\u001b[1mwarning\u001b[0m: missing wasm-opt'), 'warning: missing wasm-opt');
});

test('resolves wasm-opt from explicit Binaryen environment locations', () => {
  assert.deepEqual(resolveWasmOptCommand({ WASM_OPT_PATH: '/opt/binaryen/bin/wasm-opt' }), {
    command: '/opt/binaryen/bin/wasm-opt',
    source: 'WASM_OPT_PATH',
  });
  assert.deepEqual(resolveWasmOptCommand({ WASM_OPT_BIN: '/toolcache/wasm-opt' }), {
    command: '/toolcache/wasm-opt',
    source: 'WASM_OPT_BIN',
  });
  assert.deepEqual(resolveWasmOptCommand({ BINARYEN_BIN: '/nix/store/binaryen/bin' }), {
    command: path.join('/nix/store/binaryen/bin', 'wasm-opt'),
    source: 'BINARYEN_BIN',
  });
  assert.deepEqual(resolveWasmOptCommand({}, { projectDir: '/tmp/no-local-binaryen' }), {
    command: 'wasm-opt',
    source: 'PATH',
  });
});

test('prefers project-local npm binaryen before global PATH and exposes it to cargo', () => {
  const dir = mkdtempSync(path.join(tmpdir(), 'deploy-wasm-local-binaryen-'));
  try {
    const binDir = path.join(dir, 'node_modules/.bin');
    const wasmOptPath = path.join(binDir, 'wasm-opt');
    mkdirSync(binDir, { recursive: true });
    writeFileSync(wasmOptPath, '#!/bin/sh\necho wasm-opt version 129\n');

    assert.deepEqual(resolveWasmOptCommand({}, { projectDir: dir }), {
      command: wasmOptPath,
      source: 'node_modules',
    });

    const cargoEnv = buildCargoEnv({ env: { PATH: '/usr/bin' }, projectDir: dir });
    assert.equal(cargoEnv.wasmOpt.command, wasmOptPath);
    assert.equal(cargoEnv.wasmOpt.source, 'node_modules');
    assert.equal(cargoEnv.env.PATH, `${binDir}${path.delimiter}/usr/bin`);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('detects wasm-opt through an injected runner and records command source', () => {
  const calls = [];
  const found = detectWasmOpt({
    env: { WASM_OPT_PATH: '/pinned/wasm-opt' },
    runner(command, args, options) {
      calls.push({ command, args, options });
      return { status: 0, stdout: 'wasm-opt version 123\n', stderr: '' };
    },
  });

  assert.deepEqual(calls, [{
    command: '/pinned/wasm-opt',
    args: ['--version'],
    options: { encoding: 'utf8' },
  }]);
  assert.equal(found.found, true);
  assert.equal(found.command, '/pinned/wasm-opt');
  assert.equal(found.source, 'WASM_OPT_PATH');
  assert.equal(found.version, 'wasm-opt version 123');

  const missing = detectWasmOpt({
    env: { BINARYEN_BIN: '/missing/bin' },
    runner(command) {
      return { status: 127, stdout: '', stderr: `${command}: not found` };
    },
  });

  assert.equal(missing.found, false);
  assert.equal(missing.command, path.join('/missing/bin', 'wasm-opt'));
  assert.equal(missing.source, 'BINARYEN_BIN');
  assert.match(missing.error, /not found/);
});

test('runs Sails cargo tests with the project-local wasm-opt path exposed', () => {
  const dir = mkdtempSync(path.join(tmpdir(), 'sails-test-local-binaryen-'));
  try {
    const binDir = path.join(dir, 'node_modules/.bin');
    const wasmOptPath = path.join(binDir, 'wasm-opt');
    mkdirSync(binDir, { recursive: true });
    writeFileSync(wasmOptPath, '#!/bin/sh\necho wasm-opt version 129\n');

    const calls = [];
    const result = runSailsCargoTests({
      args: ['test', '--test', 'gtest'],
      programDir: '/repo/programs/reputation-oracle',
      projectDir: dir,
      stdio: 'pipe',
      runner(command, args, options) {
        calls.push({ command, args, options });
        return { status: 0, stdout: 'ok\n', stderr: '' };
      },
    });

    assert.equal(result.status, 0);
    assert.equal(result.wasmOpt.command, wasmOptPath);
    assert.equal(result.wasmOpt.source, 'node_modules');
    assert.deepEqual(calls[0].args, ['test', '--test', 'gtest']);
    assert.equal(calls[0].options.cwd, '/repo/programs/reputation-oracle');
    assert.equal(calls[0].options.env.PATH.startsWith(`${binDir}${path.delimiter}`), true);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
