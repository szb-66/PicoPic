use crate::{
    models::{AppSettings, ImageFormat, OptimizationLevel},
    scanner::{detect_format, MAX_SOURCE_BYTES},
};
use filetime::{set_file_times, FileTime};
use image::{DynamicImage, GenericImageView};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptimizeError {
    #[error("无法读取图片：{0}")]
    Read(#[from] std::io::Error),
    #[error("无法解码图片：{0}")]
    Decode(String),
    #[error("无法编码图片：{0}")]
    Encode(String),
    #[error("输出校验失败：{0}")]
    Validation(String),
    #[error("无法安全替换原文件：{0}")]
    Replace(String),
    #[error("任务已取消")]
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OptimizeOutcome {
    Replaced {
        original_bytes: u64,
        result_bytes: u64,
    },
    Unchanged {
        original_bytes: u64,
    },
}

const TOKEN_RUNNING: u8 = 0;
const TOKEN_CANCELLED: u8 = 1;
const TOKEN_COMMITTING: u8 = 2;
const TOKEN_COMMITTED: u8 = 3;

#[derive(Debug, Default)]
pub struct CancellationToken {
    state: AtomicU8,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) -> bool {
        self.state
            .compare_exchange(
                TOKEN_RUNNING,
                TOKEN_CANCELLED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }

    pub fn is_cancelled(&self) -> bool {
        self.state.load(Ordering::Acquire) == TOKEN_CANCELLED
    }

    fn begin_commit(&self) -> Result<(), OptimizeError> {
        self.state
            .compare_exchange(
                TOKEN_RUNNING,
                TOKEN_COMMITTING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|_| OptimizeError::Cancelled)
    }

    fn finish_commit(&self) {
        self.state.store(TOKEN_COMMITTED, Ordering::Release);
    }
}

pub fn effort_index(level: OptimizationLevel) -> u8 {
    match level {
        OptimizationLevel::Fast => 1,
        OptimizationLevel::Normal => 3,
        OptimizationLevel::Extra => 5,
        OptimizationLevel::Extreme => 6,
    }
}

fn orientation(bytes: &[u8]) -> u32 {
    let mut cursor = Cursor::new(bytes);
    exif::Reader::new()
        .read_from_container(&mut cursor)
        .ok()
        .and_then(|exif| {
            exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
                .and_then(|field| field.value.get_uint(0))
        })
        .unwrap_or(1)
}

fn decode_oriented(bytes: &[u8]) -> Result<DynamicImage, OptimizeError> {
    let image =
        image::load_from_memory(bytes).map_err(|error| OptimizeError::Decode(error.to_string()))?;
    Ok(match orientation(bytes) {
        2 => image.fliph(),
        3 => image.rotate180(),
        4 => image.flipv(),
        5 => image.fliph().rotate270(),
        6 => image.rotate90(),
        7 => image.fliph().rotate90(),
        8 => image.rotate270(),
        _ => image,
    })
}

fn jpeg_preset(level: OptimizationLevel) -> mozjpeg_rs::Preset {
    match level {
        OptimizationLevel::Fast => mozjpeg_rs::Preset::BaselineFastest,
        OptimizationLevel::Normal => mozjpeg_rs::Preset::BaselineBalanced,
        OptimizationLevel::Extra => mozjpeg_rs::Preset::ProgressiveBalanced,
        OptimizationLevel::Extreme => mozjpeg_rs::Preset::ProgressiveSmallest,
    }
}

fn encode_jpeg(
    image: &DynamicImage,
    quality: u8,
    level: OptimizationLevel,
) -> Result<Vec<u8>, OptimizeError> {
    let rgb = image.to_rgb8();
    mozjpeg_rs::Encoder::new(jpeg_preset(level))
        .quality(quality)
        .encode_rgb(rgb.as_raw(), rgb.width(), rgb.height())
        .map_err(|error| OptimizeError::Encode(error.to_string()))
}

fn encode_indexed_png(
    image: &DynamicImage,
    source: &[u8],
    quality: u8,
    level: OptimizationLevel,
    strip_metadata: bool,
) -> Result<Vec<u8>, OptimizeError> {
    let rgba = image.to_rgba8();
    let pixels: Vec<imagequant::RGBA> = rgba
        .pixels()
        .map(|pixel| imagequant::RGBA {
            r: pixel[0],
            g: pixel[1],
            b: pixel[2],
            a: pixel[3],
        })
        .collect();
    let mut attributes = imagequant::new();
    attributes
        .set_quality(0, quality)
        .map_err(|error| OptimizeError::Encode(error.to_string()))?;
    attributes
        .set_speed(match level {
            OptimizationLevel::Fast => 9,
            OptimizationLevel::Normal => 6,
            OptimizationLevel::Extra => 3,
            OptimizationLevel::Extreme => 1,
        })
        .map_err(|error| OptimizeError::Encode(error.to_string()))?;
    let mut quant_image = attributes
        .new_image(pixels, rgba.width() as usize, rgba.height() as usize, 0.0)
        .map_err(|error| OptimizeError::Encode(error.to_string()))?;
    let mut result = attributes
        .quantize(&mut quant_image)
        .map_err(|error| OptimizeError::Encode(error.to_string()))?;
    result
        .set_dithering_level(1.0)
        .map_err(|error| OptimizeError::Encode(error.to_string()))?;
    let (palette, indices) = result
        .remapped(&mut quant_image)
        .map_err(|error| OptimizeError::Encode(error.to_string()))?;
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, rgba.width(), rgba.height());
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_palette(
            palette
                .iter()
                .flat_map(|color| [color.r, color.g, color.b])
                .collect::<Vec<_>>(),
        );
        encoder.set_trns(palette.iter().map(|color| color.a).collect::<Vec<_>>());
        let mut writer = encoder
            .write_header()
            .map_err(|error| OptimizeError::Encode(error.to_string()))?;
        writer
            .write_image_data(&indices)
            .map_err(|error| OptimizeError::Encode(error.to_string()))?;
    }
    let output = if strip_metadata {
        output
    } else {
        preserve_png_metadata(&output, source)
    };
    optimize_png_lossless(&output, level, strip_metadata)
}

