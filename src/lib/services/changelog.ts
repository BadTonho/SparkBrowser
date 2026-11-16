/** A piece of a changelog line: plain text, **bold** or `code`. Rendered as elements, never as HTML. */
export interface ChangelogSpan { text: string; bold?: boolean; code?: boolean }
export interface ChangelogSection { heading: string; items: ChangelogSpan[][] }
export interface ChangelogEntry {
  version: string;
  title: string;
  /** Lines between the title and the first section, such as "In progress — not released." */
  note: string;
  sections: ChangelogSection[];
}
export type ChangelogLanguage = "en" | "pt-BR";

/** Splits `**bold**` and `` `code` `` out of a line; anything else stays literal text. */
export function parseInline(line: string): ChangelogSpan[] {
  const spans: ChangelogSpan[] = [];
  const pattern = /\*\*([^*]+)\*\*|`([^`]+)`/g;
  let last = 0;
  for (const match of line.matchAll(pattern)) {
    if (match.index > last) spans.push({ text: line.slice(last, match.index) });
    spans.push(match[1] !== undefined ? { text: match[1], bold: true } : { text: match[2], code: true });
    last = match.index + match[0].length;
  }
  if (last < line.length) spans.push({ text: line.slice(last) });
  return spans;
}

/** Reads the changelog format described in docs/en/contributing/changelog.md: a title, notes, `##` sections of `-` items. */
export function parseChangelog(markdown: string, version: string): ChangelogEntry {
  const entry: ChangelogEntry = { version, title: `Spark ${version}`, note: "", sections: [] };
  const notes: string[] = [];
  let section: ChangelogSection | null = null;
  let item: string[] | null = null;
  const flush = () => { if (section && item) section.items.push(parseInline(item.join(" "))); item = null; };
  for (const raw of markdown.replace(/\r\n?/g, "\n").split("\n")) {
    const line = raw.trimEnd();
    const trimmed = line.trim();
    if (trimmed.startsWith("# ")) { flush(); entry.title = trimmed.slice(2).trim() || entry.title; continue; }
    if (trimmed.startsWith("## ")) { flush(); section = { heading: trimmed.slice(3).trim(), items: [] }; entry.sections.push(section); continue; }
    if (/^[-*]\s+/.test(trimmed) && section) { flush(); item = [trimmed.replace(/^[-*]\s+/, "")]; continue; }
    if (!trimmed) { flush(); continue; }
    if (section && item && /^\s+/.test(line)) item.push(trimmed);
    else if (!section) notes.push(trimmed);
  }
  flush();
  entry.note = notes.join(" ");
  return entry;
}

/** Numeric parts first, then a release without a pre-release tag ranks above one with it (1.0.0 > 1.0.0-beta.1). */
export function compareVersions(a: string, b: string): number {
  const split = (value: string) => {
    const [main, pre = ""] = value.split("-", 2);
    return { numbers: main.split(".").map(part => Number.parseInt(part, 10) || 0), pre };
  };
  const left = split(a), right = split(b);
  for (let index = 0; index < Math.max(left.numbers.length, right.numbers.length); index++) {
    const difference = (left.numbers[index] ?? 0) - (right.numbers[index] ?? 0);
    if (difference) return difference;
  }
  if (left.pre === right.pre) return 0;
  if (!left.pre) return 1;
  if (!right.pre) return -1;
  return left.pre < right.pre ? -1 : 1;
}

/**
 * Newest version first. A version without a file in the chosen language falls back to English.
 * @param files Raw markdown keyed by path, such as `/changelog/0.1.1/pt-BR.md`.
 */
export function changelogEntries(files: Record<string, string>, language: ChangelogLanguage): ChangelogEntry[] {
  const byVersion = new Map<string, Partial<Record<ChangelogLanguage, string>>>();
  for (const [path, markdown] of Object.entries(files)) {
    const match = /\/changelog\/([^/]+)\/(en|pt-BR)\.md$/.exec(path.replace(/\\/g, "/"));
    if (!match || typeof markdown !== "string") continue;
    byVersion.set(match[1], { ...byVersion.get(match[1]), [match[2] as ChangelogLanguage]: markdown });
  }
  return [...byVersion.entries()]
    .sort(([a], [b]) => compareVersions(b, a))
    .map(([version, texts]) => parseChangelog(texts[language] ?? texts.en ?? "", version))
    .filter(entry => entry.sections.length > 0);
}
