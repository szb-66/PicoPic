## Context

The repository contains only OpenSpec scaffolding. The target is a Windows-first local desktop application whose visual reference is the supplied ImageOptim empty state, task list, and three preferences screenshots. The product must mutate user files safely, keep compression off the UI thread, and remain testable without packaging. See `proposal.md` and the three capability specs for observable behavior.

The available machine has Node.js and OpenSpec, but Rust/MSVC and a detectable WebView2 development environment must be established before native verification.

## Goals / Non-Goals

**Goals:**
- Keep the webview responsible for presentation and interaction state while Rust exclusively owns scanning, codecs, queue execution, settings validation, and file replacement.
- Use one versioned data contract mirrored in Rust and TypeScript, with explicit job events rather than frontend polling.
- Make every destructive step conditional on successful same-directory temporary output validation.
- Provide a browser-runnable frontend adapter so UI tests and visual QA do not depend on native APIs, while retaining native behavior in `tauri dev`.

**Non-Goals:**
- No installer, auto-update, shell integration, tray process, cloud API, resizing, conversion, undo history, backup files, animation optimization, or non-Windows platform validation.
- No arbitrary frontend filesystem permissions and no execution of external user-provided binaries.

## Decisions

### Decision: Tauri 2 with a React/TypeScript frontend and Rust core
Tauri provides native Windows file-drop paths and a smaller runtime footprint than bundling Chromium. React keeps dense queue and preference state predictable. A thin `backend.ts` adapter selects Tauri IPC in the desktop runtime and a deterministic demo adapter in a normal browser for tests and visual QA.

Alternative considered: Electron with Sharp. It simplifies codec access but materially increases application size and duplicates a browser runtime, conflicting with the lightweight goal.

### Decision: Backend-owned queue with bounded workers
`AppState` owns settings, jobs, cancellation tokens, and a semaphore-gated asynchronous task scheduler. `add_paths` scans synchronously on a blocking worker, creates stable UUID jobs, emits snapshots, and spawns accepted jobs. Worker count is `min(available_parallelism, 4)` with at least one worker. Cancellation and replacement compete through one atomic token transition so a cancellation accepted before the commit boundary prevents replacement.

Alternative considered: frontend orchestration. It would expose paths and make cancellation/restart behavior harder to keep coherent across window lifecycle events.

### Decision: Format-specific codecs behind one optimizer interface
- Decode and inspect with `image`; read orientation/metadata with an EXIF parser.
- JPEG lossy encoding uses `mozjpeg-rs`; lossless-disabled mode only strips safely removable marker segments.
- PNG lossy mode uses `imagequant`, then `oxipng`; lossless mode uses `oxipng` only.
- Static WebP encoding uses `webp`; existing animation is rejected before decode/re-encode.

Each encoder receives immutable validated settings and returns encoded bytes plus expected dimensions. Quality stays independent from the effort preset. Exact compatible crate versions are locked in `Cargo.lock` after a compile-tested dependency set is selected.

Alternative considered: shipping CLI sidecars such as pngquant/jpegtran. In-process libraries avoid path quoting, process management, antivirus friction, and extra distributable binaries.

### Decision: Same-directory temporary output and Windows atomic replacement
The optimizer creates a unique dot-prefixed sibling temporary file, writes and flushes it, reads the file bytes back, decodes them again, verifies the format and display dimensions, and compares byte size. A Windows hidden attribute is intentionally avoided because the replacement file's attributes can become the source's attributes. Only a strictly smaller output can proceed. Permissions and optional timestamps are applied to the temporary output before the atomic commit boundary. On Windows the replacement layer uses `ReplaceFileW`; unsupported or failed atomic replacement is a hard job failure. A scope guard deletes temporary files on every non-commit path.

Alternative considered: delete then rename. It creates a data-loss window and is prohibited.

### Decision: Versioned, validated settings stored by Rust
Settings are serialized as versioned JSON under the Tauri application config directory. Writes use a sibling temporary file and rename. The frontend edits a draft and saves only on confirmation. Invalid updates are rejected; unreadable or unsupported persisted content falls back to defaults.

### Decision: ImageOptim structure with Windows Fluent presentation
The main window uses a large empty drop target that becomes a compact table. A fixed bottom command/status bar contains add, clear, retry, aggregate savings, and settings controls. Preferences are an in-app modal with segmented tabs for General, Quality, and Optimization Speed. Native window chrome is retained; macOS traffic lights are not imitated. Fluent System Icons are used through an icon package rather than handcrafted glyphs.

The supplied screenshots are the source of truth for hierarchy and density. Intentional deviations are limited to Windows chrome, Fluent control styling, English/Chinese copy normalization, and additional failure/cancel states required by the specs.

### Decision: Commands and events form the public application boundary
Commands are `add_paths`, `cancel_jobs`, `retry_jobs`, `clear_jobs`, `load_settings`, and `save_settings`. Shared payloads are `ImageFormat`, `JobStatus`, `JobRecord`, `FormatQualitySettings`, `AppSettings`, `OptimizationLevel`, `ScanWarning`, and `QueueSummary`. Events are `job://updated` and `queue://summary`; every payload is a full snapshot so missed intermediate events do not corrupt UI state.

## Risks / Trade-offs

- [Risk] Rust/MSVC/WebView2 are currently unavailable or undetected → Install official development prerequisites first; keep frontend work independently verifiable in the browser adapter.
- [Risk] Native codec crates may impose compiler or C-library constraints → Prefer pure Rust where available, pin versions, compile codecs early, and replace only the affected internal adapter if necessary.
- [Risk] Metadata preservation differs across codecs → Document preservation as best-effort for supported segments and test the default removal path rigorously.
- [Risk] Large images can cause memory spikes during decode → Bound concurrency to four, process one image per worker, and surface allocation/decode failure without touching the source.
- [Risk] Windows replacement can fail on network or unusual filesystems → Treat failure as terminal and preserve the original instead of adding a non-atomic fallback.
- [Risk] Browser visual QA cannot prove native file replacement → Separate visual/interaction QA from Rust integration tests and a final `tauri dev` smoke test.

## Migration Plan

1. Establish development prerequisites and scaffold the frontend/Tauri project without packaging.
2. Land shared types, settings, scanning, queue, codec adapters, and safe replacement behind tests.
3. Build the ImageOptim-derived interface and connect it through the backend adapter.
4. Run frontend, Rust, native development, OpenSpec, and visual QA gates.
5. Since this is a new application with no persisted legacy data, rollback is removal of the new project files; user images remain protected by the replacement contract.
