//! The part of HLS (RFC 8216) needed to save an unprotected, finished stream.
use url::Url;

pub const MAX_PLAYLIST_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_SEGMENTS: usize = 10_000;

#[derive(Debug)]
pub struct Variant {
    pub url: Url,
    pub bandwidth: u64,
    pub height: u32,
    /// The sound lives in another playlist, which would need muxing.
    pub separate_audio: bool,
}
#[derive(Debug)]
pub struct MediaPlaylist {
    /// Initialization section of fragmented MP4. Without it the segments are MPEG-TS.
    pub init: Option<Url>,
    pub segments: Vec<Url>,
}
impl MediaPlaylist {
    pub fn extension(&self) -> &'static str {
        if self.init.is_some() {
            return "mp4";
        }
        // Audio-only streams use packed audio segments.
        match self.segments.first().map(|url| url.path()) {
            Some(path) if path.ends_with(".aac") => "aac",
            Some(path) if path.ends_with(".mp3") => "mp3",
            _ => "ts",
        }
    }
}
#[derive(Debug)]
pub enum Playlist {
    /// Best quality first.
    Master(Vec<Variant>),
    Media(MediaPlaylist),
}

/// Splits `KEY=value,KEY="quoted, value"`.
fn attributes(raw: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = raw;
    while let Some((key, tail)) = rest.split_once('=') {
        let (value, next) = if let Some(quoted) = tail.strip_prefix('"') {
            let (value, next) = quoted.split_once('"').unwrap_or((quoted, ""));
            (value, next.strip_prefix(',').unwrap_or(next))
        } else {
            tail.split_once(',').unwrap_or((tail, ""))
        };
        out.push((key.trim().to_string(), value.to_string()));
        rest = next;
    }
    out
}
fn attribute<'a>(list: &'a [(String, String)], key: &str) -> Option<&'a str> {
    list.iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}
fn resolve(base: &Url, reference: &str) -> Result<Url, &'static str> {
    let url = base.join(reference).map_err(|_| "video.invalid_playlist")?;
    if !matches!(url.scheme(), "http" | "https") {
        // `skd:` and `data:` references only appear in protected streams.
        return Err("video.encrypted");
    }
    Ok(url)
}
fn protected(tag_attributes: &str) -> bool {
    attribute(&attributes(tag_attributes), "METHOD") != Some("NONE")
}

pub fn parse(text: &str, base: &Url) -> Result<Playlist, &'static str> {
    let mut lines = text
        .trim_start_matches('\u{feff}')
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    if lines.next() != Some("#EXTM3U") {
        return Err("video.invalid_playlist");
    }
    let lines: Vec<_> = lines.collect();
    if lines
        .iter()
        .any(|line| line.starts_with("#EXT-X-STREAM-INF:"))
    {
        parse_master(&lines, base)
    } else {
        parse_media(&lines, base).map(Playlist::Media)
    }
}

fn parse_master(lines: &[&str], base: &Url) -> Result<Playlist, &'static str> {
    if lines.iter().any(|line| {
        line.strip_prefix("#EXT-X-SESSION-KEY:")
            .is_some_and(protected)
    }) {
        return Err("video.encrypted");
    }
    let audio_groups: Vec<String> = lines
        .iter()
        .filter_map(|line| line.strip_prefix("#EXT-X-MEDIA:"))
        .map(attributes)
        .filter(|list| attribute(list, "TYPE") == Some("AUDIO") && attribute(list, "URI").is_some())
        .filter_map(|list| attribute(&list, "GROUP-ID").map(str::to_string))
        .collect();
    let mut variants: Vec<Variant> = Vec::new();
    let mut pending = None;
    for line in lines {
        if let Some(raw) = line.strip_prefix("#EXT-X-STREAM-INF:") {
            pending = Some(attributes(raw));
        } else if !line.starts_with('#') {
            let Some(list) = pending.take() else {
                continue;
            };
            let url = resolve(base, line)?;
            if variants.iter().any(|variant| variant.url == url) {
                continue;
            }
            variants.push(Variant {
                url,
                bandwidth: attribute(&list, "BANDWIDTH")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                height: attribute(&list, "RESOLUTION")
                    .and_then(|v| v.split_once('x'))
                    .and_then(|(_, height)| height.parse().ok())
                    .unwrap_or(0),
                separate_audio: attribute(&list, "AUDIO")
                    .is_some_and(|group| audio_groups.iter().any(|g| g == group)),
            });
        }
    }
    if variants.is_empty() {
        return Err("video.invalid_playlist");
    }
    variants.sort_by_key(|v| std::cmp::Reverse((v.height, v.bandwidth)));
    variants.truncate(16);
    Ok(Playlist::Master(variants))
}

