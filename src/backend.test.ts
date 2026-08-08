import { describe, expect, it, vi } from "vitest";
import { createBrowserBackend } from "./backend";
import { DEFAULT_SETTINGS } from "./types";

describe("browser backend adapter", () => {
  it("persists valid settings and rejects invalid quality transactionally", async () => {
    const backend = createBrowserBackend();
    const valid = { ...DEFAULT_SETTINGS, png: { lossy: true, quality: 60 } };
    await expect(backend.saveSettings(valid)).resolves.toEqual(valid);
    await expect(backend.saveSettings({ ...valid, png: { lossy: true, quality: 101 } })).rejects.toThrow("设置值无效");
    await expect(backend.loadSettings()).resolves.toEqual(valid);
  });

  it("deduplicates, cancels, retries, completes, summarizes, and clears jobs", async () => {
    vi.useFakeTimers();
    const backend = createBrowserBackend([], 10);
    const summaries: number[] = [];
    await backend.onSummary((summary) => summaries.push(summary.active));

    const first = await backend.addPaths(["C:\\Images\\photo.jpg", "C:\\Images\\photo.jpg"]);
    expect(first.jobs).toHaveLength(1);
    await backend.cancelJobs([first.jobs[0].id]);
    expect((await backend.initialJobs())[0].status).toBe("cancelled");
    await backend.retryJobs([first.jobs[0].id]);
    expect((await backend.initialJobs())[0].status).toBe("queued");

    await vi.advanceTimersByTimeAsync(20);
    expect((await backend.initialJobs())[0].status).toBe("success");
    expect(summaries).toContain(0);
    await expect(backend.clearJobs("completed")).resolves.toEqual([]);
    expect(await backend.initialJobs()).toEqual([]);
    vi.useRealTimers();
  });
});
