# PicoPic

PicoPic 是一款 Windows 优先、完全离线的轻量图片压缩工具。首版支持静态 JPEG、PNG 和 WebP，压缩结果只有在验证有效且严格更小时才原位替换源文件。

## 开发环境

- Node.js 20 或更高版本
- Rust stable MSVC：`rustup default stable-msvc`
- Microsoft Visual Studio Build Tools 的“使用 C++ 的桌面开发”工作负载
- Microsoft Edge WebView2 Runtime

## 开发命令

```powershell
npm install
npm run dev
npm run typecheck
npm test
npm run tauri:dev
```

本阶段只验证开发版，不生成 MSI、NSIS 或其他安装包。
