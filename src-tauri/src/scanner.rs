use crate::models::{ImageFormat, ScanWarning};
use std::{
    collections::HashSet,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

pub const MAX_SOURCE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_PIXELS: u64 = 40_000_000;

#[derive(Debug, Clone)]
pub struct ScanCandidate {
    pub path: PathBuf,
    pub format: Option<ImageFormat>,
    pub skip_reason: Option<String>,
    pub original_bytes: u64,
}

fn extension_format(path: &Path) -> Option<ImageFormat> {
    match path
        .extension()?
        .to_string_lossy()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "png" => Some(ImageFormat::Png),
        "webp" => Some(ImageFormat::Webp),
        _ => None,
    }
}

fn png_is_animated(bytes: &[u8]) -> bool {
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return false;
    }
    let mut offset = 8_usize;
    while offset + 12 <= bytes.len() {
        let Ok(length_bytes) = bytes[offset..offset + 4].try_into() else {
            return false;
        };
        let length = u32::from_be_bytes(length_bytes) as usize;
        let Some(end) = offset
            .checked_add(12)
            .and_then(|chunk_start| chunk_start.checked_add(length))
        else {
            return false;
        };
        if end > bytes.len() {
            return false;
        }
        let kind = &bytes[offset + 4..offset + 8];
        if kind == b"acTL" {
            return true;
        }
        offset = end;
        if kind == b"IEND" {
            return false;
        }
    }
    false
}

fn webp_is_animated(bytes: &[u8]) -> bool {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return false;
    }
    let mut offset = 12_usize;
    while offset + 8 <= bytes.len() {
        let Ok(size_bytes) = bytes[offset + 4..offset + 8].try_into() else {
            return false;
        };
        let size = u32::from_le_bytes(size_bytes) as usize;
        let Some(end) = offset
            .checked_add(8)
            .and_then(|data_start| data_start.checked_add(size))
            .and_then(|data_end| data_end.checked_add(size & 1))
        else {
            return false;
        };
        if end > bytes.len() {
            return false;
        }
        let kind = &bytes[offset..offset + 4];
        if kind == b"ANIM" || kind == b"ANMF" {
            return true;
        }
        offset = end;
    }
    false
}

pub fn detect_format(path: &Path, bytes: &[u8]) -> Result<ImageFormat, String> {
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err("图片超过 128 MiB 的安全处理上限".into());
    }
    let expected = extension_format(path).ok_or_else(|| "不支持的图片类型".to_string())?;
    let detected = if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some(ImageFormat::Jpeg)
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(ImageFormat::Png)
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some(ImageFormat::Webp)
    } else {
        None
    };
    if detected != Some(expected) {
        return Err("扩展名与文件内容不匹配".into());
    }
    if expected == ImageFormat::Png && png_is_animated(bytes) {
        return Err("暂不支持 APNG 动画".into());
    }
    if expected == ImageFormat::Webp && webp_is_animated(bytes) {
        return Err("暂不支持动画 WebP".into());
    }
    let image_format = match expected {
        ImageFormat::Jpeg => image::ImageFormat::Jpeg,
        ImageFormat::Png => image::ImageFormat::Png,
        ImageFormat::Webp => image::ImageFormat::WebP,
    };
    let (width, height) = image::ImageReader::with_format(Cursor::new(bytes), image_format)
        .into_dimensions()
        .map_err(|_| "图片数据损坏或无法解码".to_string())?;
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err("图片超过 4000 万像素的安全处理上限".into());
    }
    image::load_from_memory_with_format(bytes, image_format)
        .map_err(|_| "图片数据损坏或无法解码".to_string())?;
    Ok(expected)
}

#[cfg(windows)]
fn is_reparse_point(entry: &DirEntry) -> bool {
    use std::os::windows::fs::MetadataExt;
    entry
        .metadata()
        .map(|metadata| metadata.file_attributes() & 0x400 != 0)
        .unwrap_or(true)
}

#[cfg(not(windows))]
fn is_reparse_point(entry: &DirEntry) -> bool {
    entry.file_type().is_symlink()
}

fn inspect_file(path: PathBuf) -> Option<ScanCandidate> {
    extension_format(&path)?;
    let metadata = match fs::metadata(&path) {
        Ok(value) => value,
        Err(error) => {
            return Some(ScanCandidate {
                path,
                format: None,
                skip_reason: Some(format!("无法读取文件信息：{error}")),
                original_bytes: 0,
            })
        }
    };
    if metadata.len() > MAX_SOURCE_BYTES {
        return Some(ScanCandidate {
            path,
            format: None,
            skip_reason: Some("图片超过 128 MiB 的安全处理上限".into()),
            original_bytes: metadata.len(),
        });
    }
    let bytes = match fs::read(&path) {
        Ok(value) => value,
        Err(error) => {
            return Some(ScanCandidate {
                path,
                format: None,
                skip_reason: Some(format!("无法读取文件：{error}")),
                original_bytes: metadata.len(),
            })
        }
    };
    match detect_format(&path, &bytes) {
        Ok(format) => Some(ScanCandidate {
            path,
            format: Some(format),
            skip_reason: None,
            original_bytes: metadata.len(),
        }),
        Err(reason) => Some(ScanCandidate {
            path,
            format: None,
            skip_reason: Some(reason),
            original_bytes: metadata.len(),
        }),
    }
}

