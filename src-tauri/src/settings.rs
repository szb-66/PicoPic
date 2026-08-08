use crate::models::AppSettings;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> AppSettings {
        self.try_load().unwrap_or_default()
    }

    fn try_load(&self) -> Option<AppSettings> {
        let bytes = fs::read(&self.path).ok()?;
        let settings: AppSettings = serde_json::from_slice(&bytes).ok()?;
        settings.validate().ok()?;
        Some(settings)
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), String> {
        settings.validate()?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        let mut temporary =
            tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
        serde_json::to_writer_pretty(&mut temporary, settings)
            .map_err(|error| error.to_string())?;
        temporary.flush().map_err(|error| error.to_string())?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| error.to_string())?;
        if self.path.exists() {
            let (_, temporary_path) = temporary.keep().map_err(|error| error.error.to_string())?;
            if let Err(error) = replace_existing(&self.path, &temporary_path) {
                let _ = fs::remove_file(&temporary_path);
                return Err(error);
            }
        } else {
            temporary
                .persist(&self.path)
                .map_err(|error| error.error.to_string())?;
        }
        Ok(())
    }
}

#[cfg(windows)]
fn replace_existing(target: &Path, replacement: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{ReplaceFileW, REPLACEFILE_IGNORE_MERGE_ERRORS};
    let target: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
    let replacement: Vec<u16> = replacement
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let result = unsafe {
        ReplaceFileW(
            target.as_ptr(),
            replacement.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_IGNORE_MERGE_ERRORS,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error().to_string())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_existing(_target: &Path, _replacement: &Path) -> Result<(), String> {
    Err("当前平台未实现设置文件原子替换".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_valid_settings() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("settings.json"));
        let mut settings = AppSettings::default();
        settings.webp.quality = 62;
        store.save(&settings).unwrap();
        assert_eq!(store.load(), settings);
    }

    #[test]
    fn corrupt_settings_fall_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, b"not json").unwrap();
        assert_eq!(SettingsStore::new(path).load(), AppSettings::default());
    }

    #[test]
    fn unsupported_settings_version_falls_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let unsupported = AppSettings {
            schema_version: 2,
            ..AppSettings::default()
        };
        fs::write(&path, serde_json::to_vec(&unsupported).unwrap()).unwrap();
        assert_eq!(SettingsStore::new(path).load(), AppSettings::default());
    }
}
