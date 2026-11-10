import { runScenario } from '../../tests/native/harness.mjs';
import { scenarios } from '../../tests/native/scenarios.mjs';
import { selectScenarios } from '../../tests/native/suites.mjs';
import { mkdirSync, writeFileSync } from 'node:fs';

const args = process.argv.slice(2);
if (args[0] === '--list') {
  console.log(Object.keys(scenarios).join('\n'));
} else {
  const selected = selectScenarios(args, scenarios);
  if (!selected.length || selected.some(name => !scenarios[name])) throw new Error('Use --list or --case <scenario>.');
  const failures = [];
  const results = [];
  for (const name of selected) {
    console.log(`Running native scenario: ${name}`);
    const started = Date.now();
    try {
      await runScenario(name, scenarios[name]);
      results.push({ name, status: 'passed', durationMs: Date.now() - started });
    } catch (error) {
      failures.push({ name, error });
      results.push({ name, status: 'failed', durationMs: Date.now() - started });
      console.error(`${name}: ${error.stack}`);
    }
    mkdirSync('.cache', { recursive: true });
    writeFileSync('.cache/native-results.json', JSON.stringify({ results }, null, 2));
  }
  if (failures.length) process.exitCode = 1;
}
