export type ImageFormat = "jpeg" | "png" | "webp";
export type JobStatus =
  | "queued"
  | "processing"
  | "success"
  | "unchanged"
  | "skipped"
  | "failed"
  | "cancelled";
export type OptimizationLevel = "fast" | "normal" | "extra" | "extreme";

export interface FormatQualitySettings {
  lossy: boolean;
  quality: number;
}

export interface AppSettings {
  schemaVersion: 1;
  recursiveFolders: boolean;
  stripMetadata: boolean;
  preserveTimestamps: boolean;
  jpeg: FormatQualitySettings;
  png: FormatQualitySettings;
  webp: FormatQualitySettings;
  optimizationLevel: OptimizationLevel;
}

export interface JobRecord {
  id: string;
  path: string;
  fileName: string;
  format: ImageFormat | null;
  status: JobStatus;
  originalBytes: number;
  resultBytes: number | null;
  savedBytes: number;
  savedPercent: number;
  message: string | null;
  startedAt: string | null;
  completedAt: string | null;
}

export interface ScanWarning {
  path: string;
  reason: string;
}

export interface AddResult {
  jobs: JobRecord[];
  warnings: ScanWarning[];
}

export interface QueueSummary {
  total: number;
  active: number;
  successful: number;
  originalBytes: number;
  resultBytes: number;
  savedBytes: number;
  savedPercent: number;
}

export const DEFAULT_SETTINGS: AppSettings = {
  schemaVersion: 1,
  recursiveFolders: true,
  stripMetadata: true,
  preserveTimestamps: false,
  jpeg: { lossy: true, quality: 75 },
  png: { lossy: true, quality: 75 },
  webp: { lossy: true, quality: 75 },
  optimizationLevel: "normal",
};

export function validateSettings(settings: AppSettings): boolean {
  return (
    settings.schemaVersion === 1 &&
    [settings.jpeg, settings.png, settings.webp].every(
      ({ quality }) => Number.isInteger(quality) && quality >= 1 && quality <= 100,
    ) &&
    ["fast", "normal", "extra", "extreme"].includes(settings.optimizationLevel)
  );
}

export function summarizeJobs(jobs: JobRecord[]): QueueSummary {
  const successful = jobs.filter((job) => job.status === "success");
  const originalBytes = successful.reduce((sum, job) => sum + job.originalBytes, 0);
  const resultBytes = successful.reduce((sum, job) => sum + (job.resultBytes ?? 0), 0);
  const savedBytes = Math.max(0, originalBytes - resultBytes);
  return {
    total: jobs.length,
    active: jobs.filter((job) => job.status === "queued" || job.status === "processing").length,
    successful: successful.length,
    originalBytes,
    resultBytes,
    savedBytes,
    savedPercent: originalBytes > 0 ? (savedBytes / originalBytes) * 100 : 0,
  };
}
