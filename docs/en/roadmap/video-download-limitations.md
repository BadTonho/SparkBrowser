# Video download: open limitations

[Português](../../pt-BR/roadmap/video-download-limitations.md)

Open points of [Download video](../features/video-download.md). Remove an item when it is solved.

## Streams with separate audio are not supported

**Status:** open, deliberately left out of the first version.

Some HLS streams deliver the picture and the sound in different playlists (`#EXT-X-MEDIA:TYPE=AUDIO` with its own address, referenced by the `AUDIO` attribute of each quality). The player joins them while playing. Spark only joins the segments of one playlist, so saving such a quality would give a video **without sound**. These qualities are therefore marked `video.separate_audio` and cannot be downloaded.

**What users see:** seen on Reddit (`v.redd.it`). When the same video also has an option that can be saved, such as a direct file, only that option is shown. When it has none, the card shows a note that explains why. A single file may have a lower quality than the blocked stream, and nothing in the panel says so.

**What supporting it needs:**

1. Download the audio playlist of the chosen quality as well as its video playlist.
2. Merge the two tracks into one playable file, either with a merger written in Rust (fragmented MP4 is the simpler case; MPEG-TS needs more work) or with an external program such as ffmpeg, which would have to be shipped or downloaded and kept up to date.
3. Check whether these streams also use byte ranges (`#EXT-X-BYTERANGE`), which are not supported either. This has not been verified for Reddit.

The code that reads the audio group is in `src-tauri/src/video_download/playlist.rs`.

## Some videos have no preview

**Status:** open, cause not investigated.

On Reddit, a card was shown with **No preview available**, although the player knew the video's length. Previews work on the local test page, both from a drawn frame and from a poster picture. A likely area to check is a video whose frame cannot be read and whose poster is stated somewhere the search does not look.
