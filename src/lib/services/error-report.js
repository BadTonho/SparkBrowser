/** Where "Send error" opens: the new-issue page of the project repository. */
export const ISSUES_URL = 'https://github.com/BadTonho/SparkBrowser/issues/new';
/** Browsers and GitHub reject very long addresses, so the prefilled issue stays below this. */
const MAX_URL_LENGTH = 7000;
const CUT_NOTE = '\n… (cut; the full text is in spark-errors.log)';

/**
 * @typedef {{ id: string, kind: string, summary: string, details: string, version: string, os: string, time: string }} ErrorReport
 * @param {ErrorReport} report
 * @returns {string} A GitHub "new issue" address already filled in. The user reviews it and
 * decides whether to submit, so nothing leaves the computer without a click on GitHub itself.
 */
export function buildIssueUrl(report) {
  const title = `[${report.kind}] ${report.summary}`.replace(/\s+/g, ' ').slice(0, 100);
  const urlFor = (/** @type {string} */ details) => {
    const body = [
      `**Spark version:** ${report.version}`, `**System:** ${report.os}`,
      `**Time (UTC):** ${report.time}`, `**Type:** ${report.kind}`, '',
      '**What happened**', report.summary, '', '**Details**', '```', details, '```', '',
      '**What were you doing when it happened?**', '', '',
    ].join('\n');
    return `${ISSUES_URL}?${new URLSearchParams({ title, body })}`;
  };
  let details = report.details ?? '';
  let cut = false;
  while (urlFor(details + (cut ? CUT_NOTE : '')).length > MAX_URL_LENGTH && details.length > 0) {
    details = details.slice(0, Math.floor(details.length * 0.8));
    cut = true;
  }
  return urlFor(details + (cut ? CUT_NOTE : ''));
}
