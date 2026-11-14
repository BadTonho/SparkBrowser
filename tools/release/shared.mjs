import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { resolve } from 'node:path';

export const repository = 'BadTonho/SparkBrowser';
/**
 * Where the updater's private signing key lives. An explicit SPARK_SIGNING_KEY_PATH wins, then the
 * pre-rename VELOCE_SIGNING_KEY_PATH. Without either, ~/.spark-signing is used, except that an
 * existing key still sitting in ~/.veloce-signing keeps being used, so a rename never strands it.
 */
export function signingKeyPath(env = process.env, home = homedir(), exists = existsSync) {
  const explicit = env.SPARK_SIGNING_KEY_PATH || env.VELOCE_SIGNING_KEY_PATH;
  if (explicit) return resolve(explicit);
  const current = resolve(home, '.spark-signing', 'updater.key');
  const legacy = resolve(home, '.veloce-signing', 'updater.key');
  return !exists(current) && exists(legacy) ? legacy : current;
}
export function parseTag(tag) {
  const match = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-beta\.([1-9]\d*))?$/.exec(tag);
  if (!match || match[0] !== tag || match.slice(1, 4).some(number => BigInt(number) > 18446744073709551615n)) {
    throw new Error('Use vX.Y.Z or vX.Y.Z-beta.N (N >= 1).');
  }
  return { tag, version: tag.slice(1), prerelease: tag.includes('-beta.') };
}
export function synchronizeVersion(root, tag, write = true) {
  const { version } = parseTag(tag);
  const jsonPaths = ['package.json', 'package-lock.json', 'src-tauri/tauri.conf.json'];
  const files = jsonPaths.map(path => {
    const data = JSON.parse(readFileSync(resolve(root, path), 'utf8'));
    if (!write && path === 'package-lock.json' && data.packages?.['']?.version !== version) {
      throw new Error('Version files do not match the tag. Run release:version before committing and tagging.');
    }
    const previous = data.version;
    data.version = version;
    if (path === 'package-lock.json') data.packages[''].version = version;
    return { path, previous, text: JSON.stringify(data, null, 2) + '\n' };
  });
  for (const [path, pattern] of [
    ['src-tauri/Cargo.toml', /(\[package\][\s\S]*?\nversion = ")([^"]+)(")/],
    ['src-tauri/Cargo.lock', /(\[\[package\]\]\r?\nname = "spark-browser"\r?\nversion = ")([^"]+)(")/],
  ]) {
    const text = readFileSync(resolve(root, path), 'utf8');
    const match = text.match(pattern);
    if (!match) throw new Error(`Could not find the application version in ${path}`);
    files.push({ path, previous: match[2], text: text.replace(pattern, (_, before, _old, after) => before + version + after) });
  }
  if (!write && files.some(file => file.previous !== version)) throw new Error('Version files do not match the tag. Run release:version before committing and tagging.');
  if (write) for (const file of files) writeFileSync(resolve(root, file.path), file.text);
  return version;
}
export function createManifest(tag, fileName, signature, notes, date = new Date().toISOString()) {
  const { version } = parseTag(tag);
  const expected = `SparkBrowser_${version}_x64-setup.exe`;
  if (fileName !== expected || !signature.trim()) throw new Error('Installer name or signature does not match the release.');
  const comment = Buffer.from(signature.trim(), 'base64').toString('utf8');
  const trusted = comment.split(/\r?\n/).find(line => line.startsWith('trusted comment:'));
  if (!trusted?.split(/\s+/).includes(`version:${version}`)) throw new Error('The signature must be bound to the release version; use a current Tauri CLI.');
  return { version, notes, pub_date: date, platforms: { 'windows-x86_64': {
    signature: signature.trim(), url: `https://github.com/${repository}/releases/download/${tag}/${fileName}`,
  } } };
}
