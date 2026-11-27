# Download video

[Português](../../pt-BR/features/video-download.md)

An optional toolbar button that saves videos from the loaded page. It is off by default: turn it on in **Settings** (the gear) → **Toolbar buttons** → **Download video**.

## Using it

The button opens a popover below it, above the page (which stays visible), that looks for videos in the active tab. It follows the chosen theme, palette and window style, like the other popovers. There is one card per video, with the ones on the screen first. Each card has a small preview (a frame of the video, or its poster picture when the site does not allow reading the frame), the length when the player knows it, and an **On screen** mark. Under it, each option shows its type (video file or HLS stream), the quality when known, the file name and the server; only options that can be saved are listed, best quality first. A video with nothing to save shows a short note with the reason instead. Press **Download**; progress, cancel and the saved file are in the [downloads panel](downloads.md). **Ask where to save each download** and the download folder setting apply. The suggested name comes from the page title.

If the list is empty, start the video and press **Search again**: many players only request the video after playback starts. Press **Esc**, click outside or click the button again to close it; it also closes when you switch tabs or the page changes address.

A video may have no preview when the site allows neither. Addresses only seen being requested are matched to a video by their folder; when that is not certain they get a card of their own.

## What is supported

| Source | Support |
| --- | --- |
| Direct video files (MP4, WebM and similar) | Yes, with pause, resume and checksum like any URL download |
| HLS streams (`.m3u8`) with MPEG-TS segments | Yes, saved as `.ts` (audio-only streams as `.aac` or `.mp3`) |
| HLS streams with fragmented MP4 segments | Yes, saved as `.mp4` |
| YouTube and other players that only use DASH or Media Source | No |
| DRM or any encrypted HLS, including AES-128 | No |
| Live streams | No |
| HLS with audio in a separate track, byte ranges or several initialization sections | No; see [open limitations](../roadmap/video-download-limitations.md) for separate audio |

Segments are joined without conversion; no external program is used. A `.ts` file plays in players such as VLC. Only download content you have the right to save.

## Limitations

- Site cookies are not sent, so videos that need you to be signed in can fail. The request sends a `Referer` the way browsers do: other servers only learn the page's origin.
- Streams cannot be paused. An interrupted or paused stream (for example after closing Spark) starts over when resumed. The total size is unknown until it finishes.
- A stream is limited to 10,000 segments, and its playlists to 2 MB.
- Playlists requested inside Web Workers, and videos inside closed web components, are not found. Only 8 playlists per search are read.
- A search waits at most 10 seconds for the frames of the page, 20 seconds for the playlists (read at the same time; one that does not answer is shown as a server that did not answer) and 6 seconds for the poster pictures (at most 12, requested at the same time). A video without a picture in time shows no preview.
- On Linux, only the top frame of the page is examined, so players inside embedded frames are not found.

## Privacy and security

While a page is open, Spark keeps in that page's memory the addresses of the last 32 `.m3u8` playlists it requested; nothing is saved or sent. The page is only examined when you open the panel: Spark asks it for the addresses it already uses (video elements, including those inside open web components, and `.m3u8` requests) and reads at most 8 playlists to list qualities. Every address, redirect and segment must be HTTP(S) without embedded credentials and passes the privacy shield. The two commands are available to the local interface only.

## Code

- `src-tauri/src/media/controller.js`: the `sources` action.
- `src-tauri/src/video_download/`: the search and the HLS playlist reader.
- `src-tauri/src/downloads/hls.rs`: the stream download.
- `src/lib/components/video-download/`: the panel and its style.
- `src-tauri/src/window/video_download_popover/`: its native popover.
