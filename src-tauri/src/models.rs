use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Processing,
    Success,
    Unchanged,
    Skipped,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Success | Self::Unchanged | Self::Skipped | Self::Failed | Self::Cancelled
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OptimizationLevel {
    Fast,
    Normal,
    Extra,
    Extreme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FormatQualitySettings {
    pub lossy: bool,
    pub quality: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub schema_version: u8,
    pub recursive_folders: bool,
    pub strip_metadata: bool,
    pub preserve_timestamps: bool,
    pub jpeg: FormatQualitySettings,
    pub png: FormatQualitySettings,
    pub webp: FormatQualitySettings,
    pub optimization_level: OptimizationLevel,
}

impl Default for AppSettings {
    fn default() -> Self {
        let format = FormatQualitySettings {
            lossy: true,
            quality: 75,
        };
        Self {
            schema_version: 1,
            recursive_folders: true,
            strip_metadata: true,
            preserve_timestamps: false,
            jpeg: format.clone(),
            png: format.clone(),
            webp: format,
            optimization_level: OptimizationLevel::Normal,
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("不支持的设置版本".into());
        }
        for (name, value) in [
            ("JPEG", &self.jpeg),
            ("PNG", &self.png),
            ("WebP", &self.webp),
        ] {
            if !(1..=100).contains(&value.quality) {
                return Err(format!("{name} 质量必须在 1 到 100 之间"));
            }
        }
        Ok(())
    }

    pub fn format(&self, format: ImageFormat) -> &FormatQualitySettings {
        match format {
            ImageFormat::Jpeg => &self.jpeg,
            ImageFormat::Png => &self.png,
            ImageFormat::Webp => &self.webp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRecord {
    pub id: String,
    pub path: String,
    pub file_name: String,
    pub format: Option<ImageFormat>,
    pub status: JobStatus,
    pub original_bytes: u64,
    pub result_bytes: Option<u64>,
    pub saved_bytes: u64,
    pub saved_percent: f64,
    pub message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanWarning {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddResult {
    pub jobs: Vec<JobRecord>,
    pub warnings: Vec<ScanWarning>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueueSummary {
    pub total: usize,
    pub active: usize,
    pub successful: usize,
    pub original_bytes: u64,
    pub result_bytes: u64,
    pub saved_bytes: u64,
    pub saved_percent: f64,
}

impl QueueSummary {
    pub fn from_jobs<'a>(jobs: impl Iterator<Item = &'a JobRecord>) -> Self {
        let mut summary = Self::default();
        for job in jobs {
            summary.total += 1;
            if matches!(job.status, JobStatus::Queued | JobStatus::Processing) {
                summary.active += 1;
            }
            if job.status == JobStatus::Success {
                summary.successful += 1;
                summary.original_bytes += job.original_bytes;
                summary.result_bytes += job.result_bytes.unwrap_or_default();
            }
        }
        summary.saved_bytes = summary.original_bytes.saturating_sub(summary.result_bytes);
        summary.saved_percent = if summary.original_bytes == 0 {
            0.0
        } else {
            summary.saved_bytes as f64 / summary.original_bytes as f64 * 100.0
        };
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_product_contract() {
        let settings = AppSettings::default();
        assert!(settings.recursive_folders && settings.strip_metadata);
        assert!(!settings.preserve_timestamps);
        assert_eq!(settings.jpeg.quality, 75);
        assert_eq!(settings.png.quality, 75);
        assert_eq!(settings.webp.quality, 75);
        assert_eq!(settings.optimization_level, OptimizationLevel::Normal);
    }

    #[test]
    fn invalid_quality_is_rejected() {
        let mut settings = AppSettings::default();
        settings.png.quality = 0;
        assert!(settings.validate().is_err());
    }
}