fn parse_media(lines: &[&str], base: &Url) -> Result<MediaPlaylist, &'static str> {
    let mut init: Option<Url> = None;
    let mut segments = Vec::new();
    let mut expects_segment = false;
    let mut finished = false;
    for line in lines {
        if line.strip_prefix("#EXT-X-KEY:").is_some_and(protected) {
            return Err("video.encrypted");
        } else if line.starts_with("#EXT-X-BYTERANGE:") {
            return Err("video.byterange");
        } else if let Some(raw) = line.strip_prefix("#EXT-X-MAP:") {
            let list = attributes(raw);
            if attribute(&list, "BYTERANGE").is_some() {
                return Err("video.byterange");
            }
            let url = resolve(
                base,
                attribute(&list, "URI").ok_or("video.invalid_playlist")?,
            )?;
            // A second initialization section cannot be joined into one file.
            if init.as_ref().is_some_and(|previous| previous != &url)
                || (init.is_none() && !segments.is_empty())
            {
                return Err("video.unsupported");
            }
            init = Some(url);
        } else if line.starts_with("#EXTINF:") {
            expects_segment = true;
        } else if *line == "#EXT-X-ENDLIST" {
            finished = true;
        } else if !line.starts_with('#') && expects_segment {
            if segments.len() >= MAX_SEGMENTS {
                return Err("video.too_many_segments");
            }
            segments.push(resolve(base, line)?);
            expects_segment = false;
        }
    }
    if !finished {
        return Err("video.live");
    }
    if segments.is_empty() {
        return Err("video.invalid_playlist");
    }
    Ok(MediaPlaylist { init, segments })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://media.example.test/path/index.m3u8?token=a").unwrap()
    }
    fn media(body: &str) -> Result<MediaPlaylist, &'static str> {
        match parse(&format!("#EXTM3U\n{body}"), &base())? {
            Playlist::Media(media) => Ok(media),
            Playlist::Master(_) => panic!("expected a media playlist"),
        }
    }

    #[test]
    fn resolves_segments_and_picks_the_container() {
        let ts =
            media("#EXTINF:4,\r\nseg0.ts\r\n#EXTINF:4,\r\n/other/seg1.ts\r\n#EXT-X-ENDLIST\r\n")
                .unwrap();
        assert_eq!(ts.extension(), "ts");
        assert_eq!(
            ts.segments[0].as_str(),
            "https://media.example.test/path/seg0.ts"
        );
        assert_eq!(
            ts.segments[1].as_str(),
            "https://media.example.test/other/seg1.ts"
        );
        let mp4 = media("#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\na.m4s\n#EXT-X-ENDLIST").unwrap();
        assert_eq!(mp4.extension(), "mp4");
        let clear = media("#EXT-X-KEY:METHOD=NONE\n#EXTINF:4,\na.aac\n#EXT-X-ENDLIST").unwrap();
        assert_eq!(clear.extension(), "aac");
    }

    #[test]
    fn refuses_what_cannot_be_saved() {
        for (body, code) in [
            ("#EXTINF:4,\na.ts", "video.live"),
            (
                "#EXT-X-KEY:METHOD=AES-128,URI=\"k\"\n#EXTINF:4,\na.ts\n#EXT-X-ENDLIST",
                "video.encrypted",
            ),
            (
                "#EXTINF:4,\n#EXT-X-BYTERANGE:10@0\na.ts\n#EXT-X-ENDLIST",
                "video.byterange",
            ),
            (
                "#EXT-X-MAP:URI=\"a\"\n#EXTINF:4,\na.m4s\n#EXT-X-MAP:URI=\"b\"\n#EXT-X-ENDLIST",
                "video.unsupported",
            ),
            (
                "#EXTINF:4,\nfile:///c:/secret\n#EXT-X-ENDLIST",
                "video.encrypted",
            ),
            ("#EXT-X-ENDLIST", "video.invalid_playlist"),
        ] {
            assert_eq!(media(body).unwrap_err(), code, "{body}");
        }
        assert_eq!(
            parse("<html>", &base()).unwrap_err(),
            "video.invalid_playlist"
        );
        let many = "#EXTINF:1,\na.ts\n".repeat(MAX_SEGMENTS + 1) + "#EXT-X-ENDLIST";
        assert_eq!(media(&many).unwrap_err(), "video.too_many_segments");
    }

    #[test]
    fn lists_qualities_best_first_and_flags_separate_audio() {
        let text = "#EXTM3U\n#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"aud\",URI=\"audio.m3u8\"\n\
#EXT-X-STREAM-INF:BANDWIDTH=100,RESOLUTION=640x360,CODECS=\"avc1,mp4a\"\nlow.m3u8\n\
#EXT-X-STREAM-INF:BANDWIDTH=900,RESOLUTION=1920x1080,AUDIO=\"aud\"\nhigh.m3u8\n";
        let Playlist::Master(variants) = parse(text, &base()).unwrap() else {
            panic!("expected a master playlist");
        };
        assert_eq!(
            variants.iter().map(|v| v.height).collect::<Vec<_>>(),
            [1080, 360]
        );
        assert!(variants[0].separate_audio);
        assert!(!variants[1].separate_audio);
        assert_eq!(variants[1].bandwidth, 100);
    }
}