fn optimize_png_lossless(
    bytes: &[u8],
    level: OptimizationLevel,
    strip_metadata: bool,
) -> Result<Vec<u8>, OptimizeError> {
    let mut options = oxipng::Options::from_preset(effort_index(level));
    if strip_metadata {
        options.strip = oxipng::StripChunks::All;
    }
    oxipng::optimize_from_memory(bytes, &options)
        .map_err(|error| OptimizeError::Encode(error.to_string()))
}

fn webp_effort(level: OptimizationLevel) -> (i32, i32) {
    match level {
        OptimizationLevel::Fast => (0, 1),
        OptimizationLevel::Normal => (3, 2),
        OptimizationLevel::Extra => (5, 4),
        OptimizationLevel::Extreme => (6, 10),
    }
}

fn encode_webp(
    image: &DynamicImage,
    quality: u8,
    lossless: bool,
    level: OptimizationLevel,
) -> Result<Vec<u8>, OptimizeError> {
    let rgba = image.to_rgba8();
    let encoder = webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
    let mut config = webp::WebPConfig::new()
        .map_err(|_| OptimizeError::Encode("无法初始化 WebP 编码配置".into()))?;
    let (method, pass) = webp_effort(level);
    config.lossless = if lossless { 1 } else { 0 };
    config.quality = quality as f32;
    config.method = method;
    config.pass = pass;
    config.alpha_compression = 1;
    config.thread_level = 1;
    encoder
        .encode_advanced(&config)
        .map(|output| output.to_vec())
        .map_err(|error| OptimizeError::Encode(format!("{error:?}")))
}

// Rotation is applied to pixels before lossy encoding. Reset only the IFD0
// orientation value so preserved EXIF cannot rotate them a second time.
fn normalized_exif(payload: &[u8]) -> Vec<u8> {
    let mut output = payload.to_vec();
    let base = if output.starts_with(b"Exif\0\0") {
        6
    } else {
        0
    };
    let Some(tiff) = output.get(base..) else {
        return output;
    };
    let little = match tiff.get(..2) {
        Some(b"II") => true,
        Some(b"MM") => false,
        _ => return output,
    };
    let read_u16 = |bytes: &[u8]| {
        let pair = [bytes[0], bytes[1]];
        if little {
            u16::from_le_bytes(pair)
        } else {
            u16::from_be_bytes(pair)
        }
    };
    let Some(offset_bytes) = tiff.get(4..8) else {
        return output;
    };
    let offset_bytes: [u8; 4] = offset_bytes.try_into().unwrap();
    let offset = if little {
        u32::from_le_bytes(offset_bytes)
    } else {
        u32::from_be_bytes(offset_bytes)
    } as usize;
    let Some(count_bytes) = tiff.get(offset..offset.saturating_add(2)) else {
        return output;
    };
    let count = read_u16(count_bytes) as usize;
    for index in 0..count {
        let Some(start) = offset
            .checked_add(2)
            .and_then(|v| v.checked_add(index * 12))
        else {
            break;
        };
        let Some(entry) = output.get(base + start..base + start + 12) else {
            break;
        };
        let count_one = if little { [1, 0, 0, 0] } else { [0, 0, 0, 1] };
        if read_u16(&entry[..2]) == 0x0112
            && read_u16(&entry[2..4]) == 3
            && entry[4..8] == count_one
        {
            let one = if little { [1, 0] } else { [0, 1] };
            output[base + start + 8..base + start + 10].copy_from_slice(&one);
        }
    }
    output
}

fn jpeg_metadata_segments(bytes: &[u8], normalize_orientation: bool) -> Vec<u8> {
    if bytes.len() < 4 || !bytes.starts_with(&[0xFF, 0xD8]) {
        return Vec::new();
    }
    let mut metadata = Vec::new();
    let mut offset = 2;
    while offset + 4 <= bytes.len() && bytes[offset] == 0xFF {
        let marker = bytes[offset + 1];
        if marker == 0xDA || marker == 0xD9 {
            break;
        }
        let length = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]) as usize;
        if length < 2 || offset + 2 + length > bytes.len() {
            break;
        }
        let end = offset + 2 + length;
        let is_metadata = (0xE1..=0xEF).contains(&marker) || marker == 0xFE;
        let is_exif = marker == 0xE1 && bytes[offset + 4..end].starts_with(b"Exif\0\0");
        if is_metadata {
            if normalize_orientation && is_exif {
                metadata.extend_from_slice(&bytes[offset..offset + 4]);
                metadata.extend_from_slice(&normalized_exif(&bytes[offset + 4..end]));
            } else {
                metadata.extend_from_slice(&bytes[offset..end]);
            }
        }
        offset = end;
    }
    metadata
}

fn preserve_jpeg_metadata(encoded: &[u8], source: &[u8], normalize_orientation: bool) -> Vec<u8> {
    if !encoded.starts_with(&[0xFF, 0xD8]) {
        return encoded.to_vec();
    }
    let metadata = jpeg_metadata_segments(source, normalize_orientation);
    if metadata.is_empty() {
        return encoded.to_vec();
    }
    [encoded[..2].to_vec(), metadata, encoded[2..].to_vec()].concat()
}

