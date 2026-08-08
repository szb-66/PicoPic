import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { demoJobs } from "./demoData";
import {
  DEFAULT_SETTINGS,
  summarizeJobs,
  validateSettings,
  type AddResult,
  type AppSettings,
  type JobRecord,
  type QueueSummary,
} from "./types";

export interface Backend {
  loadSettings(): Promise<AppSettings>;
  saveSettings(settings: AppSettings): Promise<AppSettings>;
  addPaths(paths: string[]): Promise<AddResult>;
  cancelJobs(ids: string[]): Promise<void>;
  retryJobs(ids: string[]): Promise<void>;
  clearJobs(scope: "completed" | "all"): Promise<JobRecord[]>;
  pickFiles(): Promise<string[]>;
  pickFolder(): Promise<string[]>;
  onJobUpdated(callback: (job: JobRecord) => void): Promise<UnlistenFn>;
  onSummary(callback: (summary: QueueSummary) => void): Promise<UnlistenFn>;
  onNativeDrop(callback: (paths: string[]) => void): Promise<UnlistenFn>;
  initialJobs(): Promise<JobRecord[]>;
}

export function isTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

const nativeBackend: Backend = {
  loadSettings: () => invoke("load_settings"),
  saveSettings: (settings) => invoke("save_settings", { settings }),
  addPaths: (paths) => invoke("add_paths", { paths }),
  cancelJobs: (ids) => invoke("cancel_jobs", { jobIds: ids }),
  retryJobs: (ids) => invoke("retry_jobs", { jobIds: ids }),
  clearJobs: (scope) => invoke("clear_jobs", { scope }),
  pickFiles: async () => {
    const selection = await open({ multiple: true, directory: false, filters: [{ name: "图片", extensions: ["jpg", "jpeg", "png", "webp"] }] });
    return selection ? (Array.isArray(selection) ? selection : [selection]) : [];
  },
  pickFolder: async () => {
    const selection = await open({ multiple: false, directory: true });
    return selection ? [selection] : [];
  },
  onJobUpdated: (callback) => listen<JobRecord>("job://updated", ({ payload }) => callback(payload)),
  onSummary: (callback) => listen<QueueSummary>("queue://summary", ({ payload }) => callback(payload)),
  onNativeDrop: async (callback) =>
    getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") callback(event.payload.paths);
    }),
  initialJobs: () => invoke("list_jobs"),
};

export function createBrowserBackend(seedJobs: JobRecord[] = [], completionDelay = 700): Backend {
  let browserSettings = structuredClone(DEFAULT_SETTINGS);
  let browserJobs = structuredClone(seedJobs);
  const jobListeners = new Set<(job: JobRecord) => void>();
  const summaryListeners = new Set<(summary: QueueSummary) => void>();

  const notify = (): void => {
    summaryListeners.forEach((listener) => listener(summarizeJobs(browserJobs)));
    browserJobs.forEach((job) => jobListeners.forEach((listener) => listener(structuredClone(job))));
  };

  return {
    loadSettings: async () => structuredClone(browserSettings),
    saveSettings: async (settings) => {
      if (!validateSettings(settings)) throw new Error("设置值无效");
      browserSettings = structuredClone(settings);
      return structuredClone(browserSettings);
    },
    addPaths: async (paths) => {
      const knownPaths = new Set(browserJobs.map(({ path }) => path.toLowerCase()));
      const uniquePaths = paths.filter((path) => {
        const normalized = path.toLowerCase();
        if (knownPaths.has(normalized)) return false;
        knownPaths.add(normalized);
        return true;
      });
      const added = uniquePaths.map((path, index): JobRecord => ({
        id: `browser-${Date.now()}-${index}`,
        path,
        fileName: path.split(/[\\/]/).pop() || "image.png",
        format: path.toLowerCase().endsWith(".jpg") || path.toLowerCase().endsWith(".jpeg") ? "jpeg" : path.toLowerCase().endsWith(".webp") ? "webp" : "png",
        status: "queued",
        originalBytes: 680_000 + index * 120_000,
        resultBytes: null,
        savedBytes: 0,
        savedPercent: 0,
        message: null,
        startedAt: null,
        completedAt: null,
      }));
      browserJobs = [...browserJobs, ...added];
      notify();
      window.setTimeout(() => {
        browserJobs = browserJobs.map((job) =>
          added.some(({ id }) => id === job.id) && job.status === "queued"
            ? { ...job, status: "success", resultBytes: Math.round(job.originalBytes * 0.34), savedBytes: Math.round(job.originalBytes * 0.66), savedPercent: 66, completedAt: new Date().toISOString() }
            : job,
        );
        notify();
      }, completionDelay);
      return { jobs: structuredClone(added), warnings: [] };
    },
    cancelJobs: async (ids) => {
      browserJobs = browserJobs.map((job) => (ids.includes(job.id) && (job.status === "queued" || job.status === "processing") ? { ...job, status: "cancelled", completedAt: new Date().toISOString() } : job));
      notify();
    },
    retryJobs: async (ids) => {
      browserJobs = browserJobs.map((job) => (ids.includes(job.id) && !["queued", "processing"].includes(job.status) ? { ...job, status: "queued", resultBytes: null, savedBytes: 0, savedPercent: 0, message: null, completedAt: null } : job));
      notify();
    },
    clearJobs: async (scope) => {
      browserJobs = scope === "all" ? [] : browserJobs.filter((job) => job.status === "queued" || job.status === "processing");
      notify();
      return structuredClone(browserJobs);
    },
    pickFiles: async () => ["C:\\Images\\new-photo.jpg", "C:\\Images\\new-graphic.png"],
    pickFolder: async () => ["C:\\Images\\Website"],
    onJobUpdated: async (callback) => {
      jobListeners.add(callback);
      return () => jobListeners.delete(callback);
    },
    onSummary: async (callback) => {
      summaryListeners.add(callback);
      return () => summaryListeners.delete(callback);
    },
    onNativeDrop: async () => () => undefined,
    initialJobs: async () => structuredClone(browserJobs),
  };
}

const browserBackend = createBrowserBackend(new URLSearchParams(window.location.search).get("demo") === "list" ? demoJobs() : []);

export const backend: Backend = isTauriRuntime() ? nativeBackend : browserBackend;
