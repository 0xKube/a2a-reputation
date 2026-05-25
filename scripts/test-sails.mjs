#!/usr/bin/env node
import { runSailsCargoTests } from '../src/sails-test.mjs';

const args = process.argv.slice(2);
const result = runSailsCargoTests({
  args: ['test', ...args],
});

if (result.error) {
  console.error(result.error);
}

if (result.status !== 0) {
  process.exitCode = result.status;
}
