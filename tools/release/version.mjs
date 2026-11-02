import { synchronizeVersion } from './shared.mjs';
const tag = process.argv[2];
if (!tag || process.argv.length !== 3) throw new Error('Usage: npm run release:version -- vX.Y.Z[-beta.N]');
console.log(`Application versions synchronized to ${synchronizeVersion('.', tag)}. Review, commit and create the matching tag locally.`);
