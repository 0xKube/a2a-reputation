import { existsSync, mkdirSync, statSync, writeFileSync } from 'node:fs';
import { copyFile } from 'node:fs/promises';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

export const PROGRAM_DIR = path.resolve('programs/reputation-oracle');
export const RELEASE_DIR = path.join(PROGRAM_DIR, 'target/wasm32-gear/release');
export const DEPLOY_OUTPUT_DIR = path.resolve('.outputs/deploy');
export const WASM_OPT_WARNING = 'wasm-opt optimizations error';

export function stripAnsi(value) {
  return String(value ?? '').replace(/\u001b\[[0-9;]*m/g, '');
}

export function resolveWasmOptCommand(env = process.env, { projectDir = process.cwd() } = {}) {
  const explicitPath = env.WASM_OPT_PATH || env.WASM_OPT_BIN;
  if (explicitPath) {
    return {
      command: explicitPath,
      source: env.WASM_OPT_PATH ? 'WASM_OPT_PATH' : 'WASM_OPT_BIN',
    };
  }

  if (env.BINARYEN_BIN) {
    return {
      command: path.join(env.BINARYEN_BIN, 'wasm-opt'),
      source: 'BINARYEN_BIN',
    };
  }

  const localCommand = path.join(projectDir, 'node_modules/.bin/wasm-opt');
  if (existsSync(localCommand)) {
    return {
      command: localCommand,
      source: 'node_modules',
    };
  }

  return {
    command: 'wasm-opt',
    source: 'PATH',
  };
}

export function detectWasmOpt({ env = process.env, projectDir = process.cwd(), runner = spawnSync } = {}) {
  const resolved = resolveWasmOptCommand(env, { projectDir });
  const result = runner(resolved.command, ['--version'], { encoding: 'utf8' });
  if (result.status !== 0) {
    return {
      found: false,
      command: resolved.command,
      source: resolved.source,
      version: null,
      error: stripAnsi(result.stderr || result.stdout || result.error?.message || `${resolved.command} not found`),
    };
  }

  return {
    found: true,
    command: resolved.command,
    source: resolved.source,
    version: stripAnsi(result.stdout || result.stderr).trim(),
    error: null,
  };
}

export function buildCargoEnv({ env = process.env, projectDir = process.cwd() } = {}) {
  const wasmOpt = resolveWasmOptCommand(env, { projectDir });
  if (wasmOpt.source !== 'node_modules') {
    return { env, wasmOpt };
  }

  const binDir = path.dirname(wasmOpt.command);
  return {
    env: {
      ...env,
      PATH: `${binDir}${path.delimiter}${env.PATH ?? ''}`,
    },
    wasmOpt,
  };
}

export function buildArtifactInventory({ releaseDir = RELEASE_DIR } = {}) {
  const artifactNames = {
    wasm: 'reputation_oracle.wasm',
    optimizedWasm: 'reputation_oracle.opt.wasm',
    idl: 'reputation_oracle.idl',
  };

  return Object.fromEntries(
    Object.entries(artifactNames).map(([key, fileName]) => {
      const artifactPath = path.join(releaseDir, fileName);
      if (!existsSync(artifactPath)) {
        return [key, { path: artifactPath, exists: false, sizeBytes: 0 }];
      }

      return [
        key,
        {
          path: artifactPath,
          exists: true,
          sizeBytes: statSync(artifactPath).size,
        },
      ];
    }),
  );
}

export function assessDeployReadiness({
  artifacts,
  buildOutput = '',
  wasmOpt = { found: false, version: null },
} = {}) {
  const output = stripAnsi(buildOutput);
  const checks = {
    wasmExists: Boolean(artifacts?.wasm?.exists),
    optimizedWasmExists: Boolean(artifacts?.optimizedWasm?.exists),
    idlExists: Boolean(artifacts?.idl?.exists),
    wasmOptFound: Boolean(wasmOpt?.found),
    buildHadWasmOptWarning: output.includes(WASM_OPT_WARNING),
  };
  checks.optimizedWasmNonEmpty = checks.optimizedWasmExists && artifacts.optimizedWasm.sizeBytes > 0;

  const blockers = [];
  if (!checks.wasmExists) blockers.push('missing base Wasm artifact');
  if (!checks.optimizedWasmExists) blockers.push('missing optimized Wasm artifact');
  if (!checks.optimizedWasmNonEmpty) blockers.push('optimized Wasm artifact is empty');
  if (!checks.idlExists) blockers.push('missing Sails IDL artifact');
  if (!checks.wasmOptFound) blockers.push('Binaryen wasm-opt is not on PATH');
  if (checks.buildHadWasmOptWarning) blockers.push('Sails/Gear build reported wasm-opt optimization did not run');

  return {
    deployReady: blockers.length === 0,
    blockers,
    checks,
    wasmOpt,
    artifacts,
  };
}

export async function exportDeployArtifacts({
  programDir = PROGRAM_DIR,
  releaseDir = RELEASE_DIR,
  outputDir = DEPLOY_OUTPUT_DIR,
  projectDir = process.cwd(),
  skipBuild = false,
  writeSummary = true,
} = {}) {
  let buildOutput = '';
  let buildStatus = 0;
  const cargoEnv = buildCargoEnv({ projectDir });
  if (!skipBuild) {
    const result = spawnSync('cargo', ['build', '--release'], {
      cwd: programDir,
      env: cargoEnv.env,
      encoding: 'utf8',
      maxBuffer: 1024 * 1024 * 16,
    });
    buildStatus = result.status ?? 1;
    buildOutput = `${result.stdout ?? ''}${result.stderr ?? ''}`;
    if (buildStatus !== 0) {
      throw new Error(`cargo build --release failed with status ${buildStatus}\n${stripAnsi(buildOutput)}`);
    }
  }

  const summary = assessDeployReadiness({
    artifacts: buildArtifactInventory({ releaseDir }),
    buildOutput,
    wasmOpt: detectWasmOpt({ projectDir }),
  });

  mkdirSync(outputDir, { recursive: true });
  if (writeSummary) {
    writeFileSync(path.join(outputDir, 'preflight.json'), `${JSON.stringify(summary, null, 2)}\n`);
  }

  if (summary.deployReady) {
    await copyFile(summary.artifacts.optimizedWasm.path, path.join(outputDir, 'reputation_oracle.opt.wasm'));
    await copyFile(summary.artifacts.idl.path, path.join(outputDir, 'reputation_oracle.idl'));
  }

  return {
    ...summary,
    buildStatus,
    outputDir,
  };
}
