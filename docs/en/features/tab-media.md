# Tab media controls

On Windows, tabs that emit audio show a speaker button in both layouts. Click it to mute the entire tab, including embedded frames and Web Audio, without selecting it. Muted tabs keep the button visible. The setting follows the tab through navigation and hibernation during the current run; new, reopened and startup-restored tabs begin unmuted. Preview and unsupported engines do not show a functional mute control.

## Page volume

The volume button beside the address opens a compact 200 × 64 px popover directly beneath the button for the focused tab, including split view. It contains a horizontal slider, from 0–200% in 1% steps, with the percentage centered below it in 12 px text. The percentage updates immediately while dragging or using the arrow keys and follows the applied level when the player responds. The page stays visible without dimming or blur. Set the slider to 100% to restore the original level. Mute preserves the chosen level. New, reopened and startup-restored tabs begin at 100%; navigation, hibernation and workspace changes retain adjustments during the run.

The slider receives keyboard focus; arrow keys adjust the level. Escape returns focus to the button. Clicking the button again or outside the popover closes it and commits the latest adjustment. Tab closes it and returns focus to the button so normal keyboard navigation can continue. Changing the focused tab, document or workspace, or opening another browser overlay, also closes it. Resizing keeps it anchored within the window. Unsupported sources and preview show a disabled slider with its percentage below. The slider exposes its label and percentage to assistive technology; the visual percentage does not add a duplicate announcement.

100% preserves the player’s own level. Compatible HTML media can be amplified locally with Web Audio and an element capture stream, without recording or transmitting content. Unsupported players retain their original audio and limit amplification to 100%; sources without volume support keep native mute available through the tab speaker button. Autoplay restrictions can require interaction with the page. Outside an active drag, the slider follows the applied level.

The Rust backend tracks default execution contexts, including embedded frames, and binds fixed operations to the WebView generation and document. Slider updates coalesce and stale documents cannot receive a previous page’s adjustment. A transparent child WebView renders the popover above both native panes without hiding them or creating another operating-system window. Only its authenticated volume/dismiss actions reach the shell; application IPC remains restricted to the main shell. No media preference is written to SQLite or backups.

Run `--case media-volume` in the isolated Windows native suite for attenuation, measured amplified waveform, player-volume independence, coalescing, stale navigation, cross-origin fallback and restart defaults.

Players with native HTML controls retain 0–100% adjustment; amplification is unavailable to preserve their own mute controls. Direct player-level changes retain the tab multiplier.

## Floating video

The volume popover has no video controls. The native Picture-in-Picture services remain available to the internal media commands and native regression suite. Only the selected original video enters the native always-on-top Picture-in-Picture window; its playback position and page controls remain attached to the original tab. Only one floating video is active at a time. Protected/disabled players and policy-restricted frames are not bypassed.

Selecting the originating tab also selects its workspace. The internal exit command closes only the floating window. Switching tabs/workspaces and opening browser overlays retain it. Closing its tab or navigating to a new document ends it. A paused floating video still protects its source WebView from hibernation and organizer bulk closure; diagnostics explain that protection. Normal close/update review also includes a floating video.

Native state notifications are bound to the current execution context and document, including accessible cross-origin frames. The media bridge reports playback/PiP state only and grants no browser commands to external pages. Unsupported engines and preview cannot use the native media services.

Run `--case media-pip` for native floating-window selection, overlays, paused protection, return/exit, navigation, stale candidates, cross-origin frames and closing the source tab. All scenarios use generated media and isolated profiles.
