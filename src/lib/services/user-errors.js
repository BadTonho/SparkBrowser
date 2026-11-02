/** Stable codes travel separately from diagnostic messages. Never classify errors by English text. */
export class UserError extends Error {
  /** @param {string} code @param {unknown} detail */
  constructor(code, detail) { super(String(detail)); this.code = code; }
}
/** @param {unknown} error @param {string} [fallback] */
export function describeError(error, fallback = 'operation.failed') {
  const code = error instanceof UserError ? error.code : fallback;
  return JSON.stringify({ veloceError: true, code, detail: error instanceof Error ? error.message : String(error) });
}
/** @param {unknown} value */
export function errorDetails(value) {
  if (value instanceof UserError) return { code: value.code, detail: value.message };
  if (typeof value === 'string') {
    try {
      const parsed = JSON.parse(value);
      if (parsed?.veloceError === true && typeof parsed.code === 'string' && typeof parsed.detail === 'string') return { code: parsed.code, detail: parsed.detail };
    } catch { /* Native diagnostic text has no user-facing semantics. */ }
  }
  return { code: 'operation.failed', detail: String(value) };
}
