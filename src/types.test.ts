import { describe, expect, it } from "vitest";
import { DEFAULT_SETTINGS, summarizeJobs, validateSettings, type JobRecord } from "./types";

function job(overrides: Partial<JobRecord>): JobRecord {
  return {
    id: "1", path: "C:\\image.png", fileName: "image.png", format: "png", status: "success",
    originalBytes: 1000, resultBytes: 400, savedBytes: 600, savedPercent: 60,
    message: null, startedAt: null, completedAt: null, ...overrides,
  };
}

describe("settings", () => {
  it("uses safe per-format defaults", () => {
    expect(DEFAULT_SETTINGS.jpeg).toEqual({ lossy: true, quality: 75 });
    expect(DEFAULT_SETTINGS.png).toEqual({ lossy: true, quality: 75 });
    expect(DEFAULT_SETTINGS.webp).toEqual({ lossy: true, quality: 75 });
  });

  it("rejects invalid quality", () => {
    expect(validateSettings({ ...DEFAULT_SETTINGS, png: { lossy: true, quality: 101 } })).toBe(false);
  });
});

describe("queue summary", () => {
  it("counts savings from successful replacements only", () => {
    const summary = summarizeJobs([job({}), job({ id: "2", status: "failed", originalBytes: 9999, resultBytes: null })]);
    expect(summary).toMatchObject({ total: 2, successful: 1, originalBytes: 1000, resultBytes: 400, savedBytes: 600, savedPercent: 60 });
  });
});
