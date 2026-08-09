## Purpose

定义用户从添加图片或文件夹到安全完成原位压缩的完整桌面工作流，包括筛选、排队、状态反馈、汇总、取消、重试和故障保护。

## ADDED Requirements

### Requirement: Add files and folders
The system SHALL accept files and folders from native drag-and-drop and file/folder pickers, SHALL recursively scan folders when that preference is enabled, and SHALL deduplicate canonical file paths.

#### Scenario: Mixed folder is added
- **WHEN** a user adds a folder containing supported images, unsupported files, nested folders, and symbolic links
- **THEN** the system queues each unique supported regular image, follows the recursive preference, ignores unsupported files, and does not traverse symbolic links or reparse points

#### Scenario: Duplicate path is added
- **WHEN** a path already represented in the current task list is added again
- **THEN** the system does not create a duplicate task

### Requirement: Show observable job state
The system SHALL show each accepted image with its file name, status, original size, output size when available, saved bytes, saved percentage, and an actionable reason for skipped or failed jobs.

#### Scenario: Compression completes
- **WHEN** a queued image is processed successfully and replaced
- **THEN** its status becomes successful and the before/after sizes and saved percentage are displayed

#### Scenario: Compression is not beneficial
- **WHEN** a valid output is not strictly smaller than the source
- **THEN** the source remains unchanged and the task is marked as no optimization needed

### Requirement: Manage the queue
The system SHALL automatically process accepted images with bounded concurrency and SHALL allow users to cancel, retry, re-optimize, and clear completed list entries without deleting image files.

#### Scenario: Running job is cancelled
- **WHEN** cancellation is requested while an image is being encoded
- **THEN** queued work is cancelled immediately and any running encoder result is prevented from replacing the source

#### Scenario: Completed jobs are cleared
- **WHEN** the user clears completed jobs
- **THEN** terminal-state rows are removed from the interface and no image file is deleted

### Requirement: Replace source safely
The system MUST write output to a unique temporary file in the source directory, validate it, and replace the source only when the output is valid and strictly smaller. Any error MUST leave the source bytes unchanged.

#### Scenario: Valid smaller output
- **WHEN** an output decodes successfully, has the expected displayed dimensions, and is strictly smaller
- **THEN** the system atomically replaces the source, preserves permissions, optionally restores timestamps, and removes the temporary file

#### Scenario: Replacement cannot be guaranteed
- **WHEN** the source is read-only, locked, inaccessible, or the platform cannot perform a safe replacement
- **THEN** the system reports failure, removes temporary output where possible, and preserves the source

### Requirement: Show queue summary
The system SHALL display total task count, active count, total original bytes, total result bytes, total saved bytes, and aggregate saved percentage based on successful replacements.

#### Scenario: Multiple jobs finish
- **WHEN** two or more jobs reach terminal states
- **THEN** the queue summary updates without requiring a page refresh
