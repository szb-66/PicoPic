## 1. Queue consistency

- [x] 1.1 Make clear operations return the remaining queue snapshot through native and browser adapters
- [x] 1.2 Update React queue state after clear and add a regression test proving terminal rows disappear

## 2. Safe replacement boundary

- [x] 2.1 Replace Boolean cancellation flags with an atomic running/cancelled/committing/committed state machine
- [x] 2.2 Validate bytes read back from the sibling temporary file and prepare permissions/timestamps before commit
- [x] 2.3 Add Rust regressions for cancellation-at-commit, temporary-byte validation, and post-replacement success semantics

## 3. Format detection and architecture alignment

- [x] 3.1 Parse PNG and WebP chunk boundaries for animation detection and cover payload-marker false positives
- [x] 3.2 Update the original design to document semaphore-gated task scheduling and dot-prefixed temporary files accurately

## 4. Verification

- [x] 4.1 Run formatter, type checking, frontend tests, Rust tests, build, Sites tests, and strict OpenSpec validation
- [x] 4.2 Smoke-test the development UI, confirm no console errors, and remove any obsolete code introduced during failed attempts
