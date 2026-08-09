## Why

PicoPic 目前只有 GitHub README 和桌面应用界面，缺少一个面向普通用户、可被搜索引擎理解并直接引导下载的官方网站。现在需要建立清晰可信的产品入口，同时保持桌面端现有工作流不受影响。

## What Changes

- 新增响应式 PicoPic 官方网站，清楚呈现离线隐私、批量压缩、安全替换、格式支持和下载入口。
- 生成并落地统一的 PicoPic 产品图标，覆盖 Windows 应用包、桌面界面和官网品牌资源。
- Web 环境展示官网，Tauri 开发版继续展示现有桌面压缩器。
- 增加语义化页面结构、页面元数据、Open Graph、JSON-LD、canonical、robots.txt 和 sitemap.xml。
- 增加可访问、可索引的常见问题内容，并优化移动端体验与性能。
- 将构建产物部署到 GitHub Pages，并提供可复用的自动部署工作流。

## Capabilities

### New Capabilities

- `official-product-website`: 面向潜在用户的 PicoPic 响应式产品官网、下载路径、产品信息与常见问题。
- `product-brand-icon`: PicoPic 软件图标的视觉语义、尺寸适配与产品内一致性。
- `website-search-discovery`: 官网的搜索引擎抓取、结构化数据、社交分享元数据和 GitHub Pages 发布行为。

### Modified Capabilities

无。

## Impact

- 影响 React Web 入口、全局样式、静态资源、Tauri 图标资源、HTML 元数据、Vite 构建配置和 GitHub Actions。
- 桌面端通过运行环境检测继续使用现有 `App`，无需修改压缩后端或 Tauri 核心逻辑。
- 发布目标为现有 `szb-66/PicoPic` 仓库的 GitHub Pages。
