/**
 * "Don't show again" only covers the lightweight parts of an exit review. Edited forms and page
 * downloads that cannot resume are lost for good, so they always keep the review on screen.
 * @param {{ tabs: { form: boolean, native_download: boolean }[], native_downloads: string[] } | null | undefined} review
 */
export function hasIrreversibleRisk(review) {
  return Boolean(review && (review.tabs.some(tab => tab.form || tab.native_download) || review.native_downloads.length));
}

/**
 * Whether a prepared review may be confirmed without asking. Updates and engine restarts share the
 * dialog but have different consequences, so the preference never applies to them.
 * @param {{ purpose: string, update_version: string | null, review: Parameters<typeof hasIrreversibleRisk>[0] }} state
 * @param {boolean} preference
 */
export function canSkipExitReview(state, preference) {
  return Boolean(preference && state.purpose === 'exit' && !state.update_version && state.review && !hasIrreversibleRisk(state.review));
}

/**
 * A review with nothing in it has nothing to ask.
 * @param {{ tabs: unknown[], downloads: unknown[], native_downloads: unknown[] } | null | undefined} review
 */
export function isEmptyReview(review) {
  return Boolean(review && !review.tabs.length && !review.downloads.length && !review.native_downloads.length);
}

/**
 * Whether the person has to decide something: a review that lists activity and is not covered by "don't
 * ask again". Everything else is confirmed automatically.
 * @param {{ purpose: string, update_version: string | null, review: (Parameters<typeof isEmptyReview>[0]) & Parameters<typeof hasIrreversibleRisk>[0] }} state
 * @param {boolean} preference
 */
export function reviewNeedsAttention(state, preference) {
  return Boolean(state.review && !isEmptyReview(state.review) && !canSkipExitReview(state, preference));
}

/**
 * What the shutdown dialog should show. `attention` needs a decision or reports an error and appears at
 * once; `working` is saving or committing and only appears if it takes long (see SHUTDOWN_REVEAL_DELAY_MS),
 * so a close with nothing to save never flashes a dialog.
 * @param {{ phase: string, purpose: string, update_version: string | null, review: Parameters<typeof reviewNeedsAttention>[0]['review'] }} state
 * @param {boolean} preference
 * @returns {'hidden' | 'attention' | 'working'}
 */
export function shutdownPresentation(state, preference) {
  if (state.phase === 'idle') return 'hidden';
  if (state.phase === 'error') return 'attention';
  if (state.phase === 'review' && reviewNeedsAttention(state, preference)) return 'attention';
  return 'working';
}

/** How long saving may take before the dialog is shown. Shorter work finishes without it. */
export const SHUTDOWN_REVEAL_DELAY_MS = 700;
