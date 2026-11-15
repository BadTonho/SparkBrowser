import { existsSync } from 'node:fs';
import { resolve } from 'node:path';

export function nativeDriverPath(platform, explicit, exists = existsSync) {
  if (explicit) return resolve(explicit);
  if (platform !== 'win32') return undefined;
  const local = resolve('.cache/native-tools/edge/msedgedriver.exe');
  return exists(local) ? local : undefined;
}
