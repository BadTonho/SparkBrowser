# Tab media controls

On Windows, tabs that emit audio show a speaker button in both layouts. Click it to mute the entire tab, including embedded frames and Web Audio, without selecting it. Muted tabs keep the button visible. The setting follows the tab through navigation and hibernation during the current run; new, reopened and startup-restored tabs begin unmuted. Preview and unsupported engines do not show a functional mute control.

## Page volume

The media button beside the address opens a keyboard-accessible panel for the focused tab, including split view. Set 0–200% in 1% steps or restore 100%. Mute preserves the chosen level. New, reopened and startup-restored tabs begin at 100%; navigation, hibernation and workspace changes retain adjustments during the run.

100% preserves the player’s own level. Compatible HTML media can be amplified locally with Web Audio and an element capture stream, without recording or transmitting content. Unsupported players retain their original audio and limit amplification to 100%; sources without volume support keep native mute available. Autoplay restrictions can require interaction with the page. The displayed percentage is the applied level, and unavailable/pending operations are indicated.

The Rust backend tracks default execution contexts, including embedded frames, and binds fixed operations to the WebView generation and document. Slider updates coalesce and stale documents cannot receive a previous page’s adjustment. No media preference is written to SQLite or backups.

Run `--case media-volume` in the isolated Windows native suite for attenuation, measured amplified waveform, player-volume independence, coalescing, stale navigation, cross-origin fallback and restart defaults.
