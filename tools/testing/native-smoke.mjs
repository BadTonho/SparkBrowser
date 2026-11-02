import { runScenario } from '../../tests/native/harness.mjs';
import { scenarios } from '../../tests/native/scenarios.mjs';

const args = process.argv.slice(2);
if (args[0] === '--list') {
  console.log(Object.keys(scenarios).join('\n'));
} else {
  const selected = args.length ? args[0] === '--case' && args.length === 2 ? [args[1]] : [] : Object.keys(scenarios);
  if (!selected.length || selected.some(name => !scenarios[name])) throw new Error('Use --list or --case <scenario>.');
  const failures = [];
  for (const name of selected) {
    console.log(`Running native scenario: ${name}`);
    try { await runScenario(name, scenarios[name]); } catch (error) { failures.push({ name, error }); console.error(`${name}: ${error.stack}`); }
  }
  if (failures.length) process.exitCode = 1;
}
