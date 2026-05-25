import { spawnSync } from 'node:child_process';
import { buildCargoEnv, PROGRAM_DIR, stripAnsi } from './deploy-wasm.mjs';

export function runSailsCargoTests({
  args = ['test'],
  programDir = PROGRAM_DIR,
  projectDir = process.cwd(),
  runner = spawnSync,
  stdio = 'inherit',
} = {}) {
  const cargoEnv = buildCargoEnv({ projectDir });
  const result = runner('cargo', args, {
    cwd: programDir,
    env: cargoEnv.env,
    encoding: stdio === 'pipe' ? 'utf8' : undefined,
    stdio,
  });

  return {
    status: result.status ?? 1,
    signal: result.signal ?? null,
    wasmOpt: cargoEnv.wasmOpt,
    stdout: stripAnsi(result.stdout ?? ''),
    stderr: stripAnsi(result.stderr ?? ''),
    error: result.error?.message ?? null,
  };
}
