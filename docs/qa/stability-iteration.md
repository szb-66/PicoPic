# Stability iteration (2026-10-19)

## Scope

- Correct EXIF transpose/transverse (5/7), normalize preserved IFD0 orientation after lossy pixel transforms, including JPEG/PNG/WebP and both TIFF byte orders.
- Refuse metadata removal in lossless mode when a non-default EXIF orientation is needed. The source remains unchanged; no implicit lossy conversion occurs.
- Keep a Windows source handle that denies in-place writes during processing. Recheck source bytes, size, modification time and readonly state immediately before commit. Path replacement by another process is still possible between the final check and ReplaceFileW; this is mitigation, not a filesystem compare-and-swap guarantee.
- Reserve active paths atomically with queue insertion. Reservations survive clearing rows and are released on all worker exit paths. Cancellation tokens are registered before publishing jobs.
- Snapshot settings at add/retry invocation. Serialize settings persistence and runtime updates.
- Reject inputs above 128 MiB or 40 million pixels. Pixel dimensions are checked before full decode. These limits reduce exposure but do not constitute a strict total memory budget; repeated decode and codec allocations remain future performance work.
- Add Windows CI for Rust formatting, Clippy, tests, frontend checks and a desktop executable build. No installer release or merge is part of this iteration.

## Regression coverage

- All EXIF orientations 2–8, both TIFF byte orders, all three image formats; preserved metadata must report orientation 1 after lossy encoding.
- Pixel-level transpose/transverse checks, lossless orientation preservation/refusal, malformed EXIF robustness.
- Same-size source edits with restored timestamps; Windows in-place writer exclusion.
- Concurrent same-path additions and clear/re-add reservation lifecycle.
- Oversized input rejection before reading file contents.

## Validation

Frontend typecheck, 12 unit tests, production build, 4 Sites packaging tests and 2 SEO tests passed locally. Rust and Windows-specific checks are recorded separately in the PR/CI results; historical QA reports are not evidence for this revision.

No user image corpus was available in this workspace. Compression ratio, visual quality and performance on real design assets have not been benchmarked in this iteration.
