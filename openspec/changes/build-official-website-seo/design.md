## Context

现有仓库将 React 界面同时用于 Vite Web 构建和 Tauri 桌面应用，现有 `src/App.tsx` 是成熟的桌面压缩器，不能被官网替换。项目已经具备可移交的站点构建脚本和 GitHub 远程仓库，产品事实与限制以 README 和现有 OpenSpec 为准。

## Goals / Non-Goals

**Goals:**

- 在同一前端入口中可靠区分 Web 官网与 Tauri 桌面界面。
- 输出无后端依赖、适合 GitHub Pages 的高性能响应式页面。
- 让 SEO、社交分享和爬虫发现文件成为版本化源码。
- 保持现有桌面应用行为与测试不变。

**Non-Goals:**

- 不在官网内实现在线图片上传或压缩。
- 不改变桌面压缩算法、设置或任务队列。
- 不新增登录、统计、Cookie 或外部营销服务。

## Decisions

### Runtime-gated single entry

入口通过 Tauri 注入的全局对象判断运行环境：Tauri 渲染现有 `App`，普通浏览器渲染新的 `Website`。相比拆成两个构建项目，该方案复用现有工具链且避免同步两套品牌资源；相比按 URL 判断，不会因 GitHub Pages 子路径影响桌面行为。

### Static, semantic landing page

官网使用静态 React 标记和 CSS，不增加路由器或内容系统。页面以原生 header、main、section、footer、details 等元素构建，核心信息在首次渲染中可见。相比动态数据和重型组件库，它更适合单页产品官网的性能、可访问性和抓取需求。

### Repository-backed facts and download

所有产品说明均来自 README 与现有规格；下载链接固定到 `github.com/szb-66/PicoPic/releases/latest`，不在客户端猜测版本号。这样最新版发布后无需修改官网，也避免出现虚假的统计或评价。

### Generated raster icon with deterministic derivatives

使用 ImageGen 生成无文字的高分辨率正方形主图，再通过色键移除得到透明 PNG；所有 Windows PNG、ICO 与 ICNS 变体由 Tauri 官方图标命令从该主图统一派生。相比手工分别绘制尺寸，此方案确保每个产品表面使用相同轮廓；官网直接复用透明主图，桌面标题栏使用同一资源的应用内版本。

### GitHub Pages-compatible output

Vite 使用相对资源基址，使产物既能部署在 `/PicoPic/` 子路径也能由现有站点工作流在根路径提供。GitHub Actions 使用官方 Pages Actions 构建和发布 `dist/client`，同时保留当前 Sites 所需的服务端包装产物。

### SEO source split

对首屏必要的 meta、canonical、JSON-LD 放入 `index.html`，robots、sitemap 和 Web App manifest 放入 `public/`。社交预览图作为静态品牌资源。正式 URL 采用已知仓库对应的 `https://szb-66.github.io/PicoPic/`。

## Risks / Trade-offs

- [GitHub Pages 可能尚未启用 Actions 发布源] → 工作流推送后使用 GitHub API 检查并在有权限时配置 Pages。
- [运行环境检测依赖 Tauri 全局标记] → 使用 Tauri 2 的稳定注入标记并添加入口测试覆盖两种分支。
- [相对资源路径在深层路由不可用] → 官网只提供根路径单页，不引入客户端深层路由。
- [未签名安装包可能降低下载信任] → 在主要下载区域明确提示来源与 SHA-256 校验方式。
- [生成式图标可能包含小尺寸噪声] → 检查透明边缘，并验证 32px、128px 与 ICO 派生资源；界面中保留足够尺寸和对比度。

## Migration Plan

1. 生成透明主图并派生 Tauri/Windows 图标资源。
2. 新增官网组件、专用样式和静态 SEO 资源。
3. 更新入口以按运行环境选择界面，并补充自动化测试。
4. 完成开发版、单元测试、站点构建和静态产物检查。
5. 提交并推送到 GitHub，启用或更新 GitHub Pages Actions 发布。
6. 若发布失败，保留源码与工作流，桌面应用仍可独立运行；回滚 Pages 工作流即可撤下官网。
