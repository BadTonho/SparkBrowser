use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub fn extensions_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app_data_dir: {}", e))?;
    let ext_dir = app_data.join("extensions");
    if !ext_dir.exists() {
        fs::create_dir_all(&ext_dir)
            .map_err(|e| format!("Failed to create extensions directory: {}", e))?;
    }
    Ok(ext_dir)
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    }
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_type = entry.file_type().map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn remove_extension_dir(app: &AppHandle, target: &Path) -> Result<(), String> {
    remove_managed_directory(&extensions_dir(app)?, target)
}

fn remove_managed_directory(base: &Path, target: &Path) -> Result<(), String> {
    let base = base.canonicalize().map_err(|e| e.to_string())?;
    if !target.exists() {
        return Ok(());
    }
    let resolved = target.canonicalize().map_err(|e| e.to_string())?;
    if resolved.parent() != Some(base.as_path()) {
        return Err("Extension directory is outside managed storage".into());
    }
    fs::remove_dir_all(resolved).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_the_recorded_directory_but_refuses_paths_outside_storage() {
        let root = tempfile::tempdir().unwrap();
        let base = root.path().join("extensions");
        let installed = base.join("copied-folder");
        let outside = root.path().join("outside");
        fs::create_dir_all(&installed).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(installed.join("manifest.json"), b"{}").unwrap();
        assert!(remove_managed_directory(&base, &outside).is_err());
        assert!(outside.exists());
        remove_managed_directory(&base, &installed).unwrap();
        assert!(!installed.exists());
        assert!(base.exists());
    }
}
