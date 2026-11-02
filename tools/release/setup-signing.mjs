import { mkdirSync, readFileSync, existsSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, resolve, relative, isAbsolute, sep } from 'node:path';
import { spawnSync } from 'node:child_process';

// Private material stays outside the checkout. Never print the signer's captured output.
const root = resolve('.');
const file = 'src-tauri/tauri.conf.json';
const config = JSON.parse(readFileSync(file, 'utf8'));
const key = resolve(process.env.VELOCE_SIGNING_KEY_PATH ?? resolve(homedir(), '.veloce-signing', 'updater.key'));
const inside = relative(root, key);
if (inside !== '..' && !inside.startsWith('..' + sep) && !isAbsolute(inside)) throw new Error('Choose a signing-key path outside the repository.');
if (config.plugins?.updater?.pubkey && !existsSync(key)) {
  throw new Error('This checkout already has a verification key. Restore its matching private key and .pub backup; do not generate a replacement.');
}
mkdirSync(dirname(key), { recursive: true, mode: 0o700 });
if (!existsSync(key)) {
  const result = spawnSync(process.execPath, [resolve('node_modules/@tauri-apps/cli/tauri.js'), 'signer', 'generate', '--ci', '--write-keys', key], {
    encoding: 'utf8', windowsHide: true,
  });
  if (result.error || result.status !== 0) throw new Error('Signing-key generation failed. No existing key was overwritten.');
}
const publicKey = readFileSync(key + '.pub', 'utf8').trim();
if (!/^[A-Za-z0-9+/=]+$/.test(publicKey)) throw new Error('Invalid public signing key.');
const previous = config.plugins?.updater?.pubkey;
if (previous && previous !== publicKey) throw new Error('A different public key is already configured. Key rotation needs a separate migration.');
config.plugins ??= {};
config.plugins.updater = { pubkey: publicKey, requireSignedVersion: true, windows: { installMode: 'passive' } };
writeFileSync(file, JSON.stringify(config, null, 2) + '\n');
console.log('Public verification key configured. The existing/private key remains outside the repository.');
console.log('Back up the signing-key directory securely before publishing the first release.');
