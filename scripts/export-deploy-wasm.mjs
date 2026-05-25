#!/usr/bin/env node
import { exportDeployArtifacts } from '../src/deploy-wasm.mjs';

const args = new Set(process.argv.slice(2));

try {
  const summary = await exportDeployArtifacts({
    skipBuild: args.has('--skip-build'),
  });

  console.log(JSON.stringify(summary, null, 2));
  if (!summary.deployReady) {
    console.error(
      [
        'Deployable optimized Wasm is not ready.',
        ...summary.blockers.map((blocker) => `- ${blocker}`),
        '',
        'Run npm install to use the project-local binaryen dev dependency, install Binaryen so wasm-opt is on PATH, or set WASM_OPT_PATH/WASM_OPT_BIN/BINARYEN_BIN to a pinned Binaryen install, then rerun npm run deploy:wasm.',
        'Typical system package names: binaryen (apt/brew), or a CI image that already includes wasm-opt.',
      ].join('\n'),
    );
    process.exitCode = 1;
  }
} catch (error) {
  console.error(error.stack || error.message);
  process.exitCode = 1;
}
