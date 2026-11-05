import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createManifest, parseTag } from './shared.mjs';

const tag = process.argv[2] || process.env.GITHUB_REF_NAME;
if (!tag) throw new Error('Release tag required (e.g. v0.1.0).');
const { version } = parseTag(tag);

const possibleDirs = [
  resolve('src-tauri/target/release/bundle/nsis'),
  resolve('target/release/bundle/nsis'),
  resolve('.cache/release-target/release/bundle/nsis'),
];

const bundlesDir = possibleDirs.find(dir => existsSync(dir));
if (!bundlesDir) {
  throw new Error(`Could not find nsis bundle directory in: ${possibleDirs.join(', ')}`);
}

const installers = readdirSync(bundlesDir).filter(name => name.endsWith('.exe') && !name.endsWith('.sig'));
if (installers.length === 0) {
  throw new Error(`No .exe installers found in ${bundlesDir}`);
}

const installerName = installers.find(name => name.includes(version)) || installers[0];
const installerPath = resolve(bundlesDir, installerName);
const sigPath = installerPath + '.sig';

if (!existsSync(sigPath)) {
  throw new Error(`Signature file not found: ${sigPath}. Ensure TAURI_SIGNING_PRIVATE_KEY was configured during build.`);
}

const signature = readFileSync(sigPath, 'utf8').trim();
const targetFileName = `VeloceBrowser_${version}_x64-setup.exe`;
const notes = process.env.RELEASE_NOTES || `Veloce Browser ${version}`;

const manifest = createManifest(tag, targetFileName, signature, notes);

const outputDir = resolve('release-artifacts');
mkdirSync(outputDir, { recursive: true });

copyFileSync(installerPath, resolve(outputDir, targetFileName));
copyFileSync(sigPath, resolve(outputDir, targetFileName + '.sig'));
writeFileSync(resolve(outputDir, 'updater.json'), JSON.stringify(manifest, null, 2) + '\n');

console.log(`Generated release artifacts in ${outputDir}:`);
console.log(`- ${targetFileName}`);
console.log(`- ${targetFileName}.sig`);
console.log(`- updater.json`);

