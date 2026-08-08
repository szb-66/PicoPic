# PicoPic Design QA

Status: **passed (visual QA only)**

## Scope

- Product: PicoPic Windows desktop image compressor
- Visual source: five ImageOptim reference captures supplied by the user
- Intended adaptation: preserve ImageOptim's compact drop zone, queue, status hierarchy, and three-part preferences model while applying Windows Fluent visual language
- Implementation viewport: 1004 x 850 for all captured states
- Native window: Tauri development build at 960 x 680 with a 720 x 520 minimum size

## Evidence

| State | Reference size | Implementation size |
| --- | --- | --- |
| Empty / drop | 1004 x 850 | 1004 x 850 |
| Queue / processing | 1004 x 850 | 1004 x 850 |
| Preferences / General | 1326 x 798 | 1004 x 850 |
| Preferences / Quality | 1326 x 798 | 1004 x 850 |
| Preferences / Speed | 1326 x 798 | 1004 x 850 |

The local QA captures were reviewed during development and removed before the public source release; this document retains the resulting assessment without publishing workstation paths or redundant binary evidence.

Every visual judgment was made from the corresponding combined comparison image rather than from isolated screenshots.

## Interaction checks

- Empty and populated queue states render without clipping at the configured desktop viewport.
- General, Quality, and Optimization Speed tabs are keyboard-focusable and switch to the correct panel.
- General settings use accessible switches; Quality and Speed controls expose slider labels and current values.
- Settings are transactional: changing `Preserve modification time`, selecting Cancel, and reopening the dialog restores the saved unchecked value.
- Queue actions, summaries, processing state, completed state, empty state, and reasons are present in deterministic browser demo data.
- Browser console inspection found no warnings or errors during the five captured states.
- The native Tauri development executable started successfully, remained responsive, and rendered through WebView2. It has since been stopped; see the incident note below.

## Visual assessment

- **P0 blockers:** none.
- **P1 major issues:** none.
- **P2 polish issues:** none.
- **P3 intentional differences:** Windows title treatment, Segoe UI typography, Fluent blue accents, card-based queue rows, and a modal preferences surface replace the macOS chrome and separate preferences window shown by ImageOptim.

The implementation retains the reference hierarchy: large central drop target, compact status-first queue, before/after savings columns, persistent bottom actions, and three clearly separated settings categories. Contrast, focus rings, hit targets, spacing, and disabled states are consistent and readable.

## Functional verification supporting the design

- `npm run typecheck`: passed
- `npm test -- --run`: 7 tests passed
- `npm run build`: passed
- `npm run test:sites`: 4 tests passed
- `cargo test`: 21 tests passed
- `openspec validate build-picopic-desktop-compressor --strict`: passed
- `tauri dev`: native `PicoPic` window started and responded

The same path ingestion, recursive scan, JPEG/PNG/WebP encoding, strictly-smaller rule, cancellation gate, and atomic replacement path are covered by Rust integration tests using real files on Windows; picker and drop handlers are additionally exercised through frontend tests and browser interaction checks.

## Native smoke-test incident

During automated native folder-picker testing, the system dialog confirmed its current folder instead of the intended workspace-only smoke directory. The native development process was stopped immediately after the incorrect selection was noticed. A read-only audit identified 117 PNG files with replacement timestamps from 2026-08-09 04:04:19 through 04:04:44; no non-PNG files were identified in that replacement window. The path-level inventory remains private and is excluded from the public repository. Because PicoPic intentionally performs in-place replacement without backups, the application cannot restore original bytes.
