## Purpose

为 PicoPic 建立清晰、可识别且跨尺寸一致的产品图标，在 Windows 应用、桌面界面和官方网站之间形成统一的品牌识别。

## ADDED Requirements

### Requirement: Icon communicates image compression
产品图标 SHALL 通过图片图形与向内压缩符号表达图片压缩，同时避免依赖文字、扩展名或 ZIP 文件隐喻。

#### Scenario: User sees the icon without a label
- **WHEN** 图标单独显示在应用列表、任务栏或官网中
- **THEN** 图标保持可识别的图片与压缩视觉语义

### Requirement: Icon remains legible across required sizes
产品图标 SHALL 提供 Windows 打包所需的 PNG 和 ICO 资源，并在 16 至 256 像素常见显示尺寸下保持清晰轮廓。

#### Scenario: Windows displays a small icon
- **WHEN** 系统以小尺寸显示 PicoPic 图标
- **THEN** 主要蓝色轮廓和白色图片压缩符号仍可辨认

### Requirement: Product surfaces use one brand mark
桌面应用标题区域、官网导航与 favicon SHALL 使用同一图标源或由该图标源生成的尺寸变体。

#### Scenario: User moves between website and application
- **WHEN** 用户先访问官网再启动桌面应用
- **THEN** 两个界面展示一致的 PicoPic 品牌图标
