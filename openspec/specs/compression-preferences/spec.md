# compression-preferences Specification

## Purpose

定义图片压缩偏好设置的默认值、可配置范围、本地持久化和对新任务的生效规则，使常规、质量与优化速度设置保持一致且可恢复。

## Requirements

### Requirement: Provide three preference sections
The system SHALL present General, Quality, and Optimization Speed sections in a keyboard-accessible preferences dialog.

#### Scenario: Preferences are opened
- **WHEN** the user activates the settings control
- **THEN** the dialog opens on the General section and all three sections can be selected without closing the dialog

### Requirement: Provide safe defaults
The system SHALL default to recursive folder scanning enabled, metadata removal enabled, timestamp preservation disabled, lossy compression enabled at quality 75 for JPEG/PNG/WebP, and normal optimization speed.

#### Scenario: First launch
- **WHEN** no valid persisted settings exist
- **THEN** the complete default settings are loaded and shown in the preferences dialog

### Requirement: Configure each format independently
The system SHALL allow JPEG, PNG, and WebP to independently enable lossy compression and select an integer quality from 1 through 100, where higher values prioritize fidelity.

#### Scenario: PNG quality is changed only
- **WHEN** the user changes PNG quality from 75 to 60
- **THEN** JPEG and WebP quality remain unchanged and future PNG tasks use quality 60

#### Scenario: Invalid quality is submitted
- **WHEN** a settings payload contains a quality outside 1 through 100
- **THEN** the system rejects the update and retains the last valid settings

### Requirement: Persist versioned preferences locally
The system SHALL save validated settings locally with a schema version and SHALL restore them on the next launch without network access.

#### Scenario: Valid settings are saved
- **WHEN** the user changes preferences and confirms the dialog
- **THEN** the settings are persisted and apply to newly queued or retried jobs

#### Scenario: Persisted settings are corrupt or unsupported
- **WHEN** the stored settings cannot be parsed or have an unsupported schema version
- **THEN** the system loads safe defaults and remains usable

### Requirement: Preserve settings transactionally
The system SHALL treat edits as a draft until the user confirms them and SHALL discard draft changes when the dialog is cancelled.

#### Scenario: Preferences are cancelled
- **WHEN** the user modifies settings and closes the dialog with Cancel
- **THEN** runtime and persisted settings remain unchanged
