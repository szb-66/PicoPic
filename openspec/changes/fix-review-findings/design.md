## Context

See `proposal.md` for motivation. PicoPic replaces user files in place, so the commit boundary must be explicit and every fallible preparation step must happen before it. The current frontend receives per-job and summary events, but deletion is not representable through either event. The existing design also describes a Tokio channel and a Windows-hidden temporary file although the implementation uses semaphore-gated tasks and dot-prefixed sibling files.

## Goals / Non-Goals

**Goals:**

- Make cancellation and replacement compete through one atomic state transition.
- Ensure no fallible permission, timestamp, or validation work remains after source replacement.
- Return the post-clear queue snapshot so the frontend can represent deletions deterministically.
- Parse container chunk boundaries for animation markers.
- Make architecture documentation describe the scheduler and temporary-file policy actually required by the product.

**Non-Goals:**

- Adding backups, undo, packaging, or a new queue persistence model.
- Changing compression quality, codec selection, or public product requirements.
- Introducing a Windows hidden attribute that could be inherited by the replacement source.

## Decisions

### Atomic cancellation token state

Replace the Boolean cancellation flag with a small atomic state machine: running, cancelled, committing, and committed. Cancellation atomically changes running to cancelled. The optimizer atomically changes running to committing only after the validated temporary file is ready; only the winner may proceed. Once committing begins, cancellation reports no transition instead of claiming that the job was cancelled.

This is preferred over adding another Boolean check because any check followed by replacement retains a race window. A mutex would also serialize the boundary, but the state machine makes late cancellation explicit and testable without holding a lock across `ReplaceFileW`.

### Prepare the replacement file completely before commit

Write and sync the sibling temporary file, read its bytes back, validate signature, decode, dimensions, and strict size reduction, then apply source permissions and optional timestamps to the temporary file. Only after all these operations succeed may the token enter committing and call `ReplaceFileW`. No fallible restoration step follows a successful replacement.

The temporary file remains uniquely named and dot-prefixed. The original design will be corrected to avoid promising the Windows hidden attribute: applying that attribute to the replacement file risks making the final source hidden, while clearing it creates another fallible pre-commit step with no user-visible safety benefit.

### Clear returns a full queue snapshot

Change `clear_jobs` and the frontend backend adapter to return the remaining jobs. The React handler awaits the operation and replaces its local list and summary from that snapshot. This is preferred over synthesizing deletion from summary events, which contain no job identities.

### Container-aware animation detection

Walk PNG and WebP chunks using their declared lengths and padding rules, and treat animation markers only as chunk types. Marker bytes in payloads, metadata, or trailing data are ignored.

### Document the existing bounded scheduler

Keep the current per-job asynchronous tasks gated by an `AppState`-owned semaphore. It already provides bounded concurrency and simpler per-job cancellation than an unused channel. Update the original design decision instead of adding a channel solely to match stale prose.

## Risks / Trade-offs

- [Risk] A cancellation arriving after the token enters committing cannot stop `ReplaceFileW` → Report cancellation only when the atomic transition succeeds; committing is the explicit point of no return.
- [Risk] Applying timestamps to the temporary file may have filesystem-specific behavior → Cover Windows replacement with timestamp tests and keep any error before commit.
- [Risk] Returning snapshots can race with later job updates → The snapshot contains all jobs remaining at command completion; subsequent full job-update events remain valid additions or updates.
- [Risk] Malformed chunk lengths could overflow → Use checked arithmetic and fail closed as non-animated; later image decoding still rejects invalid image data before replacement.

## Migration Plan

No persisted data migration is required. Land tests with the implementation, run the existing frontend/Rust/Sites/OpenSpec gates in development mode, and revert this change as a unit if regressions appear.
