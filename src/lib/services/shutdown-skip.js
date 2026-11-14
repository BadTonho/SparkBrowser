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