pub fn scan_paths(paths: &[String], recursive: bool) -> (Vec<ScanCandidate>, Vec<ScanWarning>) {
    let mut found = Vec::new();
    let mut warnings = Vec::new();
    let mut seen = HashSet::new();
    for raw in paths {
        let path = PathBuf::from(raw);
        if path.is_file() {
            let canonical = match fs::canonicalize(&path) {
                Ok(path) => path,
                Err(error) => {
                    warnings.push(ScanWarning {
                        path: raw.clone(),
                        reason: error.to_string(),
                    });
                    continue;
                }
            };
            if seen.insert(canonical.clone()) {
                if let Some(candidate) = inspect_file(canonical) {
                    found.push(candidate);
                }
            }
            continue;
        }
        if !path.is_dir() {
            warnings.push(ScanWarning {
                path: raw.clone(),
                reason: "路径不存在或不可访问".into(),
            });
            continue;
        }
        let walker = if recursive {
            WalkDir::new(&path)
        } else {
            WalkDir::new(&path).max_depth(1)
        };
        for entry in walker.follow_links(false).into_iter() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    warnings.push(ScanWarning {
                        path: error
                            .path()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| raw.clone()),
                        reason: error.to_string(),
                    });
                    continue;
                }
            };
            if entry.file_type().is_symlink()
                || is_reparse_point(&entry)
                || !entry.file_type().is_file()
            {
                continue;
            }
            let canonical = match fs::canonicalize(entry.path()) {
                Ok(path) => path,
                Err(error) => {
                    warnings.push(ScanWarning {
                        path: entry.path().display().to_string(),
                        reason: error.to_string(),
                    });
                    continue;
                }
            };
            if seen.insert(canonical.clone()) {
                if let Some(candidate) = inspect_file(canonical) {
                    found.push(candidate);
                }
            }
        }
    }
    (found, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_with_chunk(bytes: &[u8], kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let iend = bytes
            .windows(4)
            .position(|window| window == b"IEND")
            .unwrap()
            - 4;
        let mut chunk = Vec::new();
        chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
        chunk.extend_from_slice(kind);
        chunk.extend_from_slice(data);
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(kind);
        hasher.update(data);
        chunk.extend_from_slice(&hasher.finalize().to_be_bytes());
        [bytes[..iend].to_vec(), chunk, bytes[iend..].to_vec()].concat()
    }

    fn webp_chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut bytes = b"RIFF\0\0\0\0WEBP".to_vec();
        bytes.extend_from_slice(kind);
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(data);
        if data.len() % 2 == 1 {
            bytes.push(0);
        }
        let riff_size = (bytes.len() - 8) as u32;
        bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
        bytes
    }

    fn valid_png() -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, 2, 2);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[255; 16]).unwrap();
        }
        bytes
    }

    #[test]
    fn rejects_oversized_files_before_reading_them() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("huge.png");
        fs::File::create(&path)
            .unwrap()
            .set_len(MAX_SOURCE_BYTES + 1)
            .unwrap();
        let (found, _) = scan_paths(&[path.display().to_string()], false);
        assert_eq!(found.len(), 1);
        assert!(found[0].skip_reason.as_ref().unwrap().contains("128 MiB"));
    }

    #[test]
    fn detects_signature_mismatch() {
        let error = detect_format(Path::new("image.png"), b"not a png").unwrap_err();
        assert!(error.contains("不匹配"));
    }

    #[test]
    fn detects_apng_marker() {
        let bytes = png_with_chunk(&valid_png(), b"acTL", &[0, 0, 0, 1, 0, 0, 0, 0]);
        assert!(detect_format(Path::new("image.png"), &bytes)
            .unwrap_err()
            .contains("APNG"));
    }

    #[test]
    fn detects_animated_webp_marker() {
        let bytes = webp_chunk(b"ANIM", &[]);
        assert!(detect_format(Path::new("image.webp"), &bytes)
            .unwrap_err()
            .contains("动画 WebP"));
    }

    #[test]
    fn marker_bytes_inside_payloads_do_not_imply_animation() {
        let png = png_with_chunk(&valid_png(), b"tEXt", b"Comment\0contains acTL text");
        assert_eq!(
            detect_format(Path::new("image.png"), &png).unwrap(),
            ImageFormat::Png
        );

        let webp = webp_chunk(b"EXIF", b"ANIM and ANMF are metadata text");
        assert!(!webp_is_animated(&webp));
    }

    #[test]
    fn ignores_unsupported_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("notes.txt"), b"hello").unwrap();
        let (found, warnings) = scan_paths(&[dir.path().display().to_string()], true);
        assert!(found.is_empty() && warnings.is_empty());
    }

    #[test]
    fn recursively_scans_supported_files_and_deduplicates_paths() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let image = nested.join("image.png");
        fs::write(&image, valid_png()).unwrap();
        fs::write(dir.path().join("notes.txt"), b"unsupported").unwrap();

        let raw = dir.path().display().to_string();
        let (recursive, warnings) = scan_paths(&[raw.clone(), raw], true);
        assert!(warnings.is_empty());
        assert_eq!(recursive.len(), 1);
        assert_eq!(recursive[0].format, Some(ImageFormat::Png));

        let (shallow, _) = scan_paths(&[dir.path().display().to_string()], false);
        assert!(shallow.is_empty());
    }
}
