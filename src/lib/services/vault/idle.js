/**
 * "Away from the computer" for a password that is on screen. After this long without the person using
 * the window (or while the tab was out of sight, or the computer asleep), the password is closed and
 * has to be confirmed again.
 */
export const REVEAL_IDLE_MS = 5 * 60 * 1000;

/**
 * True when `limit` milliseconds or more passed since `lastActivity`. It compares wall-clock times, so
 * time spent asleep or in a throttled background tab counts.
 * @param {number} lastActivity
 * @param {number} now
 * @param {number} [limit]
 */
export function isIdle(lastActivity, now, limit = REVEAL_IDLE_MS) {
  return now - lastActivity >= limit;
}

// The Settings tab is a projection: what the person does there reaches the interface as click, mousedown,
// keydown, input and change events on the controls (pointer moves and scrolling are not forwarded).
const ACTIVITY_EVENTS = ['pointerdown', 'pointermove', 'mousedown', 'click', 'keydown', 'input', 'change', 'wheel', 'focus'];

/**
 * Calls `onexpire` once when the person has been away for the limit. Any activity first checks whether
 * the limit was already passed (a tab that was hidden for ten minutes expires at the moment it is shown
 * again, before the new activity could save it), and only then counts as activity.
 * Returns a function that stops watching.
 * @param {() => void} onexpire
 * @param {{ target?: EventTarget, doc?: EventTarget, now?: () => number, limit?: number, every?: number }} [options]
 * @returns {() => void}
 */
export function watchIdle(onexpire, options = {}) {
  const { target = window, doc = document, now = Date.now, limit = REVEAL_IDLE_MS, every = 10_000 } = options;
  let last = now();
  let done = false;
  const expire = () => {
    if (done) return;
    done = true;
    onexpire();
  };
  const activity = () => {
    if (isIdle(last, now(), limit)) expire();
    else last = now();
  };
  for (const name of ACTIVITY_EVENTS) target.addEventListener(name, activity, true);
  doc.addEventListener('visibilitychange', activity);
  const timer = setInterval(() => {
    if (isIdle(last, now(), limit)) expire();
  }, every);
  return () => {
    for (const name of ACTIVITY_EVENTS) target.removeEventListener(name, activity, true);
    doc.removeEventListener('visibilitychange', activity);
    clearInterval(timer);
  };
}
