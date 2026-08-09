## Purpose

为 PicoPic 提供可信、易理解且可在桌面与移动设备访问的官方产品入口，帮助用户理解产品价值并安全前往官方发布页下载。

## ADDED Requirements

### Requirement: Web visitors receive a product website
系统 SHALL 在普通 Web 浏览器中展示 PicoPic 官方网站，并在 Tauri 运行环境中继续展示桌面压缩器。

#### Scenario: Visit from a browser
- **WHEN** 用户通过普通浏览器访问站点
- **THEN** 系统展示响应式产品官网及主要下载入口

#### Scenario: Launch desktop development build
- **WHEN** 用户通过 Tauri 开发版启动应用
- **THEN** 系统展示现有图片压缩器而非官网

### Requirement: Website communicates verified product capabilities
官网 SHALL 准确说明 PicoPic 的本地离线处理、支持格式、批量处理、安全替换、元数据清理和当前平台限制。

#### Scenario: User evaluates the product
- **WHEN** 用户浏览官网主要内容
- **THEN** 用户可以了解支持 JPEG、PNG、WebP 静态图片、Windows 64 位以及原位替换等重要信息

### Requirement: Website provides trustworthy download actions
官网 SHALL 将主要下载操作指向项目的 GitHub Releases 最新版本，并清楚标识当前 Windows 平台支持与未签名安装包提示。

#### Scenario: User chooses to download
- **WHEN** 用户激活主要下载按钮
- **THEN** 浏览器打开官方 GitHub Releases 最新版本页面

### Requirement: Website is usable across devices and input modes
官网 SHALL 在主流桌面和移动视口下保持清晰层级、可操作导航、可见焦点以及符合语义的内容结构。

#### Scenario: Mobile visitor reads the site
- **WHEN** 视口宽度缩小到移动设备尺寸
- **THEN** 页面内容重排且不存在必须横向滚动才能访问的主要内容

#### Scenario: Keyboard visitor navigates actions
- **WHEN** 用户仅使用键盘浏览交互元素
- **THEN** 所有主要操作均可聚焦、识别并激活

