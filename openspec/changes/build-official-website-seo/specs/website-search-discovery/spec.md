## Purpose

让 PicoPic 官网能够被搜索引擎正确抓取、理解和分享，并通过可重复的 GitHub Pages 流程持续发布经过验证的静态站点。

## ADDED Requirements

### Requirement: Pages expose complete search metadata
官网 SHALL 提供唯一且描述准确的标题、描述、canonical URL、语言声明、Open Graph、X Card 和 SoftwareApplication 结构化数据。

#### Scenario: Search crawler reads the homepage
- **WHEN** 搜索引擎抓取官网首页
- **THEN** 响应包含与 PicoPic、离线图片压缩和 Windows 平台相关的可解析元数据与结构化数据

### Requirement: Crawlers can discover and index the site
官网 SHALL 提供 robots.txt 和 sitemap.xml，并在其中声明 GitHub Pages 的正式站点 URL。

#### Scenario: Crawler requests discovery files
- **WHEN** 爬虫访问 robots.txt 或 sitemap.xml
- **THEN** 系统返回可解析文件且其中的站点地址指向正式公开网址

### Requirement: Social shares have a branded preview
官网 SHALL 为支持 Open Graph 的分享平台提供与官网品牌和核心信息一致的预览图像与文字。

#### Scenario: Link is shared
- **WHEN** 分享平台读取官网 Open Graph 元数据
- **THEN** 平台可获取 PicoPic 品牌预览图、标题和产品描述

### Requirement: GitHub Pages deployment is repeatable
仓库 SHALL 包含在主分支更新后构建并发布官网到 GitHub Pages 的自动化流程，且发布内容来自通过验证的生产构建。

#### Scenario: Main branch receives website changes
- **WHEN** 官网改动进入主分支或发布流程被手动触发
- **THEN** GitHub Actions 构建站点并将静态产物部署到 GitHub Pages

