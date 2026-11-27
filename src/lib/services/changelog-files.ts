/// <reference types="vite/client" />
// The release notes shipped with the app, from the root `changelog/` folder. Each file is its own
// small chunk, read only when the Changelog section is opened: bundling them all made the settings
// window larger with every release.
import type { ChangelogLanguage } from "./changelog";

const loaders = import.meta.glob<string>("/changelog/*/*.md", { query: "?raw", import: "default" });

/**
 * The release notes in `language`, by file path. A version without that translation gets its
 * English notes, as `changelogEntries` expects.
 */
export async function loadChangelogFiles(language: ChangelogLanguage): Promise<Record<string, string>> {
  const versionOf = (path: string) => /\/changelog\/([^/]+)\//.exec(path)?.[1];
  const translated = new Set(Object.keys(loaders).filter(path => path.endsWith(`/${language}.md`)).map(versionOf));
  const wanted = Object.keys(loaders).filter(path =>
    path.endsWith(`/${language}.md`) || (path.endsWith("/en.md") && !translated.has(versionOf(path))));
  const texts = await Promise.all(wanted.map(path => loaders[path]()));
  return Object.fromEntries(wanted.map((path, index) => [path, texts[index]]));
}
