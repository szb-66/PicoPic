## Context

PicoPic 当前只有一次初始提交，工作区包含首版完成后的界面、窗口控制、审查修复和 OpenSpec 归档改动。Rust crate 与 Tauri 配置已是 `0.1.0`，但 `package.json` 仍为 `0.0.0`，Tauri bundle 尚未启用，README 也主要面向开发者。发布目标为 GitHub 上可下载的 Windows 首版；动机见 `proposal.md`。

## Goals / Non-Goals

**Goals:**

- 让项目首页准确解释 PicoPic 的用户价值、系统要求、安装使用方法和原位替换安全边界。
- 让所有公开版本元数据一致，并产出可安装、可校验的 Windows NSIS 包。
- 在发布前完成前端、Rust、Sites 兼容性和安装包构建验证。
- 使提交、标签、GitHub Release 和附件对应同一份已验证源码。

**Non-Goals:**

- 不新增压缩格式或改变编码算法。
- 不在首版引入自动更新、代码签名或 CI/CD 发布流水线。
- 不发布 npm 或 crates.io 包；`package.json` 保持 `private`。

## Decisions

### 以 `v0.1.0` 作为首个公开版本

采用语义化版本 `0.1.0` 表示产品已可供早期用户使用，但接口和行为仍可能在 `1.0.0` 前调整。相比 `v1.0.0`，该版本更准确表达首版成熟度；相比保留 `0.0.0`，它能建立清晰的发布基线。

### 发布 NSIS 安装包而非包管理器包

PicoPic 是 Tauri Windows 桌面应用，用户交付物应是 GitHub Release 中的 NSIS 安装程序。npm 和 Cargo 清单只作为构建元数据，不对外发布。首版不同时生成 MSI，以减少工具链差异和重复附件。

### README 以用户路径为主、贡献路径为辅

README 顶部先说明产品、能力、下载和使用，再描述安全边界、限制与开发命令。压缩会原位替换文件，因此必须醒目标注先备份重要图片，并解释“仅在有效且更小时替换”的保护机制。

### 发布附件提供 SHA-256

为安装程序生成同名 `.sha256` 文件并同时上传，便于用户验证下载完整性。首版没有代码签名证书，README 和 Release Notes 应如实提示 Windows 可能显示未知发布者警告。

## Risks / Trade-offs

- [安装程序未签名，Windows 可能显示 SmartScreen 警告] → 在 README 与 Release Notes 中明确披露，不提供绕过系统安全机制的误导性承诺。
- [原位压缩具有数据风险] → README 强调先备份，并准确说明临时文件、输出验证、严格变小和失败保留源文件的边界。
- [工作区包含多项既有未提交改动] → 发布前审查完整 diff、运行全套测试与构建，并把所有变更作为同一个首版快照提交。
- [本机打包工具链可能缺失] → 优先构建 NSIS；若工具自动下载失败，保留源码提交但不得创建缺少附件的正式 Release。

## Migration Plan

1. 同步版本元数据并启用 NSIS bundle。
2. 完成 README，检查链接和项目声明。
3. 运行类型检查、前端测试、Rust 测试、Sites 测试与前端构建。
4. 构建 NSIS 安装程序并计算 SHA-256。
5. 审查 diff 后提交并推送 `main`，创建并推送 `v0.1.0` 标签。
6. 从该标签创建 GitHub Release 并上传安装程序及校验文件。

若发布前验证失败，则不创建标签或 Release；若上传后发现附件错误，则删除草稿 Release 并从同一提交重新构建，正式发布后的标签不移动。
