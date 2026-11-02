import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync, rmSync } from 'node:fs';
import { basename, dirname, join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import assert from 'node:assert/strict';
import { createManifest } from '../release/shared.mjs';

if (process.platform !== 'win32' || process.arch !== 'x64') throw new Error('Installer fixtures require Windows x64.');
const temporary = mkdtempSync(join(tmpdir(), 'veloce-updater-signing-'));
const key = join(temporary, 'fixture.key');
const cli = resolve('node_modules/@tauri-apps/cli/tauri.js');
const output = resolve('.cache/update-fixtures');
mkdirSync(output, { recursive: true });
try {
  const generated = spawnSync(process.execPath, [cli, 'signer', 'generate', '--ci', '--write-keys', key], { encoding: 'utf8', windowsHide: true });
  if (generated.error || generated.status !== 0) throw new Error('Could not generate the temporary test signing key.');
  const publicKey = readFileSync(key + '.pub', 'utf8').trim();
  const versions = ['0.1.1-beta.1', '0.1.1'];
  const configPath = resolve(output, 'build-config.json');
  for (const version of versions) {
    writeFileSync(configPath, JSON.stringify({ identifier: 'org.veloce.browser.updater-tests', productName: 'VeloceUpdateFixture', version,
      bundle: { targets: ['nsis'], createUpdaterArtifacts: true, windows: { nsis: { installMode: 'currentUser' } } },
      plugins: { updater: { pubkey: publicKey, requireSignedVersion: true, windows: { installMode: 'passive' } } },
    }));
    const result = spawnSync(process.execPath, [cli, 'build', '--bundles', 'nsis', '--config', configPath], { stdio: 'inherit', windowsHide: true,
      env: { ...process.env, CARGO_TARGET_DIR: resolve('.cache/native-target'), TAURI_SIGNING_PRIVATE_KEY: key, TAURI_SIGNING_PRIVATE_KEY_PASSWORD: '' } });
    if (result.error || result.status !== 0) throw new Error('Test installer build failed. Nothing was installed.');
    const bundles = resolve('.cache/native-target/release/bundle/nsis');
    const name = readdirSync(bundles).find(name => name.startsWith(`VeloceUpdateFixture_${version}_`) && name.endsWith('_x64-setup.exe'));
    if (!name) throw new Error('Test installer not found.');
    const destination = resolve(output, version);
    mkdirSync(destination, { recursive: true });
    const fileName = `VeloceBrowser_${version}_x64-setup.exe`;
    const signature = readFileSync(resolve(bundles, name + '.sig'), 'utf8').trim();
    copyFileSync(resolve(bundles, name), resolve(destination, fileName));
    writeFileSync(resolve(destination, fileName + '.sig'), signature + '\n');
    writeFileSync(resolve(destination, 'updater.json'), JSON.stringify(createManifest(`v${version}`, fileName, signature, 'Artificial isolated installer fixture'), null, 2));
  }
  writeFileSync(resolve(output, 'fixtures.json'), JSON.stringify({ identifier: 'org.veloce.browser.updater-tests', publicKey, versions }, null, 2));
  console.log('Two signed NSIS fixtures built. No installer was executed. Rebuild test:native:build before running the normal native suite.');
} finally {
  assert.equal(dirname(temporary), resolve(tmpdir()));
  assert.ok(basename(temporary).startsWith('veloce-updater-signing-'));
  rmSync(temporary, { recursive: true, force: true });
}
