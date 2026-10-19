use crate::{
    models::{AddResult, AppSettings, JobRecord, JobStatus, QueueSummary},
    optimizer::{optimize_file, CancellationToken, OptimizeError, OptimizeOutcome},
    scanner::scan_paths,
    settings::SettingsStore,
};
use parking_lot::{Mutex, RwLock};
use std::{
    collections::{HashMap, HashSet},
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
    pub active_paths: Arc<Mutex<HashSet<String>>>,
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
            active_paths: Arc::new(Mutex::new(HashSet::new())),
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

// A reservation outlives clearing the visible list and is released on every exit.
struct PathReservation {
    paths: Arc<Mutex<HashSet<String>>>,
    path: String,
    cancellations: Arc<Mutex<HashMap<String, Arc<CancellationToken>>>>,
    id: String,
}

impl Drop for PathReservation {
    fn drop(&mut self) {
        self.cancellations.lock().remove(&self.id);
        self.paths.lock().remove(&self.path);
    }
}

fn reserve_job(state: &AppState, record: JobRecord, token: Arc<CancellationToken>) -> bool {
    let mut jobs = state.jobs.lock();
    // Both checking existing rows and reserving active paths occur while holding
    // the jobs lock, so concurrent add_paths calls cannot insert duplicates.
    if jobs.values().any(|job| job.path == record.path) {
        return false;
    }
    if record.status == JobStatus::Queued && !state.active_paths.lock().insert(record.path.clone())
    {
        return false;
    }
    if record.status == JobStatus::Queued {
        state.cancellations.lock().insert(record.id.clone(), token);
    }
    jobs.insert(record.id.clone(), record);
    true
}

fn spawn_job(
    app: AppHandle,
    state: AppState,
    id: String,
    path: String,
    settings: AppSettings,
    cancellation: Arc<CancellationToken>,
) {
    let reservation = PathReservation {
        paths: state.active_paths.clone(),
        path,
        cancellations: state.cancellations.clone(),
        id: id.clone(),
    };
    tauri::async_runtime::spawn(async move {
        let reservation = reservation;
        let permit = match state.semaphore.clone().acquire_owned().await {
            Ok(value) => value,
            Err(_) => return,
        };
        if cancellation.is_cancelled() {
            if let Some(job) = state.jobs.lock().get_mut(&id) {
                job.status = JobStatus::Cancelled;
                job.completed_at = Some(now());
            }
            drop(reservation);
            emit_job(&app, &state, &id);
            return;
        }
        let (path, format) = {
            let mut jobs = state.jobs.lock();
            let Some(job) = jobs.get_mut(&id) else { return };
            let Some(format) = job.format else { return };
            job.status = JobStatus::Processing;
            job.started_at = Some(now());
            job.message = Some("正在压缩并验证输出".into());
            (PathBuf::from(&job.path), format)
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
        drop(reservation);
        emit_job(&app, &state, &id);
    });
}

#[tauri::command]
pub async fn add_paths(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
) -> Result<AddResult, String> {
    let settings = state.settings.read().clone();
    let recursive = settings.recursive_folders;
    let (candidates, warnings) =
        tauri::async_runtime::spawn_blocking(move || scan_paths(&paths, recursive))
            .await
            .map_err(|error| error.to_string())?;
    let state = state.inner().clone();
    let mut added = Vec::new();
    for candidate in candidates {
        let path_string = candidate.path.display().to_string();
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
        let token = Arc::new(CancellationToken::new());
        if !reserve_job(&state, record.clone(), token.clone()) {
            continue;
        }
        added.push(record.clone());
        emit_job(&app, &state, &id);
        if status == JobStatus::Queued {
            spawn_job(
                app.clone(),
                state.clone(),
                id,
                record.path,
                settings.clone(),
                token,
            );
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
    let settings = state.settings.read().clone();
    for id in job_ids {
        let token = Arc::new(CancellationToken::new());
        let retry = {
            let mut jobs = state.jobs.lock();
            if let Some(job) = jobs.get_mut(&id) {
                if job.status.is_terminal()
                    && job.format.is_some()
                    && state.active_paths.lock().insert(job.path.clone())
                {
                    job.status = JobStatus::Queued;
                    job.result_bytes = None;
                    job.saved_bytes = 0;
                    job.saved_percent = 0.0;
                    job.message = None;
                    job.started_at = None;
                    job.completed_at = None;
                    state.cancellations.lock().insert(id.clone(), token.clone());
                    Some(job.path.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(path) = retry {
            emit_job(&app, &state, &id);
            spawn_job(
                app.clone(),
                state.clone(),
                id,
                path,
                settings.clone(),
                token,
            );
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
        let mut jobs = state.jobs.lock();
        for flag in state.cancellations.lock().values() {
            flag.cancel();
        }
        jobs.clear();
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
    let mut current = state.settings.write();
    state.store.save(&settings)?;
    *current = settings.clone();
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queued_record(id: &str) -> JobRecord {
        JobRecord {
            id: id.into(),
            path: "same.png".into(),
            file_name: "same.png".into(),
            format: Some(crate::models::ImageFormat::Png),
            status: JobStatus::Queued,
            original_bytes: 100,
            result_bytes: None,
            saved_bytes: 0,
            saved_percent: 0.0,
            message: None,
            started_at: None,
            completed_at: None,
        }
    }

    #[test]
    fn concurrent_additions_reserve_one_job_per_path() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(SettingsStore::new(dir.path().join("settings.json")));
        let barrier = Arc::new(std::sync::Barrier::new(8));
        let threads: Vec<_> = (0..8)
            .map(|index| {
                let state = state.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    reserve_job(
                        &state,
                        queued_record(&index.to_string()),
                        Arc::new(CancellationToken::new()),
                    )
                })
            })
            .collect();
        let count = threads
            .into_iter()
            .map(|thread| usize::from(thread.join().unwrap()))
            .sum::<usize>();
        assert_eq!(count, 1);
        assert_eq!(state.jobs.lock().len(), 1);
        assert_eq!(state.cancellations.lock().len(), 1);
    }

    #[test]
    fn clearing_rows_does_not_release_a_running_path() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(SettingsStore::new(dir.path().join("settings.json")));
        assert!(reserve_job(
            &state,
            queued_record("old"),
            Arc::new(CancellationToken::new())
        ));
        let reservation = PathReservation {
            paths: state.active_paths.clone(),
            path: "same.png".into(),
            cancellations: state.cancellations.clone(),
            id: "old".into(),
        };
        state.jobs.lock().clear();
        assert!(!reserve_job(
            &state,
            queued_record("new"),
            Arc::new(CancellationToken::new())
        ));
        drop(reservation);
        assert!(state.cancellations.lock().is_empty());
        assert!(reserve_job(
            &state,
            queued_record("new"),
            Arc::new(CancellationToken::new())
        ));
    }

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
