import { writable } from "svelte/store";

export type ErrorLevel = "info" | "warning" | "error" | "critical";

export interface ErrorReport {
  id: string;
  level: ErrorLevel;
  kind: string;
  summary: string;
  details: string;
  version: string;
  os: string;
  time: string;
}

/** Newest first; the notification shows a few and the rest wait until one is dismissed. */
export const errorReports = writable<ErrorReport[]>([]);

export function addErrorReport(report: ErrorReport): void {
  errorReports.update((list) => (list.some((item) => item.id === report.id) ? list : [report, ...list].slice(0, 20)));
}

export function dismissErrorReport(id: string): void {
  errorReports.update((list) => list.filter((item) => item.id !== id));
}
