# Spark 0.1.1

In progress — not released.

## New features

- Add an optional audio-reactive background with 20 visual styles (bars, waves, circular shapes, particles, glows, aurora, 3D terrain and more), custom colors and visual intensity, driven by normal browser-tab audio.

- Add a YouTube now-playing widget for the YouTube and YouTube Music tabs you already have open, with thumbnail, a single scrolling title line, a progress bar and playback controls, a thumbnail that is either small or covers the whole card, an optional muted video preview, and a compact version of the card at the bottom of the vertical tabs sidebar; Spotify and YouTube now share a single **Music and video** entry in the new-tab customization menu.

- Add a Spotify new-tab widget with account connection, playback information and controls for the active device, using the same minimal card as YouTube: logo that opens Spotify, a single scrolling title line, a progress bar and a small or full-card cover.

- Choose whether the new-tab customization panel opens as a compact corner panel or as a wide one centered on the screen with the categories as blocks.

- Choose whether browser settings open in a pop-up or in a tab.

## Improvements
- Settings open and close with a subtle animation that respects reduced-motion preferences.
- Tab dragging groups pointer updates to reduce repeated work.
- Idle settings, tools and downloads panels release their native views after use.
- Local diagnostics show recent native timing percentiles and separate tab-switch wait and execution times.

## Fixes

- Fixed the update channel selector being disabled in native builds where automatic updates are unavailable.
