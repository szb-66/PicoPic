## 1. Regression coverage

- [x] 1.1 Add a window-actions seam and component tests for minimize, maximize/restore, close, and removal of the dead more-options control
- [x] 1.2 Add layout assertions for the custom title bar drag region and vertically centered command groups

## 2. Desktop window shell

- [x] 2.1 Disable native decorations and grant only the Tauri window permissions required by the custom title bar
- [x] 2.2 Implement the branded drag region and accessible minimize, maximize/restore, and close controls
- [x] 2.3 Remove the dead more-options control and preserve the existing settings entry

## 3. Command bar alignment

- [x] 3.1 Replace the two-row command grid with a single centered layout at regular and compact widths

## 4. Verification

- [x] 4.1 Run type checking, frontend tests, build, Sites tests, and strict OpenSpec validation
- [x] 4.2 Run the development app, verify title-bar interactions and responsive visual alignment, and remove any temporary or failed-attempt code

## 5. Minimal empty state

- [x] 5.1 Remove duplicate file and folder buttons from the empty workspace while preserving the persistent command-bar entries
- [x] 5.2 Add regression coverage for the minimal empty state and update the durable prototype design decision
