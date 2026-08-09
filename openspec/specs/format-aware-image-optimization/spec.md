# format-aware-image-optimization Specification

## Purpose

定义静态 JPEG、PNG、WebP 的本地压缩、质量控制、元数据策略和格式校验行为，确保不同编码路径具有一致且可验证的安全边界。

## Requirements

### Requirement: Recognize supported static formats
The system SHALL recognize JPEG, PNG, and WebP using both extension and file signature, and SHALL reject invalid data or animated variants without modifying them.

#### Scenario: Static supported image
- **WHEN** a regular file has a supported extension and matching valid static image data
- **THEN** the system identifies its format and makes it eligible for optimization

#### Scenario: Animated or mismatched image
- **WHEN** a file is APNG, animated WebP, has an unsupported format, or its signature does not match its extension
- **THEN** the system skips the file with a specific reason and does not modify it

### Requirement: Optimize JPEG according to its format settings
The system SHALL encode JPEG pixels at the configured quality when lossy compression is enabled and SHALL avoid pixel re-encoding when lossy compression is disabled.

#### Scenario: JPEG lossy mode
- **WHEN** JPEG lossy compression is enabled at quality 75
- **THEN** the system encodes a valid JPEG using that quality target and proceeds to common output validation

#### Scenario: JPEG lossless mode
- **WHEN** JPEG lossy compression is disabled
- **THEN** the system performs only lossless container and enabled metadata cleanup and keeps the source unless the result is smaller

### Requirement: Optimize PNG according to its format settings
The system SHALL use palette quantization followed by lossless PNG optimization when PNG lossy compression is enabled, and SHALL use lossless PNG optimization only when it is disabled.

#### Scenario: Transparent PNG in lossy mode
- **WHEN** PNG lossy compression is enabled for a PNG with alpha transparency
- **THEN** the system preserves alpha transparency while applying the configured quality target and validates the optimized PNG

#### Scenario: PNG in lossless mode
- **WHEN** PNG lossy compression is disabled
- **THEN** the system preserves pixel values while optimizing the PNG container

### Requirement: Optimize WebP according to its format settings
The system SHALL encode static WebP at the configured quality when lossy compression is enabled and SHALL avoid converting an existing lossy WebP into a new lossy representation when lossy compression is disabled.

#### Scenario: WebP lossy mode
- **WHEN** WebP lossy compression is enabled
- **THEN** the system encodes a static WebP at the configured quality and validates it

#### Scenario: Existing lossy WebP in lossless mode
- **WHEN** lossy compression is disabled for an already lossy WebP
- **THEN** the system performs only enabled metadata/container cleanup and keeps the source unless the result is smaller

### Requirement: Handle orientation and metadata
The system SHALL normalize EXIF orientation before lossy pixel re-encoding and SHALL remove EXIF, GPS, XMP, ICC, and nonessential text metadata when metadata removal is enabled.

#### Scenario: Rotated JPEG with metadata removal enabled
- **WHEN** a JPEG contains a non-default EXIF orientation and privacy metadata
- **THEN** the output pixels have the expected displayed orientation and the configured metadata is absent

#### Scenario: Metadata preservation enabled
- **WHEN** metadata removal is disabled
- **THEN** the optimizer retains supported metadata that can be preserved safely by the selected codec path

### Requirement: Apply optimization effort independently from quality
The system SHALL provide fast, normal, extra, and extreme optimization levels that change encoder search effort without changing the configured quality value.

#### Scenario: Optimization level changes
- **WHEN** the user changes from normal to extra optimization while JPEG quality remains 75
- **THEN** subsequent JPEG jobs use greater encoding effort while retaining quality 75
