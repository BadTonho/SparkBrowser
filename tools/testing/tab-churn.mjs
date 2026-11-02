import { runScenario } from '../../tests/native/harness.mjs';
import { checkTabChurn } from '../../tests/native/tab-churn.mjs';

const minutes = Number(process.argv[2] ?? 10);
if (process.argv.length > 3 || !Number.isInteger(minutes) || minutes < 1 || minutes > 60) throw new Error('Usage: node tools/testing/tab-churn.mjs [minutes: 1..60]');
await runScenario('tab-churn-extended', context => checkTabChurn(context, { minutes }));
