# Audio-reactive background

[Português](../../pt-BR/features/audio-background.md)

Open **Customize new tab → Wallpaper → Audio equalizer** and enable **Show audio-reactive visualizer**. The effect starts disabled. Choose a visual style, a color and a visual intensity from 0.5 to 2.0; these controls never change playback volume.

The visualizer appears over your current solid color, image or video background, behind the page controls. Band styles stay within the lower quarter of the new tab; full-screen styles use the whole page. It reacts to the actual audio spectrum, including Web Audio, from normal browser tabs across windows and workspaces in the same profile. The visual fades out in roughly half a second of silence. Reduced-motion mode uses discrete, less frequent updates instead of smooth transitions.

## Visual styles

All styles are drawn from the same 32 real frequency levels and use the chosen color. **Bars** is the default.

| Group | Styles |
|---|---|
| Lines and waves | Bars, Wave, Mountain area, Stacked lines, Heartbeat line |
| Circular | Radial bars, Pulsing ring, Concentric rings, Spiral, Vinyl record |
| Dots and particles | LED dots, Particles, Bubbles, Fireflies |
| Ambient | Edge glow, Orbs, Spectrogram, Aurora |
| Geometric | Polygons, 3D terrain |

Bars, Wave, Mountain area, Stacked lines, Heartbeat line, LED dots and Spectrogram use the lower quarter of the page. Every other style uses the full page. The drawing code lives in `public/new-tab/audio-styles.js`; Bars is built into `audio-background.js`. The accepted style names are validated by the backend, and an unknown name is rejected.

## Platforms and capture

- **Windows:** requires build 20348 or later and a compatible WebView2 runtime. WASAPI Application Loopback captures only the relevant normal WebView2 browser-process trees, deduplicated across tabs. It never monitors the entire desktop or the Spark application's full process tree.
- **Linux:** requires `libpulse.so.0` and a running PulseAudio server or PipeWire's PulseAudio compatibility server. Debian-family packages recommend `libpulse0`; RPM packages recommend `pulseaudio-libs`. The library is loaded on demand, so its absence does not prevent Spark from starting. Each eligible playback stream is monitored individually. Its process ID and WebKit ancestry must match the normal Spark process; ambiguous streams are excluded.

The settings panel explains unavailable capture. Protected content may return silence. There is no simulated spectrum or attempt to bypass content protection. Audio playing in the Spotify desktop app, on a phone, or in another browser does not drive this effect; Spotify playing in a Spark tab can participate when the system provides capturable audio.

## Lifecycle and privacy

One native service shares capture and FFT analysis between visible new-tab pages. Capture starts only when the effect is enabled, a new tab is visible in a non-minimized window and a normal browser tab is audibly playing at nonzero volume. Capture stops when these conditions end. New audio streams and output-device changes are detected and retried automatically. Closing, navigating away from, hiding or disabling a consumer removes its rendering subscription.

The analyzer does not open a microphone, record a file, transmit audio, change playback routing or adjust volume. PCM exists only in bounded transient native buffers. The local new-tab document receives 32 normalized levels at no more than 20 updates per second through its authenticated, document-bound bridge. Old or malformed frames are ignored. The canvas stops animating once the levels have settled into silence and clears stale levels if updates stop.

Preferences are local to the device, synchronized across normal new tabs and restored after restart. Backups exclude them. Private windows start with the effect disabled and default preferences; their audio-process trees are not included in normal-profile capture.

## Validation

Unit tests cover silence, frequency ordering, amplitude changes, mixed tones, preference validation, Linux ABI layout/process filtering, consumer lifecycle, reduced motion and stale/replayed bridge messages. Linux capture sources also compile in Windows unit-test builds, without loading libpulse there. Run the isolated native `audio-background` scenario for actual process-loopback capture, the 440 Hz frequency peak, mute, zero volume, visibility, persistence and unchanged playback. Native Linux audio-server and protected-player checks require the corresponding environment.
