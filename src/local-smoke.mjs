import { existsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { DEPLOY_OUTPUT_DIR, stripAnsi } from './deploy-wasm.mjs';

export const LOCAL_NETWORK = 'local';
export const LOCAL_ENDPOINT = 'ws://localhost:9944';
export const DEFAULT_WASM_PATH = path.join(DEPLOY_OUTPUT_DIR, 'reputation_oracle.opt.wasm');
export const DEFAULT_IDL_PATH = path.join(DEPLOY_OUTPUT_DIR, 'reputation_oracle.idl');

export function normalizeCliResult(result = {}) {
  return {
    status: result.status ?? (result.error ? 1 : 0),
    signal: result.signal ?? null,
    stdout: stripAnsi(result.stdout ?? ''),
    stderr: stripAnsi(result.stderr ?? ''),
    error: result.error?.message ?? null,
  };
}

export function runCli(command, args, { runner = spawnSync } = {}) {
  return normalizeCliResult(
    runner(command, args, {
      encoding: 'utf8',
      maxBuffer: 1024 * 1024 * 8,
    }),
  );
}

export function parseProgramId(output) {
  const text = String(output ?? '').trim();
  if (!text) return null;

  try {
    const parsed = JSON.parse(text);
    return parsed.programId ?? parsed.program_id ?? parsed.program?.id ?? null;
  } catch {
    const match = text.match(/programId["'\s:]+([A-Za-z0-9]+)/);
    return match?.[1] ?? null;
  }
}

export function artifactStatus({ wasmPath = DEFAULT_WASM_PATH, idlPath = DEFAULT_IDL_PATH } = {}) {
  const blockers = [];
  if (!existsSync(wasmPath)) blockers.push(`missing optimized Wasm artifact: ${wasmPath}`);
  if (!existsSync(idlPath)) blockers.push(`missing Sails IDL artifact: ${idlPath}`);

  return {
    ready: blockers.length === 0,
    blockers,
    wasmPath,
    idlPath,
  };
}

export function skip(reason, details = {}) {
  return {
    status: 'SKIP',
    reason,
    ...details,
  };
}

export function fail(reason, details = {}) {
  return {
    status: 'FAIL',
    reason,
    ...details,
  };
}

export function runLocalSmoke({
  account,
  env = process.env,
  execute = false,
  idlPath = DEFAULT_IDL_PATH,
  runner = spawnSync,
  walletCommand = env.VARA_WALLET_BIN || 'vara-wallet',
  wasmPath = DEFAULT_WASM_PATH,
} = {}) {
  const walletCheck = runCli(walletCommand, ['--help'], { runner });
  if (walletCheck.status !== 0) {
    return skip('vara-wallet CLI not found; install it before running local-node smoke', {
      walletCommand,
      error: walletCheck.error || walletCheck.stderr || walletCheck.stdout,
    });
  }

  const nodeInfo = runCli(walletCommand, ['--network', LOCAL_NETWORK, 'node', 'info'], { runner });
  if (nodeInfo.status !== 0) {
    return skip(`local Vara node unavailable at ${LOCAL_ENDPOINT}`, {
      walletCommand,
      network: LOCAL_NETWORK,
      endpoint: LOCAL_ENDPOINT,
      error: nodeInfo.error || nodeInfo.stderr || nodeInfo.stdout,
    });
  }

  const artifacts = artifactStatus({ wasmPath, idlPath });
  if (!artifacts.ready) {
    return fail('deploy artifacts are not ready; run npm run deploy:wasm first', {
      blockers: artifacts.blockers,
    });
  }

  if (!execute) {
    return skip('local node is reachable, but write smoke is disabled', {
      next: 'rerun with npm run smoke:local -- --execute and VARA_WALLET_ACCOUNT set to a funded local wallet account',
    });
  }

  const walletAccount = account || env.VARA_WALLET_ACCOUNT;
  if (!walletAccount) {
    return skip('VARA_WALLET_ACCOUNT is required for the write smoke path');
  }

  const upload = runCli(
    walletCommand,
    [
      '--network',
      LOCAL_NETWORK,
      '--account',
      walletAccount,
      'program',
      'upload',
      wasmPath,
      '--idl',
      idlPath,
      '--init',
      'Create',
      '--args',
      '[]',
    ],
    { runner },
  );
  if (upload.status !== 0) {
    return fail('local program upload failed', {
      error: upload.error || upload.stderr || upload.stdout,
    });
  }

  const programId = parseProgramId(upload.stdout);
  if (!programId) {
    return fail('local program upload did not return a programId', {
      output: upload.stdout || upload.stderr,
    });
  }

  const initialConfig = runCli(
    walletCommand,
    [
      '--network',
      LOCAL_NETWORK,
      'call',
      programId,
      'ReputationOracle/ExportMigrationConfig',
      '--args',
      '[]',
      '--idl',
      idlPath,
    ],
    { runner },
  );
  if (initialConfig.status !== 0) {
    return fail('post-deploy ExportMigrationConfig query failed', {
      programId,
      error: initialConfig.error || initialConfig.stderr || initialConfig.stdout,
    });
  }

  const setReadOnly = runCli(
    walletCommand,
    [
      '--network',
      LOCAL_NETWORK,
      '--account',
      walletAccount,
      'call',
      programId,
      'ReputationOracle/SetReadOnly',
      '--args',
      '[true]',
      '--idl',
      idlPath,
    ],
    { runner },
  );
  if (setReadOnly.status !== 0) {
    return fail('SetReadOnly local command failed', {
      programId,
      error: setReadOnly.error || setReadOnly.stderr || setReadOnly.stdout,
    });
  }

  const readOnlyConfig = runCli(
    walletCommand,
    [
      '--network',
      LOCAL_NETWORK,
      'call',
      programId,
      'ReputationOracle/ExportMigrationConfig',
      '--args',
      '[]',
      '--idl',
      idlPath,
    ],
    { runner },
  );
  if (readOnlyConfig.status !== 0) {
    return fail('read-only ExportMigrationConfig query failed', {
      programId,
      error: readOnlyConfig.error || readOnlyConfig.stderr || readOnlyConfig.stdout,
    });
  }

  return {
    status: 'PASS',
    network: LOCAL_NETWORK,
    endpoint: LOCAL_ENDPOINT,
    programId,
    checkedRoutes: [
      'ReputationOracle/ExportMigrationConfig',
      'ReputationOracle/SetReadOnly',
    ],
  };
}