fn png_chunks(bytes: &[u8]) -> Option<Vec<(&[u8], &[u8])>> {
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return None;
    }
    let mut chunks = Vec::new();
    let mut offset = 8;
    while offset + 12 <= bytes.len() {
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
        let end = offset.checked_add(12 + length)?;
        if end > bytes.len() {
            return None;
        }
        chunks.push((&bytes[offset + 4..offset + 8], &bytes[offset..end]));
        offset = end;
        if chunks.last()?.0 == b"IEND" {
            return Some(chunks);
        }
    }
    None
}

fn preserve_png_metadata(encoded: &[u8], source: &[u8]) -> Vec<u8> {
    let Some(source_chunks) = png_chunks(source) else {
        return encoded.to_vec();
    };
    let metadata: Vec<&[u8]> = source_chunks
        .into_iter()
        .filter_map(|(kind, chunk)| {
            matches!(
                kind,
                b"iCCP"
                    | b"eXIf"
                    | b"tEXt"
                    | b"zTXt"
                    | b"iTXt"
                    | b"pHYs"
                    | b"cHRM"
                    | b"gAMA"
                    | b"sRGB"
                    | b"sBIT"
            )
            .then_some(chunk)
        })
        .collect();
    if metadata.is_empty() {
        return encoded.to_vec();
    }
    let Some(chunks) = png_chunks(encoded) else {
        return encoded.to_vec();
    };
    let mut output = b"\x89PNG\r\n\x1a\n".to_vec();
    for (kind, chunk) in chunks {
        output.extend_from_slice(chunk);
        if kind == b"IHDR" {
            for metadata_chunk in &metadata {
                if &metadata_chunk[4..8] == b"eXIf" && orientation(source) != 1 {
                    let payload = normalized_exif(&metadata_chunk[8..metadata_chunk.len() - 4]);
                    output.extend_from_slice(&(payload.len() as u32).to_be_bytes());
                    output.extend_from_slice(b"eXIf");
                    output.extend_from_slice(&payload);
                    let mut crc = crc32fast::Hasher::new();
                    crc.update(b"eXIf");
                    crc.update(&payload);
                    output.extend_from_slice(&crc.finalize().to_be_bytes());
                } else {
                    output.extend_from_slice(metadata_chunk);
                }
            }
        }
    }
    output
}

fn webp_chunks(bytes: &[u8]) -> Option<Vec<(&[u8], &[u8])>> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }
    let mut chunks = Vec::new();
    let mut offset = 12;
    while offset + 8 <= bytes.len() {
        let size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().ok()?) as usize;
        let end = offset.checked_add(8 + size + (size & 1))?;
        if end > bytes.len() {
            return None;
        }
        chunks.push((&bytes[offset..offset + 4], &bytes[offset..end]));
        offset = end;
    }
    (offset == bytes.len()).then_some(chunks)
}

fn webp_is_lossless(bytes: &[u8]) -> bool {
    webp_chunks(bytes).is_some_and(|chunks| chunks.iter().any(|(kind, _)| *kind == b"VP8L"))
}

fn push_u24(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes()[..3]);
}

fn preserve_webp_metadata(
    encoded: &[u8],
    source: &[u8],
    dimensions: (u32, u32),
    has_alpha: bool,
    normalize_orientation: bool,
) -> Vec<u8> {
    let Some(source_chunks) = webp_chunks(source) else {
        return encoded.to_vec();
    };
    let metadata: Vec<(&[u8], &[u8])> = source_chunks
        .into_iter()
        .filter(|(kind, _)| matches!(*kind, b"EXIF" | b"XMP " | b"ICCP"))
        .collect();
    if metadata.is_empty() {
        return encoded.to_vec();
    }
    let Some(encoded_chunks) = webp_chunks(encoded) else {
        return encoded.to_vec();
    };
    let metadata_flags = metadata.iter().fold(0_u8, |flags, (kind, _)| {
        flags
            | match *kind {
                b"ICCP" => 0x20,
                b"EXIF" => 0x08,
                b"XMP " => 0x04,
                _ => 0,
            }
    });
    let mut output = b"RIFF\0\0\0\0WEBP".to_vec();
    let mut wrote_vp8x = false;
    for (kind, chunk) in encoded_chunks {
        if kind == b"VP8X" && chunk.len() >= 18 {
            let mut updated = chunk.to_vec();
            updated[8] |= metadata_flags;
            output.extend_from_slice(&updated);
            wrote_vp8x = true;
        } else {
            if !wrote_vp8x {
                output.extend_from_slice(b"VP8X\x0a\0\0\0");
                output.push(metadata_flags | if has_alpha { 0x10 } else { 0 });
                output.extend_from_slice(&[0, 0, 0]);
                push_u24(&mut output, dimensions.0.saturating_sub(1));
                push_u24(&mut output, dimensions.1.saturating_sub(1));
                wrote_vp8x = true;
            }
            output.extend_from_slice(chunk);
        }
    }
    for (kind, chunk) in metadata {
        if kind == b"EXIF" && normalize_orientation {
            let size = u32::from_le_bytes(chunk[4..8].try_into().unwrap()) as usize;
            output.extend_from_slice(&chunk[..8]);
            output.extend_from_slice(&normalized_exif(&chunk[8..8 + size]));
            if size % 2 == 1 {
                output.push(0);
            }
        } else {
            output.extend_from_slice(chunk);
        }
    }
    let riff_size = (output.len() - 8) as u32;
    output[4..8].copy_from_slice(&riff_size.to_le_bytes());
    output
}

