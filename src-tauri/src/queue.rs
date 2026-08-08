use crate::{
    models::{AddResult, AppSettings, JobRecord, JobStatus, QueueSummary},
    optimizer::{optimize_file, CancellationToken, OptimizeError, OptimizeOutcome},
    scanner::scan_paths,
    settings::SettingsStore,
};
use parking_lot::{Mutex, RwLock};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub jobs: Arc<Mutex<HashMap<String, JobRecord>>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub cancellations: Arc<Mutex<HashMap<String, Arc<CancellationToken>>>>,
    pub semaphore: Arc<Semaphore>,
    pub store: SettingsStore,
}

impl AppState {
    pub fn new(store: SettingsStore) -> Self {
        let settings = store.load();
        let worker_count = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
            .clamp(1, 4);
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(RwLock::new(settings)),
            cancellations: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(worker_count)),
            store,
        }
    }
}

fn now() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string()
}

fn emit_summary(app: &AppHandle, state: &AppState) {
    let jobs = state.jobs.lock();
    let summary = QueueSummary::from_jobs(jobs.values());
    let _ = app.emit("queue://summary", summary);
}

fn emit_job(app: &AppHandle, state: &AppState, id: &str) {
    if let Some(job) = state.jobs.lock().get(id).cloned() {
        let _ = app.emit("job://updated", job);
    }
    emit_summary(app, state);
}

fn spawn_job(app: AppHandle, state: AppState, id: String) {
    tauri::async_runtime::spawn(async move {
        let permit = match state.semaphore.clone().acquire_owned().await {
            Ok(value) => value,
            Err(_) => return,
        };
        let cancellation = state
            .cancellations
            .lock()
            .get(&id)
            .cloned()
            .unwrap_or_else(|| Arc::new(CancellationToken::new()));
        if cancellation.is_cancelled() {
            if let Some(job) = state.jobs.lock().get_mut(&id) {
                job.status = JobStatus::Cancelled;
                job.completed_at = Some(now());
            }
            emit_job(&app, &state, &id);
            return;
        }
        let (path, format, settings) = {
            let mut jobs = state.jobs.lock();
            let Some(job) = jobs.get_mut(&id) else { return };
            let Some(format) = job.format else { return };
            job.status = JobStatus::Processing;
            job.started_at = Some(now());
            job.message = Some("正在压缩并验证输出".into());
            (
                PathBuf::from(&job.path),
                format,
                state.settings.read().clone(),
            )
        };
        emit_job(&app, &state, &id);
        let cancel_for_worker = cancellation.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            optimize_file(&path, format, &settings, cancel_for_worker)
        })
        .await;
        drop(permit);
        let mut jobs = state.jobs.lock();
        let Some(job) = jobs.get_mut(&id) else { return };
        match result {
            Ok(Ok(OptimizeOutcome::Replaced {
                original_bytes,
                result_bytes,
            })) => {
                job.status = JobStatus::Success;
                job.original_bytes = original_bytes;
                job.result_bytes = Some(result_bytes);
                job.saved_bytes = original_bytes.saturating_sub(result_bytes);
                job.saved_percent = if original_bytes == 0 {
                    0.0
                } else {
                    job.saved_bytes as f64 / original_bytes as f64 * 100.0
                };
                job.message = None;
            }
            Ok(Ok(OptimizeOutcome::Unchanged { original_bytes })) => {
                job.status = JobStatus::Unchanged;
                job.original_bytes = original_bytes;
                job.result_bytes = Some(original_bytes);
                job.saved_bytes = 0;
                job.saved_percent = 0.0;
                job.message = Some("压缩结果没有更小，已保留原文件".into());
            }
            Ok(Err(OptimizeError::Cancelled)) => {
                job.status = JobStatus::Cancelled;
                job.message = Some("任务已取消，原文件未改变".into());
            }
            Ok(Err(error)) => {
                job.status = JobStatus::Failed;
                job.message = Some(error.to_string());
            }
            Err(error) => {
                job.status = JobStatus::Failed;
                job.message = Some(format!("压缩线程异常：{error}"));
            }
        }
        job.completed_at = Some(now());
        drop(jobs);
        state.cancellations.lock().remove(&id);
        emit_job(&app, &state, &id);
    });
}

