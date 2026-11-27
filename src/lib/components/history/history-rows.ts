/**
 * How the history panel lays out visits: which day each one belongs to (today, yesterday or a date),
 * the site shown after the title, the key of its icon, and a mark where more than half an hour passed
 * since the newer visit of the same day. Visits arrive newest first.
 */
export const GAP_SECONDS = 30 * 60;

export interface HistoryRowInput { id: number; title: string; url: string; visited_at: number }

export interface HistoryRow {
  id: number;
  title: string;
  url: string;
  /** The site shown after the title, without `www.`. */
  host: string;
  /** Key of the site's icon, or `null` when the page has none (such as an internal page). */
  site: string | null;
  /** More than half an hour passed since the newer visit of the same day. */
  gap: boolean;
  visitedAt: number;
  /** The local day, so visits of one day share one heading. */
  dayKey: string;
  relative: "today" | "yesterday" | null;
}

function dayKey(ms: number): string {
  const date = new Date(ms);
  return `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`;
}

function hostOf(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return "";
  }
}

export function historyRows(entries: HistoryRowInput[], nowMs: number, keyOf: (url: string) => string | null): HistoryRow[] {
  const today = dayKey(nowMs);
  const before = new Date(nowMs);
  before.setDate(before.getDate() - 1);
  const yesterday = dayKey(before.getTime());
  let previous: HistoryRow | null = null;
  return entries.map((entry) => {
    const key = dayKey(entry.visited_at * 1000);
    const row: HistoryRow = {
      id: entry.id,
      title: entry.title,
      url: entry.url,
      host: hostOf(entry.url),
      site: keyOf(entry.url),
      gap: previous !== null && previous.dayKey === key && previous.visitedAt - entry.visited_at > GAP_SECONDS,
      visitedAt: entry.visited_at,
      dayKey: key,
      relative: key === today ? "today" : key === yesterday ? "yesterday" : null,
    };
    previous = row;
    return row;
  });
}