fn strip_jpeg_metadata(bytes: &[u8]) -> Vec<u8> {
    if bytes.len() < 4 || !bytes.starts_with(&[0xFF, 0xD8]) {
        return bytes.to_vec();
    }
    let mut out = vec![0xFF, 0xD8];
    let mut offset = 2;
    while offset + 4 <= bytes.len() {
        if bytes[offset] != 0xFF {
            break;
        }
        let marker = bytes[offset + 1];
        if marker == 0xDA {
            out.extend_from_slice(&bytes[offset..]);
            return out;
        }
        if marker == 0xD9 {
            out.extend_from_slice(&bytes[offset..offset + 2]);
            return out;
        }
        let length = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]) as usize;
        if length < 2 || offset + 2 + length > bytes.len() {
            return bytes.to_vec();
        }
        let end = offset + 2 + length;
        let removable = (0xE1..=0xEF).contains(&marker) || marker == 0xFE;
        if !removable {
            out.extend_from_slice(&bytes[offset..end]);
        }
        offset = end;
    }
    bytes.to_vec()
}

fn strip_webp_metadata(bytes: &[u8]) -> Vec<u8> {
    let Some(chunks) = webp_chunks(bytes) else {
        return bytes.to_vec();
    };
    let mut out = b"RIFF\0\0\0\0WEBP".to_vec();
    for (kind, chunk) in chunks {
        if matches!(kind, b"EXIF" | b"XMP " | b"ICCP") {
            continue;
        }
        if kind == b"VP8X" && chunk.len() >= 18 {
            let mut updated = chunk.to_vec();
            updated[8] &= !(0x20 | 0x08 | 0x04);
            out.extend_from_slice(&updated);
        } else {
            out.extend_from_slice(chunk);
        }
    }
    let riff_size = (out.len() - 8) as u32;
    out[4..8].copy_from_slice(&riff_size.to_le_bytes());
    out
}

fn encoded_bytes(
    source: &[u8],
    format: ImageFormat,
    settings: &AppSettings,
) -> Result<(Vec<u8>, (u32, u32)), OptimizeError> {
    let format_settings = settings.format(format);
    if !format_settings.lossy {
        if settings.strip_metadata && orientation(source) != 1 {
            return Err(OptimizeError::Validation(
                "图片依赖 EXIF 方向；请保留元数据或启用有损模式，原文件未改变".into(),
            ));
        }
        let original = image::load_from_memory(source)
            .map_err(|error| OptimizeError::Decode(error.to_string()))?;
        let dimensions = original.dimensions();
        return Ok((
            match format {
                ImageFormat::Jpeg => {
                    if settings.strip_metadata {
                        strip_jpeg_metadata(source)
                    } else {
                        source.to_vec()
                    }
                }
                ImageFormat::Png => optimize_png_lossless(
                    source,
                    settings.optimization_level,
                    settings.strip_metadata,
                )?,
                ImageFormat::Webp => {
                    if webp_is_lossless(source) {
                        let image = image::load_from_memory(source)
                            .map_err(|error| OptimizeError::Decode(error.to_string()))?;
                        let has_alpha = image.to_rgba8().pixels().any(|pixel| pixel[3] < 255);
                        let encoded = encode_webp(
                            &image,
                            format_settings.quality,
                            true,
                            settings.optimization_level,
                        )?;
                        if settings.strip_metadata {
                            encoded
                        } else {
                            preserve_webp_metadata(&encoded, source, dimensions, has_alpha, false)
                        }
                    } else if settings.strip_metadata {
                        strip_webp_metadata(source)
                    } else {
                        source.to_vec()
                    }
                }
            },
            dimensions,
        ));
    }
    let image = decode_oriented(source)?;
    let dimensions = image.dimensions();
    let has_alpha = image.to_rgba8().pixels().any(|pixel| pixel[3] < 255);
    let output = match format {
        ImageFormat::Jpeg => {
            let encoded =
                encode_jpeg(&image, format_settings.quality, settings.optimization_level)?;
            if settings.strip_metadata {
                encoded
            } else {
                preserve_jpeg_metadata(&encoded, source, orientation(source) != 1)
            }
        }
        ImageFormat::Png => encode_indexed_png(
            &image,
            source,
            format_settings.quality,
            settings.optimization_level,
            settings.strip_metadata,
        )?,
        ImageFormat::Webp => {
            let encoded = encode_webp(
                &image,
                format_settings.quality,
                false,
                settings.optimization_level,
            )?;
            if settings.strip_metadata {
                encoded
            } else {
                preserve_webp_metadata(&encoded, source, dimensions, has_alpha, true)
            }
        }
    };
    // Unusual or malformed EXIF must not bypass normalization and cause
    // consumers to rotate already-transformed pixels again.
    if orientation(&output) != 1 {
        return Err(OptimizeError::Validation(
            "无法安全规范化输出 EXIF 方向，原文件未改变".into(),
        ));
    }
    Ok((output, dimensions))
}

