## 1. Project foundation

- [x] 1.1 Verify or install the Windows Tauri development prerequisites and record the development commands
- [x] 1.2 Scaffold the React/TypeScript/Vite frontend and Tauri 2 Rust application without packaging targets
- [x] 1.3 Define mirrored Rust/TypeScript domain types, default settings, validation, and the browser/native backend adapter

## 2. Native compression core

- [x] 2.1 Implement path scanning, canonical deduplication, signature validation, animated-format rejection, and scan warnings
- [x] 2.2 Implement versioned transactional settings load/save with corruption fallback and validation tests
- [x] 2.3 Implement JPEG, PNG, and WebP codec adapters with per-format lossy settings, quality, effort mapping, metadata handling, and output validation
- [x] 2.4 Implement same-directory temporary output, strictly-smaller checks, Windows atomic replacement, permission/timestamp handling, and failure cleanup
- [x] 2.5 Implement the bounded job queue, cancellation, retry, clear operations, summaries, Tauri commands, and snapshot events

## 3. Desktop experience

- [x] 3.1 Build the ImageOptim-derived Fluent empty/drop state and native drag/drop plus file/folder picker interactions
- [x] 3.2 Build the compact job table with all statuses, before/after sizes, savings, reasons, aggregate summary, clear, cancel, and re-optimize controls
- [x] 3.3 Build the transactional General, Quality, and Optimization Speed preferences dialog with accessibility and persisted settings integration
- [x] 3.4 Add polished responsive styling, Fluent icons, keyboard/focus states, loading/empty/success/error states, and deterministic browser demo data

## 4. Verification

- [x] 4.1 Add and pass frontend tests for queue presentation, summaries, settings transactions, validation, and backend adapter behavior
- [x] 4.2 Add and pass Rust unit/integration tests for scanning, formats, settings, compression, cancellation, safe replacement, and source preservation
- [x] 4.3 Run the frontend and Tauri development checks, manually smoke-test native drag/drop and JPEG/PNG/WebP replacement, and document any environment limitations
- [x] 4.4 Capture matching reference and implementation states, iterate on P0/P1/P2 visual differences, and create a passing project-root design-qa.md
- [x] 4.5 Run strict OpenSpec validation and verify implementation coverage against every requirement and scenario
