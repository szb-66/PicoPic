import type { JobRecord } from "./types";

const base: Omit<JobRecord, "id" | "path">[] = [
  {
    fileName: "landing-hero.png",
    format: "png",
    status: "success",
    originalBytes: 1_402_580,
    resultBytes: 480_675,
    savedBytes: 921_905,
    savedPercent: 65.73,
    message: null,
    startedAt: "2026-08-09T03:10:00Z",
    completedAt: "2026-08-09T03:10:02Z",
  },
  {
    fileName: "product-photo.jpg",
    format: "jpeg",
    status: "processing",
    originalBytes: 799_646,
    resultBytes: null,
    savedBytes: 0,
    savedPercent: 0,
    message: "正在使用 JPEG 质量 75% 压缩",
    startedAt: "2026-08-09T03:10:03Z",
    completedAt: null,
  },
  {
    fileName: "dashboard.webp",
    format: "webp",
    status: "success",
    originalBytes: 1_805_156,
    resultBytes: 543_352,
    savedBytes: 1_261_804,
    savedPercent: 69.9,
    message: null,
    startedAt: "2026-08-09T03:10:00Z",
    completedAt: "2026-08-09T03:10:04Z",
  },
];

export function demoJobs(): JobRecord[] {
  return base.map((job, index) => ({ ...job, id: `demo-${index}`, path: `C:\\Images\\${job.fileName}` }));
}
