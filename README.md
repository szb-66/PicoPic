<div align="center">
  <img src="src-tauri/icons/128x128.png" width="96" height="96" alt="PicoPic 图标">
  <h1>PicoPic</h1>
  <p>轻巧、离线、安全的 Windows 图片压缩工具</p>
  <p>
    <a href="https://szb-66.github.io/PicoPic/">官方网站</a>
    ·
    <a href="https://github.com/szb-66/PicoPic/releases/latest">下载最新版</a>
    ·
    <a href="https://github.com/szb-66/PicoPic/issues">反馈问题</a>
  </p>
</div>

PicoPic 在本地压缩 JPEG、PNG 和 WebP 图片，不上传文件，也不依赖云端服务。你可以拖入图片或文件夹，应用会自动排队处理，并且只在压缩结果有效且严格更小时替换原文件。

> [!IMPORTANT]
> PicoPic 会原位替换压缩成功的图片。虽然应用包含临时文件、输出验证和失败回退保护，首次使用前仍建议备份重要图片。

## 功能亮点

- **完全离线**：所有扫描、编码和验证都在本机完成。
- **批量处理**：支持添加多张图片、拖放文件夹和递归扫描子文件夹。
- **格式独立设置**：分别控制 JPEG、PNG、WebP 的有损开关与质量。
- **安全替换**：先写入同目录临时文件，验证格式与尺寸，且结果严格更小时才替换源文件。
- **隐私清理**：可移除 EXIF、GPS、XMP、ICC 和非必要文本元数据，并在重新编码前规范化 EXIF 方向。
- **可控优化强度**：快速、常规、额外、极致四档只改变编码搜索强度，不暗中改变质量值。
- **清晰的任务状态**：展示原始大小、压缩后大小、节省比例，以及跳过或失败的具体原因。

## 支持的格式

| 格式 | 有损模式 | 无损模式 | 说明 |
| --- | --- | --- | --- |
| JPEG | 按质量重新编码 | 容器与元数据优化 | 支持 EXIF 方向规范化 |
| PNG | 调色板量化后无损优化 | 无损容器优化 | 保留 Alpha 透明度 |
| WebP | 按质量重新编码 | 安全的容器与元数据优化 | 仅支持静态 WebP |

APNG、动画 WebP、扩展名与文件内容不匹配的图片会被明确跳过，不会修改源文件。

## 下载与安装

1. 前往 [Releases](https://github.com/szb-66/PicoPic/releases/latest) 下载最新的 Windows 安装程序。
2. 可选：使用同一 Release 中的 `.sha256` 文件校验下载内容：

   ```powershell
   Get-FileHash .\PicoPic_0.1.0_x64-setup.exe -Algorithm SHA256
   ```

3. 运行安装程序并启动 PicoPic。

首版安装程序尚未进行代码签名，因此 Windows 可能显示“未知发布者”或 SmartScreen 提示。请只从本仓库的 Releases 页面下载，并核对 SHA-256 校验值。

## 使用方法

1. 将图片或文件夹拖到窗口中，或使用底部的“添加图片”“文件夹”按钮。
2. PicoPic 会自动筛选支持的静态图片、去除重复路径并开始压缩。
3. 在列表中查看每张图片的状态、压缩前后大小和节省比例。
4. 通过设置面板调整递归扫描、元数据、时间戳、各格式质量和优化速度；新设置会应用于之后加入或重试的任务。

如果压缩结果没有变小，PicoPic 会保留原文件并显示“无需优化”。取消任务、清理已完成列表或发生编码错误时，也不会删除源图片。

## 当前限制

- 当前发行包面向 64 位 Windows。
- 仅处理静态 JPEG、PNG 和 WebP，不支持动画图片或其他格式。
- 压缩结果直接替换源文件，目前不提供“输出到其他目录”模式。
- `0.x` 阶段的设置结构和交互仍可能调整。

## 本地开发

### 环境要求

- Node.js 20 或更高版本
- Rust stable MSVC：`rustup default stable-msvc`
- Microsoft Visual Studio Build Tools，并安装“使用 C++ 的桌面开发”工作负载
- Microsoft Edge WebView2 Runtime

### 常用命令

```powershell
npm install
npm run tauri:dev
```

开发版验证：

```powershell
npm run typecheck
npm test
npm run test:sites
cargo test --manifest-path src-tauri/Cargo.toml
```

构建 Windows 安装包：

```powershell
npm run tauri build
```

主要代码位于 `src/`（React 界面）和 `src-tauri/src/`（Rust 扫描、队列、设置与压缩核心）。产品行为约束记录在 `openspec/specs/`。

## 参与贡献

欢迎通过 [Issues](https://github.com/szb-66/PicoPic/issues) 报告可复现的问题或提出改进建议。提交代码前，请运行上述开发版验证命令，并说明涉及的文件格式、设置和预期结果。

## 许可证

本仓库目前未附带开源许可证。在许可证明确之前，源码仅供查看，默认版权限制仍然适用。
