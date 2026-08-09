# desktop-window-shell Specification

## Purpose

统一 PicoPic 桌面主窗口的品牌外观与原生窗口操作，并确保主要命令区在不同窗口尺寸下保持清晰、稳定和可操作。

## Requirements

### Requirement: Unified custom title bar
桌面应用 SHALL 使用与主界面一致的自绘标题栏替代系统标题栏，并 SHALL 提供拖动窗口、最小化、最大化或还原以及关闭窗口的能力。

#### Scenario: Move the window
- **WHEN** 用户按住标题栏的非交互区域并拖动
- **THEN** 主窗口跟随指针移动

#### Scenario: Use window controls
- **WHEN** 用户点击最小化、最大化或还原、关闭按钮
- **THEN** 桌面窗口执行对应的原生窗口操作

#### Scenario: Toggle maximize by title-bar gesture
- **WHEN** 用户双击标题栏的非交互区域
- **THEN** 桌面窗口在最大化与还原状态之间切换

### Requirement: Every visible control has a clear purpose
主窗口 SHALL 不显示没有可观察行为或与现有入口无差别重复的操作控件。

#### Scenario: Inspect the main header
- **WHEN** 用户查看主窗口标题栏
- **THEN** 标题栏只显示品牌信息和可用的窗口控制，不显示无行为的“更多选项”按钮

#### Scenario: Inspect the empty workspace
- **WHEN** 当前没有图片任务且底部命令栏已提供添加图片和文件夹入口
- **THEN** 空状态只显示拖放提示，不重复显示选择图片或选择文件夹按钮

### Requirement: Vertically centered command groups
底部命令栏 SHALL 让左右两侧命令组在可用高度内垂直居中，并 SHALL 在响应式紧凑布局下维持居中。

#### Scenario: View regular-width command bar
- **WHEN** 主窗口宽度足以显示命令文字
- **THEN** 左右命令组与底栏的垂直中线对齐

#### Scenario: View compact command bar
- **WHEN** 主窗口进入隐藏命令文字的紧凑布局
- **THEN** 左右图标命令仍与底栏的垂直中线对齐