#[tauri::command]
pub async fn add_paths(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
) -> Result<AddResult, String> {
    let recursive = state.settings.read().recursive_folders;
    let (candidates, warnings) =
        tauri::async_runtime::spawn_blocking(move || scan_paths(&paths, recursive))
            .await
            .map_err(|error| error.to_string())?;
    let state = state.inner().clone();
    let existing_paths: std::collections::HashSet<String> = state
        .jobs
        .lock()
        .values()
        .map(|job| job.path.clone())
        .collect();
    let mut added = Vec::new();
    for candidate in candidates {
        let path_string = candidate.path.display().to_string();
        if existing_paths.contains(&path_string)
            || added.iter().any(|job: &JobRecord| job.path == path_string)
        {
            continue;
        }
        let id = Uuid::new_v4().to_string();
        let status = if candidate.skip_reason.is_some() {
            JobStatus::Skipped
        } else {
            JobStatus::Queued
        };
        let record = JobRecord {
            id: id.clone(),
            path: path_string,
            file_name: candidate
                .path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| "image".into()),
            format: candidate.format,
            status,
            original_bytes: candidate.original_bytes,
            result_bytes: None,
            saved_bytes: 0,
            saved_percent: 0.0,
            message: candidate.skip_reason,
            started_at: None,
            completed_at: if status.is_terminal() {
                Some(now())
            } else {
                None
            },
        };
        state.jobs.lock().insert(id.clone(), record.clone());
        added.push(record);
        emit_job(&app, &state, &id);
        if status == JobStatus::Queued {
            state
                .cancellations
                .lock()
                .insert(id.clone(), Arc::new(CancellationToken::new()));
            spawn_job(app.clone(), state.clone(), id);
        }
    }
    Ok(AddResult {
        jobs: added,
        warnings,
    })
}

#[tauri::command]
pub fn list_jobs(state: tauri::State<'_, AppState>) -> Vec<JobRecord> {
    snapshot_jobs(state.inner())
}

fn snapshot_jobs(state: &AppState) -> Vec<JobRecord> {
    let mut jobs: Vec<_> = state.jobs.lock().values().cloned().collect();
    jobs.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    jobs
}

#[tauri::command]
pub fn cancel_jobs(app: AppHandle, state: tauri::State<'_, AppState>, job_ids: Vec<String>) {
    for id in job_ids {
        let cancelled = state
            .cancellations
            .lock()
            .get(&id)
            .is_some_and(|token| token.cancel());
        if let Some(job) = state.jobs.lock().get_mut(&id) {
            if cancelled && job.status == JobStatus::Queued {
                job.status = JobStatus::Cancelled;
                job.completed_at = Some(now());
                job.message = Some("任务已取消".into());
            }
        }
        emit_job(&app, state.inner(), &id);
    }
}

#[tauri::command]
pub fn retry_jobs(app: AppHandle, state: tauri::State<'_, AppState>, job_ids: Vec<String>) {
    let state = state.inner().clone();
    for id in job_ids {
        let retry = {
            let mut jobs = state.jobs.lock();
            if let Some(job) = jobs.get_mut(&id) {
                if job.status.is_terminal() && job.format.is_some() {
                    job.status = JobStatus::Queued;
                    job.result_bytes = None;
                    job.saved_bytes = 0;
                    job.saved_percent = 0.0;
                    job.message = None;
                    job.started_at = None;
                    job.completed_at = None;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        if retry {
            state
                .cancellations
                .lock()
                .insert(id.clone(), Arc::new(CancellationToken::new()));
            emit_job(&app, &state, &id);
            spawn_job(app.clone(), state.clone(), id);
        }
    }
}

#[tauri::command]
pub fn clear_jobs(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    scope: String,
) -> Vec<JobRecord> {
    if scope == "all" {
        for flag in state.cancellations.lock().values() {
            flag.cancel();
        }
        state.jobs.lock().clear();
    } else {
        state.jobs.lock().retain(|_, job| !job.status.is_terminal());
    }
    emit_summary(&app, state.inner());
    snapshot_jobs(state.inner())
}

#[tauri::command]
pub fn load_settings(state: tauri::State<'_, AppState>) -> AppSettings {
    state.settings.read().clone()
}

#[tauri::command]
pub fn save_settings(
    state: tauri::State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    settings.validate()?;
    state.store.save(&settings)?;
    *state.settings.write() = settings.clone();
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_counts_only_successful_bytes() {
        let success = JobRecord {
            id: "1".into(),
            path: "a".into(),
            file_name: "a".into(),
            format: None,
            status: JobStatus::Success,
            original_bytes: 100,
            result_bytes: Some(40),
            saved_bytes: 60,
            saved_percent: 60.0,
            message: None,
            started_at: None,
            completed_at: None,
        };
        let failed = JobRecord {
            status: JobStatus::Failed,
            original_bytes: 200,
            ..success.clone()
        };
        let summary = QueueSummary::from_jobs([&success, &failed].into_iter());
        assert_eq!(summary.saved_bytes, 60);
        assert_eq!(summary.successful, 1);
    }
}
