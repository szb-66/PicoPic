## Why

PicoPic 已具备首个可用版本的核心能力，但 GitHub 访问者仍缺少完整的产品说明、安装方式和安全提示，同时仓库也尚未提供可下载的正式发行包。现在需要统一首版版本号、文档和构建产物，使用户能够理解、下载并安全使用 PicoPic。

## What Changes

- 将 README 重写为面向 GitHub 用户的项目首页，涵盖功能、安装、使用、安全机制、开发和许可信息。
- 将前端包、Rust crate 与 Tauri 应用版本统一为 `0.1.0`。
- 启用 Windows NSIS 安装包构建，并验证开发版、测试与生产构建。
- 提交并推送当前首版代码，创建 `v0.1.0` Git 标签和 GitHub Release，附带 Windows 安装包及校验文件。

## Capabilities

本次变更仅涉及文档、版本元数据、打包配置与发布流程，不改变应用行为，因此不新增或修改产品能力规范。

## Impact

影响 `README.md`、Node/Rust/Tauri 版本元数据、Tauri bundle 配置、OpenSpec 发布记录、Git 历史以及 GitHub Releases。发布步骤会在远端创建不可变标签和面向用户的下载入口。
