# Tab media controls

On Windows, tabs that emit audio show a speaker button in both layouts. Click it to mute the entire tab, including embedded frames and Web Audio, without selecting it. Muted tabs keep the button visible. The setting follows the tab through navigation and hibernation during the current run; new, reopened and startup-restored tabs begin unmuted. Preview and unsupported engines do not show a functional mute control.

## Page volume

The media button beside the address opens a keyboard-accessible panel for the focused tab, including split view. Set 0–200% in 1% steps or restore 100%. Mute preserves the chosen level. New, reopened and startup-restored tabs begin at 100%; navigation, hibernation and workspace changes retain adjustments during the run.

100% preserves the player’s own level. Compatible HTML media can be amplified locally with Web Audio and an element capture stream, without recording or transmitting content. Unsupported players retain their original audio and limit amplification to 100%; sources without volume support keep native mute available. Autoplay restrictions can require interaction with the page. The displayed percentage is the applied level, and unavailable/pending operations are indicated.

The Rust backend tracks default execution contexts, including embedded frames, and binds fixed operations to the WebView generation and document. Slider updates coalesce and stale documents cannot receive a previous page’s adjustment. No media preference is written to SQLite or backups.

Run `--case media-volume` in the isolated Windows native suite for attenuation, measured amplified waveform, player-volume independence, coalescing, stale navigation, cross-origin fallback and restart defaults.

Players with native HTML controls retain 0–100% adjustment; amplification is unavailable to preserve their own mute controls. Direct player-level changes retain the tab multiplier.

## Floating video

Open the page media panel and select **Open floating video**. Only the original video enters the native always-on-top Picture-in-Picture window; its playback position and page controls remain attached to the original tab. If several eligible videos exist, choose one explicitly. Only one floating video is active at a time. Protected/disabled players and policy-restricted frames are not bypassed.

**Return to tab** selects the originating tab and workspace. **Exit floating mode** closes only the floating window. Switching tabs/workspaces and opening browser overlays retain it. Closing its tab or navigating to a new document ends it. A paused floating video still protects its source WebView from hibernation and organizer bulk closure; diagnostics explain that protection. Normal close/update review also includes a floating video.

Native state notifications are bound to the current execution context and document, including accessible cross-origin frames. The media bridge reports playback/PiP state only and grants no browser commands to external pages. On unsupported engines or preview, unavailable controls are indicated.

Run `--case media-pip` for native floating-window selection, overlays, paused protection, return/exit, navigation, stale candidates, cross-origin frames and closing the source tab. All scenarios use generated media and isolated profiles.
