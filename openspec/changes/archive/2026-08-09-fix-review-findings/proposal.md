## Why

The initial implementation passes its automated checks but a code review found queue/UI divergence, cancellation and replacement race conditions, post-commit error reporting, weak temporary-output validation, and false-positive animation detection. These defects undermine the source-preservation contract of an in-place image optimizer and must be corrected before the original change is archived.

## What Changes

- Keep the frontend task list synchronized when terminal jobs are cleared.
- Make cancellation and source replacement share a commit boundary so a cancellation accepted before commit prevents replacement.
- Ensure failures reported after replacement do not contradict the source-preservation contract.
- Validate bytes read back from the same-directory temporary file before replacement and document the safe dot-prefixed temporary-file policy without a Windows hidden attribute.
- Parse PNG and WebP chunk structures when detecting animation.
- Align the bounded backend queue implementation with its documented ownership model where practical, or update the design decision to describe the equivalent scheduler accurately.
- Add regression tests for each corrected behavior.

## Capabilities

### New Capabilities

None. This change restores behavior already required by the existing desktop compression workflow and format-aware optimization specifications.

### Modified Capabilities

None. No product requirement changes are introduced; `skip_specs` is enabled because this is a conformance and safety repair.

## Impact

Affected areas are the React queue state, the Tauri queue/event boundary, Windows replacement coordination, temporary-file handling, static-format scanning, and their frontend/Rust tests. No packaging target, external service, or new runtime dependency is introduced.
