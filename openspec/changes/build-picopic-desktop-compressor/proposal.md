## Why

Windows 用户需要一款像 ImageOptim 一样简单、但完全离线且能够安全原位替换图片的轻量压缩工具。当前项目为空，需要建立从拖放、格式筛选、压缩队列到可配置质量和安全写盘的完整桌面体验。

## What Changes

- 新建 Windows 优先的 Tauri 2 桌面应用，使用 React/TypeScript 构建 Fluent 风格界面，Rust 承担文件访问和图片压缩。
- 首页支持拖入图片或文件夹、递归发现静态 JPEG/PNG/WebP、自动排除不可处理文件并立即加入压缩队列。
- 为每个文件显示等待、处理中、成功、无需优化、跳过、失败和取消状态，以及压缩前后体积和节省比例。
- 压缩结果先写同目录临时文件，验证通过且严格更小时才原子替换原图；任何失败都保留原文件。
- 新增“常规、质量、优化速度”三页设置，支持各格式独立开启有损压缩和配置默认 75% 的质量值。
- 所有处理完全离线；首版不包含格式转换、缩放、备份、云服务、正式安装包或动画图片压缩。

## Capabilities

### New Capabilities
- `desktop-compression-workflow`: 文件/文件夹导入、格式筛选、队列状态、安全原位替换、取消、重试和汇总展示。
- `format-aware-image-optimization`: 静态 JPEG、PNG、WebP 的无损/有损压缩策略、质量映射、元数据处理和输出校验。
- `compression-preferences`: 常规、质量、优化速度设置的交互、校验、默认值和本地持久化。

### Modified Capabilities

无。

## Impact

- 新增 React/TypeScript/Vite 前端、Tauri 2 壳层和 Rust 压缩核心。
- 新增原生文件拖放与选择、Tauri 命令/事件接口、设置存储、压缩任务状态模型和视觉验收材料。
- 引入 JPEG、PNG、WebP 编解码与优化依赖，并要求 Windows Rust stable-msvc、Microsoft C++ Build Tools 和 WebView2 开发环境。