#[cfg(windows)]
fn atomic_replace(source: &Path, replacement: &Path) -> Result<(), OptimizeError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{ReplaceFileW, REPLACEFILE_IGNORE_MERGE_ERRORS};
    let source_wide: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let replacement_wide: Vec<u16> = replacement
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let result = unsafe {
        ReplaceFileW(
            source_wide.as_ptr(),
            replacement_wide.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_IGNORE_MERGE_ERRORS,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if result == 0 {
        Err(OptimizeError::Replace(
            std::io::Error::last_os_error().to_string(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn atomic_replace(_source: &Path, _replacement: &Path) -> Result<(), OptimizeError> {
    Err(OptimizeError::Replace("当前平台未实现安全原子替换".into()))
}

fn suffix(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Jpeg => ".jpg",
        ImageFormat::Png => ".png",
        ImageFormat::Webp => ".webp",
    }
}

fn validate_temporary_output(
    path: &Path,
    format: ImageFormat,
    original_bytes: u64,
    expected_dimensions: (u32, u32),
) -> Result<u64, OptimizeError> {
    let output = fs::read(path)?;
    let result_bytes = output.len() as u64;
    if result_bytes >= original_bytes {
        return Err(OptimizeError::Validation("临时输出不小于源文件".into()));
    }
    let detected = detect_format(path, &output).map_err(OptimizeError::Validation)?;
    if detected != format {
        return Err(OptimizeError::Validation("输出格式与预期不一致".into()));
    }
    let decoded = image::load_from_memory(&output)
        .map_err(|error| OptimizeError::Validation(error.to_string()))?;
    if decoded.dimensions() != expected_dimensions {
        return Err(OptimizeError::Validation("输出尺寸与预期不一致".into()));
    }
    Ok(result_bytes)
}

// On Windows deny in-place writers while encoding, but permit ReplaceFileW.
// Another process may still rename/replace the path; recheck it before commit.
fn open_source(path: &Path) -> Result<fs::File, OptimizeError> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_DELETE, FILE_SHARE_READ};
        options.share_mode(FILE_SHARE_READ | FILE_SHARE_DELETE);
    }
    options.open(path).map_err(OptimizeError::Read)
}

fn verify_source_unchanged(
    path: &Path,
    original: &[u8],
    metadata: &fs::Metadata,
) -> Result<(), OptimizeError> {
    let current = fs::metadata(path)?;
    let metadata_changed = current.len() != original.len() as u64
        || current.modified()? != metadata.modified()?
        || current.permissions().readonly();
    let mut file = fs::File::open(path)?;
    let mut offset = 0;
    let mut buffer = [0_u8; 64 * 1024];
    let mut content_changed = false;
    if !metadata_changed {
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            if original.get(offset..offset + count) != Some(&buffer[..count]) {
                content_changed = true;
                break;
            }
            offset += count;
        }
    }
    if metadata_changed || content_changed || offset != original.len() {
        return Err(OptimizeError::Replace(
            "源文件在处理期间发生变化，已保留当前文件".into(),
        ));
    }
    Ok(())
}

pub fn optimize_file(
    path: &Path,
    format: ImageFormat,
    settings: &AppSettings,
    cancellation: Arc<CancellationToken>,
) -> Result<OptimizeOutcome, OptimizeError> {
    if cancellation.is_cancelled() {
        return Err(OptimizeError::Cancelled);
    }
    let mut source_handle = open_source(path)?;
    let metadata = source_handle.metadata()?;
    let mut source = Vec::new();
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(OptimizeError::Validation(
            "图片超过 128 MiB 的安全处理上限".into(),
        ));
    }
    (&mut source_handle)
        .take(MAX_SOURCE_BYTES + 1)
        .read_to_end(&mut source)?;
    if detect_format(path, &source).map_err(OptimizeError::Validation)? != format {
        return Err(OptimizeError::Validation("源文件格式发生变化".into()));
    }
    if metadata.permissions().readonly() {
        return Err(OptimizeError::Replace("源文件为只读".into()));
    }
    let original_bytes = source.len() as u64;
    let modified = FileTime::from_last_modification_time(&metadata);
    let accessed = FileTime::from_last_access_time(&metadata);
    let permissions = metadata.permissions();
    let (output, expected_dimensions) = encoded_bytes(&source, format, settings)?;
    if output.len() as u64 >= original_bytes {
        return Ok(OptimizeOutcome::Unchanged { original_bytes });
    }
    if cancellation.is_cancelled() {
        return Err(OptimizeError::Cancelled);
    }
    let parent = path
        .parent()
        .ok_or_else(|| OptimizeError::Replace("无法确定源文件目录".into()))?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".picopic-")
        .suffix(suffix(format))
        .tempfile_in(parent)?;
    temporary.write_all(&output)?;
    temporary.flush()?;
    temporary.as_file().sync_all()?;
    let result_bytes = validate_temporary_output(
        temporary.path(),
        format,
        original_bytes,
        expected_dimensions,
    )?;
    verify_source_unchanged(path, &source, &metadata)?;
    fs::set_permissions(temporary.path(), permissions)?;
    if settings.preserve_timestamps {
        set_file_times(temporary.path(), accessed, modified)?;
    }
    if cancellation.is_cancelled() {
        return Err(OptimizeError::Cancelled);
    }
    verify_source_unchanged(path, &source, &metadata)?;
    let (_, temporary_path) = temporary
        .keep()
        .map_err(|error| OptimizeError::Read(error.error))?;
    if let Err(error) = cancellation.begin_commit() {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    let replacement_result = atomic_replace(path, &temporary_path);
    cancellation.finish_commit();
    drop(source_handle);
    if let Err(error) = replacement_result {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    Ok(OptimizeOutcome::Replaced {
        original_bytes,
        result_bytes,
    })
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

    fn jpeg_with_app2(bytes: &[u8], payload: &[u8]) -> Vec<u8> {
        let mut segment = vec![0xFF, 0xE2];
        segment.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        segment.extend_from_slice(payload);
        [bytes[..2].to_vec(), segment, bytes[2..].to_vec()].concat()
    }

    fn sample_rgba() -> image::RgbaImage {
        image::ImageBuffer::from_fn(320, 240, |x, y| {
            let checker = if (x / 24 + y / 24) % 2 == 0 { 36 } else { 210 };
            image::Rgba([
                checker,
                (x % 256) as u8,
                (y % 256) as u8,
                if x % 7 == 0 { 180 } else { 255 },
            ])
        })
    }

    fn source_bytes(format: ImageFormat) -> Vec<u8> {
        let rgba = sample_rgba();
        match format {
            ImageFormat::Jpeg => {
                let mut bytes = Vec::new();
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 100)
                    .encode_image(&DynamicImage::ImageRgba8(rgba))
                    .unwrap();
                bytes
            }
            ImageFormat::Png => {
                let mut bytes = Vec::new();
                {
                    let mut encoder = png::Encoder::new(&mut bytes, rgba.width(), rgba.height());
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    encoder.set_compression(png::Compression::NoCompression);
                    let mut writer = encoder.write_header().unwrap();
                    writer.write_image_data(rgba.as_raw()).unwrap();
                }
                bytes
            }
            ImageFormat::Webp => {
                let mut bytes =
                    webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height())
                        .encode_lossless()
                        .to_vec();
                let metadata = vec![42_u8; 50_000];
                bytes.extend_from_slice(b"EXIF");
                bytes.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
                bytes.extend_from_slice(&metadata);
                if metadata.len() % 2 == 1 {
                    bytes.push(0);
                }
                let riff_size = (bytes.len() - 8) as u32;
                bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
                bytes
            }
        }
    }

    fn tiff_orientation(value: u8, little: bool) -> Vec<u8> {
        if little {
            vec![
                b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 1, 3, 0, 1, 0, 0, 0, value, 0, 0, 0, 0,
                0, 0, 0,
            ]
        } else {
            vec![
                b'M', b'M', 0, 42, 0, 0, 0, 8, 0, 1, 1, 0x12, 0, 3, 0, 0, 0, 1, 0, value, 0, 0, 0,
                0, 0, 0,
            ]
        }
    }

    fn with_orientation(format: ImageFormat, value: u8, little: bool) -> Vec<u8> {
        let base = source_bytes(format);
        let tiff = tiff_orientation(value, little);
        match format {
            ImageFormat::Jpeg => {
                let payload = [b"Exif\0\0".as_slice(), &tiff].concat();
                let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
                out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
                out.extend_from_slice(&payload);
                out.extend_from_slice(&base[2..]);
                out
            }
            ImageFormat::Png => png_with_chunk(&base, b"eXIf", &tiff),
            ImageFormat::Webp => {
                let rgba = sample_rgba();
                let encoded = webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height())
                    .encode_lossless()
                    .to_vec();
                let mut metadata = b"RIFF\0\0\0\0WEBP".to_vec();
                metadata.extend_from_slice(b"EXIF");
                metadata.extend_from_slice(&(tiff.len() as u32).to_le_bytes());
                metadata.extend_from_slice(&tiff);
                let size = (metadata.len() - 8) as u32;
                metadata[4..8].copy_from_slice(&size.to_le_bytes());
                preserve_webp_metadata(&encoded, &metadata, (320, 240), true, false)
            }
        }
    }

    #[test]
    fn all_orientations_are_reset_after_lossy_encoding_in_every_format() {
        for format in [ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::Webp] {
            for little in [true, false] {
                for value in 2..=8 {
                    let source = with_orientation(format, value, little);
                    assert_eq!(orientation(&source), value as u32);
                    let settings = AppSettings {
                        strip_metadata: false,
                        ..AppSettings::default()
                    };
                    let (output, dimensions) = encoded_bytes(&source, format, &settings).unwrap();
                    assert_eq!(orientation(&output), 1, "{format:?}, {value}, {little}");
                    assert_eq!(dimensions, if value >= 5 { (240, 320) } else { (320, 240) });
                    assert_eq!(decode_oriented(&output).unwrap().dimensions(), dimensions);
                }
            }
        }
    }

    #[test]
    fn diagonal_orientations_match_transpose_and_transverse() {
        let source = with_orientation(ImageFormat::Png, 5, true);
        let raw = image::load_from_memory(&source).unwrap().to_rgba8();
        let transposed = decode_oriented(&source).unwrap().to_rgba8();
        for (x, y, pixel) in raw.enumerate_pixels() {
            assert_eq!(transposed.get_pixel(y, x), pixel);
        }
        let transverse = decode_oriented(&with_orientation(ImageFormat::Png, 7, true))
            .unwrap()
            .to_rgba8();
        for (x, y, pixel) in raw.enumerate_pixels() {
            assert_eq!(
                transverse.get_pixel(raw.height() - 1 - y, raw.width() - 1 - x),
                pixel
            );
        }
    }

    #[test]
    fn lossless_metadata_removal_refuses_to_lose_orientation() {
        for format in [ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::Webp] {
            let mut settings = AppSettings::default();
            settings.jpeg.lossy = false;
            settings.png.lossy = false;
            settings.webp.lossy = false;
            let source = with_orientation(format, 6, true);
            assert!(matches!(
                encoded_bytes(&source, format, &settings),
                Err(OptimizeError::Validation(_))
            ));
            settings.strip_metadata = false;
            let (output, _) = encoded_bytes(&source, format, &settings).unwrap();
            assert_eq!(orientation(&output), 6);
        }
    }

    #[test]
    fn nonstandard_orientation_count_cannot_bypass_output_validation() {
        let mut tiff = tiff_orientation(6, true);
        tiff[14..18].copy_from_slice(&2_u32.to_le_bytes());
        let source = png_with_chunk(&source_bytes(ImageFormat::Png), b"eXIf", &tiff);
        assert_eq!(orientation(&source), 6);
        let settings = AppSettings {
            strip_metadata: false,
            ..AppSettings::default()
        };
        assert!(matches!(
            encoded_bytes(&source, ImageFormat::Png, &settings),
            Err(OptimizeError::Validation(_))
        ));
    }

    #[test]
    fn malformed_exif_is_left_intact_without_panics() {
        for bytes in [
            vec![],
            b"Exif\0\0II".to_vec(),
            vec![255; 100],
            b"II*\0\xff\xff\xff\xff".to_vec(),
        ] {
            assert_eq!(normalized_exif(&bytes), bytes);
        }
    }

    #[test]
    fn source_guard_detects_same_size_edits_even_with_restored_timestamp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("source.png");
        fs::write(&path, b"original").unwrap();
        let metadata = fs::metadata(&path).unwrap();
        verify_source_unchanged(&path, b"original", &metadata).unwrap();
        fs::write(&path, b"modified").unwrap();
        filetime::set_file_mtime(&path, FileTime::from_last_modification_time(&metadata)).unwrap();
        assert!(matches!(
            verify_source_unchanged(&path, b"original", &metadata),
            Err(OptimizeError::Replace(_))
        ));
        assert_eq!(fs::read(&path).unwrap(), b"modified");
    }

    #[cfg(windows)]
    #[test]
    fn source_handle_blocks_in_place_writers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("source.png");
        fs::write(&path, b"original").unwrap();
        let handle = open_source(&path).unwrap();
        assert!(fs::OpenOptions::new().write(true).open(&path).is_err());
        drop(handle);
        assert!(fs::OpenOptions::new().write(true).open(&path).is_ok());
    }

    #[test]
    fn effort_levels_are_ordered() {
        assert!(effort_index(OptimizationLevel::Fast) < effort_index(OptimizationLevel::Normal));
        assert!(effort_index(OptimizationLevel::Normal) < effort_index(OptimizationLevel::Extra));
        assert!(effort_index(OptimizationLevel::Extra) < effort_index(OptimizationLevel::Extreme));
        assert_eq!(webp_effort(OptimizationLevel::Fast), (0, 1));
        assert_eq!(webp_effort(OptimizationLevel::Normal), (3, 2));
        assert_eq!(webp_effort(OptimizationLevel::Extra), (5, 4));
        assert_eq!(webp_effort(OptimizationLevel::Extreme), (6, 10));
    }

    fn active_token() -> Arc<CancellationToken> {
        Arc::new(CancellationToken::new())
    }

    fn cancelled_token() -> Arc<CancellationToken> {
        let token = active_token();
        assert!(token.cancel());
        token
    }

    #[test]
    fn cancellation_and_commit_are_one_atomic_boundary() {
        let cancelled = cancelled_token();
        assert!(matches!(
            cancelled.begin_commit(),
            Err(OptimizeError::Cancelled)
        ));

        let committing = active_token();
        committing.begin_commit().unwrap();
        assert!(!committing.cancel());
        committing.finish_commit();
        assert!(!committing.cancel());
    }

    #[test]
    fn jpeg_metadata_segments_are_removed_without_touching_scan_data() {
        let bytes = [
            vec![0xFF, 0xD8, 0xFF, 0xE1, 0, 4, 1, 2],
            vec![0xFF, 0xDA, 0, 2, 9, 8, 7],
        ]
        .concat();
        let stripped = strip_jpeg_metadata(&bytes);
        assert!(!stripped.windows(2).any(|pair| pair == [0xFF, 0xE1]));
        assert!(stripped.ends_with(&[9, 8, 7]));
    }

    #[test]
    fn exif_orientation_is_normalized_before_lossy_encoding() {
        let base = source_bytes(ImageFormat::Jpeg);
        let exif_payload: Vec<u8> = [
            b"Exif\0\0".as_slice(),
            &[
                b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0,
                0, 0, 0,
            ],
        ]
        .concat();
        let mut oriented = vec![0xFF, 0xD8, 0xFF, 0xE1];
        oriented.extend_from_slice(&((exif_payload.len() + 2) as u16).to_be_bytes());
        oriented.extend_from_slice(&exif_payload);
        oriented.extend_from_slice(&base[2..]);
        let decoded = decode_oriented(&oriented).unwrap();
        assert_eq!(decoded.dimensions(), (240, 320));
    }

    #[test]
    fn all_supported_formats_replace_only_with_smaller_valid_output() {
        for (format, extension) in [
            (ImageFormat::Jpeg, "jpg"),
            (ImageFormat::Png, "png"),
            (ImageFormat::Webp, "webp"),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join(format!("sample.{extension}"));
            let original = source_bytes(format);
            fs::write(&path, &original).unwrap();
            let outcome =
                optimize_file(&path, format, &AppSettings::default(), active_token()).unwrap();
            let OptimizeOutcome::Replaced { result_bytes, .. } = outcome else {
                panic!("{format:?} should become smaller")
            };
            let output = fs::read(&path).unwrap();
            assert_eq!(result_bytes, output.len() as u64);
            assert!(output.len() < original.len());
            assert_eq!(
                image::load_from_memory(&output).unwrap().dimensions(),
                (320, 240)
            );
            if format == ImageFormat::Png {
                assert!(image::load_from_memory(&output)
                    .unwrap()
                    .to_rgba8()
                    .pixels()
                    .any(|pixel| pixel[3] < 255));
            }
        }
    }

    #[test]
    fn lossy_encoders_preserve_supported_metadata_when_requested() {
        let settings = AppSettings {
            strip_metadata: false,
            ..AppSettings::default()
        };

        let jpeg = jpeg_with_app2(
            &source_bytes(ImageFormat::Jpeg),
            b"ICC_PROFILE\0profile-data",
        );
        let (jpeg_output, _) = encoded_bytes(&jpeg, ImageFormat::Jpeg, &settings).unwrap();
        assert!(jpeg_output
            .windows(11)
            .any(|window| window == b"ICC_PROFILE"));

        let png = png_with_chunk(&source_bytes(ImageFormat::Png), b"tEXt", b"Author\0PicoPic");
        let (png_output, _) = encoded_bytes(&png, ImageFormat::Png, &settings).unwrap();
        assert!(png_output
            .windows(14)
            .any(|window| window == b"Author\0PicoPic"));

        let webp = source_bytes(ImageFormat::Webp);
        let (webp_output, _) = encoded_bytes(&webp, ImageFormat::Webp, &settings).unwrap();
        assert!(webp_chunks(&webp_output)
            .unwrap()
            .iter()
            .any(|(kind, _)| *kind == b"EXIF"));
    }

    #[test]
    fn lossless_webp_is_reencoded_without_dropping_requested_metadata() {
        let source = source_bytes(ImageFormat::Webp);
        let defaults = AppSettings::default();
        let settings = AppSettings {
            strip_metadata: false,
            webp: crate::models::FormatQualitySettings {
                lossy: false,
                ..defaults.webp
            },
            ..defaults
        };
        let (output, dimensions) = encoded_bytes(&source, ImageFormat::Webp, &settings).unwrap();
        assert_eq!(dimensions, (320, 240));
        assert!(webp_is_lossless(&output));
        assert!(webp_chunks(&output)
            .unwrap()
            .iter()
            .any(|(kind, _)| *kind == b"EXIF"));
    }

    #[test]
    fn lossless_container_mode_keeps_unchanged_source_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.jpg");
        let source = source_bytes(ImageFormat::Jpeg);
        fs::write(&path, &source).unwrap();
        let defaults = AppSettings::default();
        let settings = AppSettings {
            strip_metadata: false,
            jpeg: crate::models::FormatQualitySettings {
                lossy: false,
                ..defaults.jpeg
            },
            ..defaults
        };
        assert_eq!(
            optimize_file(&path, ImageFormat::Jpeg, &settings, active_token()).unwrap(),
            OptimizeOutcome::Unchanged {
                original_bytes: source.len() as u64
            }
        );
        assert_eq!(fs::read(path).unwrap(), source);
    }

    #[test]
    fn temporary_output_is_validated_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".picopic-invalid.png");
        fs::write(&path, b"not a png despite the extension").unwrap();
        let error =
            validate_temporary_output(&path, ImageFormat::Png, 10_000, (320, 240)).unwrap_err();
        assert!(matches!(error, OptimizeError::Validation(_)));
    }

    #[test]
    fn timestamp_is_prepared_before_replacement_and_survives_commit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.png");
        fs::write(&path, source_bytes(ImageFormat::Png)).unwrap();
        let expected = FileTime::from_unix_time(1_700_000_000, 0);
        filetime::set_file_mtime(&path, expected).unwrap();
        let settings = AppSettings {
            preserve_timestamps: true,
            ..AppSettings::default()
        };

        let outcome = optimize_file(&path, ImageFormat::Png, &settings, active_token()).unwrap();

        assert!(matches!(outcome, OptimizeOutcome::Replaced { .. }));
        let actual = FileTime::from_last_modification_time(&fs::metadata(&path).unwrap());
        assert_eq!(actual, expected);
    }

    #[test]
    fn cancellation_preserves_source_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.png");
        let original = source_bytes(ImageFormat::Png);
        fs::write(&path, &original).unwrap();
        let error = optimize_file(
            &path,
            ImageFormat::Png,
            &AppSettings::default(),
            cancelled_token(),
        )
        .unwrap_err();
        assert!(matches!(error, OptimizeError::Cancelled));
        assert_eq!(fs::read(path).unwrap(), original);
    }

    #[test]
    fn readonly_source_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.jpg");
        let original = source_bytes(ImageFormat::Jpeg);
        fs::write(&path, &original).unwrap();
        let original_permissions = fs::metadata(&path).unwrap().permissions();
        let mut readonly_permissions = original_permissions.clone();
        readonly_permissions.set_readonly(true);
        fs::set_permissions(&path, readonly_permissions).unwrap();
        let error = optimize_file(
            &path,
            ImageFormat::Jpeg,
            &AppSettings::default(),
            active_token(),
        )
        .unwrap_err();
        assert!(matches!(error, OptimizeError::Replace(_)));
        assert_eq!(fs::read(&path).unwrap(), original);
        fs::set_permissions(&path, original_permissions).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn locked_source_is_preserved_and_temporary_output_is_cleaned() {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.png");
        let original = source_bytes(ImageFormat::Png);
        fs::write(&path, &original).unwrap();
        let _lock = fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .open(&path)
            .unwrap();

        let error = optimize_file(
            &path,
            ImageFormat::Png,
            &AppSettings::default(),
            active_token(),
        )
        .unwrap_err();
        assert!(matches!(error, OptimizeError::Replace(_)));
        assert_eq!(fs::read(&path).unwrap(), original);
        assert!(!fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .any(|entry| entry.file_name().to_string_lossy().starts_with(".picopic-")));
    }
}
