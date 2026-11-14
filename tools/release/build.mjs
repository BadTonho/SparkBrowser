import { execFileSync, spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve, relative, isAbsolute, sep } from 'node:path';
import { createManifest, parseTag, signingKeyPath, synchronizeVersion } from './shared.mjs';
import { runValidation } from '../testing/validate.mjs';

if (process.platform !== 'win32' || process.arch !== 'x64') throw new Error('Release packaging currently supports Windows x64 only.');
const tag = process.argv[2];
const { version, prerelease } = parseTag(tag);
const notesIndex = process.argv.indexOf('--notes');
if (process.argv.length !== (notesIndex === -1 ? 3 : 5) || (notesIndex !== -1 && notesIndex !== 3)) throw new Error('Usage: npm run release:build -- vX.Y.Z[-beta.N] [--notes path/to/notes.md]');
const notes = notesIndex === -1 ? `Spark Browser ${version}` : readFileSync(process.argv[4], 'utf8');
synchronizeVersion('.', tag, false);
const git = (...args) => execFileSync('git', args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
if (git('rev-parse', `${tag}^{commit}`) !== git('rev-parse', 'HEAD')) throw new Error('The release tag must point to the current commit.');
if (git('status', '--porcelain')) throw new Error('Review and commit pending changes before packaging a tagged release.');
runValidation({ native: true });
if (git('status', '--porcelain')) throw new Error('Validation changed the checkout. Review the diff before packaging a tagged release.');
const keyPath = signingKeyPath();
const keyRelative = relative(resolve('.'), keyPath);
if (keyRelative !== '..' && !keyRelative.startsWith('..' + sep) && !isAbsolute(keyRelative)) throw new Error('The private signing key must stay outside the checkout.');
if (!existsSync(keyPath)) throw new Error('Run release:setup-signing first, or set SPARK_SIGNING_KEY_PATH to your existing private key.');
const config = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
if (readFileSync(keyPath + '.pub', 'utf8').trim() !== config.plugins?.updater?.pubkey) throw new Error('The signing key does not match the public key embedded in the app.');
const target = resolve('.cache/release-target');
const output = resolve('.cache/release-artifacts', tag);
mkdirSync(output, { recursive: true });
const buildConfig = resolve(output, 'build-config.json');
writeFileSync(buildConfig, JSON.stringify({ bundle: { targets: ['nsis'], createUpdaterArtifacts: true, windows: { nsis: { installMode: 'currentUser' } } } }));
const result = spawnSync(process.execPath, [resolve('node_modules/@tauri-apps/cli/tauri.js'), 'build', '--bundles', 'nsis', '--config', buildConfig], {
  stdio: 'inherit', windowsHide: true,
  env: { ...process.env, CARGO_TARGET_DIR: target, TAURI_SIGNING_PRIVATE_KEY: keyPath, TAURI_SIGNING_PRIVATE_KEY_PASSWORD: process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD ?? '' },
});
if (result.error || result.status !== 0) throw new Error('Signed release build failed. Nothing was published.');
const bundles = resolve(target, 'release/bundle/nsis');
const installers = readdirSync(bundles).filter(name => name.includes(`_${version}_`) && name.endsWith('_x64-setup.exe'));
if (installers.length !== 1) throw new Error('Expected exactly one installer for this version.');
const source = resolve(bundles, installers[0]);
const fileName = `SparkBrowser_${version}_x64-setup.exe`;
const signature = readFileSync(source + '.sig', 'utf8').trim();
const manifest = createManifest(tag, fileName, signature, notes);
copyFileSync(source, resolve(output, fileName));
writeFileSync(resolve(output, fileName + '.sig'), signature + '\n');
writeFileSync(resolve(output, 'updater.json'), JSON.stringify(manifest, null, 2) + '\n');
console.log(`Release prepared in .cache/release-artifacts/${tag}. Upload only the installer, its .sig and updater.json.`);
console.log(`Publish the GitHub release for ${tag} with Pre-release ${prerelease ? 'enabled' : 'disabled'}. No tags or files were pushed.`);
