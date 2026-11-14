use super::*;

fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        let mut writer = encoder.write_header().unwrap();
        writer
            .write_image_data(&vec![120; (width * height * 4) as usize])
            .unwrap();
    }
    bytes
}

#[test]
fn references_accept_only_registered_id_shapes() {
    for value in [
        r#"{"kind":"file","path":"C:/Windows/file.png"}"#,
        r#"{"kind":"task","id":"one","path":"elsewhere"}"#,
        r#"{"kind":"history","id":"one"}"#,
        r#"{"kind":"task","id":"one","id":"two"}"#,
    ] {
        assert!(serde_json::from_str::<VisualSource>(value).is_err());
    }
    assert!(serde_json::from_str::<VisualSource>(r#"{"kind":"task","id":"one"}"#).is_ok());
    assert!(serde_json::from_str::<VisualSource>(r#"{"kind":"history","id":1}"#).is_ok());
}

#[test]
fn local_reads_stay_inside_the_published_download_directory() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let file = root.path().join("picture.png");
    std::fs::write(&file, png(12, 6)).unwrap();
    let visual = inspect(root.path(), &file);
    assert!(visual.size_bytes.is_some());
    #[cfg(windows)]
    assert!(visual.thumbnail_png.is_some());
    let other = outside.path().join("picture.png");
    std::fs::write(&other, png(1, 1)).unwrap();
    for path in [&other, &root.path().join("missing.png"), root.path()] {
        let visual = inspect(root.path(), path);
        assert!(visual.size_bytes.is_none() && visual.thumbnail_png.is_none());
    }
    let nested = root.path().join(".spark-partials");
    std::fs::create_dir(&nested).unwrap();
    let partial = nested.join("partial.png");
    std::fs::write(&partial, png(1, 1)).unwrap();
    assert!(inspect(root.path(), &partial).size_bytes.is_none());
    std::fs::remove_file(&file).unwrap();
    assert!(inspect(root.path(), &file).size_bytes.is_none());
}

#[test]
fn unsupported_corrupt_and_oversized_files_keep_metadata_without_a_preview() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("payload.bin");
    for bytes in [
        b"not an image".to_vec(),
        b"\x89PNG\r\n\x1a\ncorrupt".to_vec(),
        b"<svg onload='unsafe()'/>".to_vec(),
    ] {
        std::fs::write(&file, &bytes).unwrap();
        let visual = inspect(root.path(), &file);
        assert_eq!(visual.size_bytes, Some(bytes.len() as u64));
        assert!(visual.thumbnail_png.is_none());
    }
    File::create(&file).unwrap().set_len(MAX_FILE + 1).unwrap();
    let visual = inspect(root.path(), &file);
    assert_eq!(visual.size_bytes, Some(MAX_FILE + 1));
    assert!(visual.thumbnail_png.is_none());
}

#[test]
fn scaling_rejects_decompression_bombs_and_keeps_aspect_ratio() {
    assert_eq!(thumbnail_size(1920, 1080), Some((96, 54)));
    assert_eq!(thumbnail_size(20, 40), Some((20, 40)));
    assert_eq!(thumbnail_size(1, 1000), Some((1, 96)));
    for (width, height) in [(0, 1), (1, 0), (8000, 8000), (u32::MAX, u32::MAX)] {
        assert!(thumbnail_size(width, height).is_none());
    }
}

#[cfg(windows)]
#[test]
fn wic_returns_only_small_reencoded_pngs_and_rejects_links() {
    let bytes = windows::thumbnail(&png(320, 200)).unwrap();
    let reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    assert_eq!((reader.info().width, reader.info().height), (96, 60));
    assert!(windows::thumbnail(&png(4001, 4000)).is_none());
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let other = outside.path().join("external.png");
    std::fs::write(&other, png(1, 1)).unwrap();
    let link = root.path().join("link.png");
    // Non-admin Windows installations may disable symbolic-link creation.
    if std::os::windows::fs::symlink_file(&other, &link).is_ok() {
        assert!(inspect(root.path(), &link).size_bytes.is_none());
    }
    assert!(inspect(root.path(), &root.path().join("../external.png"))
        .size_bytes
        .is_none());
}
