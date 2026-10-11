<div align="center">

# pp-switch

**Pure Provider. 只切地址和 Key。**

为 Claude Code、Codex 和 Gemini CLI 准备的极简 Provider 切换器。

[![CI](https://github.com/phonowell/pp-switch/actions/workflows/ci.yml/badge.svg)](https://github.com/phonowell/pp-switch/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-black.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux-black)

[安装](#安装) · [使用](#使用) · [兼容范围](#兼容范围) · [开发](#开发)

</div>

## 为什么做 pp-switch

你已经配好了模型、权限和工具。换一家 API 服务时，只想换地址和 Key。

pp-switch 为这个操作而生：一个窗口，几组 Provider，点一下切换。
你的模型、推理参数、MCP、Skills 和其他客户端配置继续由你管理。
关闭窗口，应用退出。

## 界面预览

![pp-switch 的真实窗口：客户端选择、Provider 列表、地址与密钥编辑和链接导入](assets/screenshot.png)

截图使用示例地址和测试密钥。图片不随应用分发。

## 功能边界

| | pp-switch | cc-switch |
| --- | --- | --- |
| 定位 | 地址和 API Key 切换器 | 多客户端综合管理工具 |
| Provider 链接 | 导入受支持的 `ccswitch://` Provider 链接 | 完整深链及综合管理能力 |
| 修改客户端配置 | 仅当前生效的地址和 Key | 按功能管理更多 Provider／客户端字段 |
| 模型、代理、用量、MCP、Skills | 不提供 | 提供对应管理能力 |
| 运行方式 | 单窗口，关闭即退出 | 更多窗口、托盘及代理等工作流 |

需要综合管理时，[cc-switch](https://github.com/farion1231/cc-switch) 是更完整的选择。
pp-switch 专注于已经配置好客户端、只想快速更换 API 服务的人。

## 安装

当前版本为 **0.0.1 开发预览**。源码和基本功能已实现，内存预算尚未达标。
构建包可在 [Actions](https://github.com/phonowell/pp-switch/actions/workflows/ci.yml) 对应运行的 Artifacts 中下载；
正式发布会出现在 [GitHub Releases](https://github.com/phonowell/pp-switch/releases)。

| 系统 | 构建包 | 使用方式 |
| --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin.zip` | 解压，将 `.app` 移入 Applications |
| macOS Intel | `x86_64-apple-darwin.zip` | 同上 |
| Windows x64 | `x86_64-pc-windows-msvc.zip` | 解压，运行 `pp-switch.exe` |
| Linux x64 | `x86_64-unknown-linux-gnu.tar.gz` | 解压，在桌面环境运行 `pp-switch` |

macOS 构建未使用 Apple Developer 签名或公证。Linux 需要 X11／XWayland 和系统 X11、fontconfig 库。
具体平台是否已经通过构建与 GUI 验收，请以 [验证记录](docs/verification.md) 和对应 CI 运行结果为准。

## 使用

1. 选择 Claude Code、Codex 或 Gemini CLI。
2. 新增 Provider，填写名称、Base URL 和 API Key；也可以粘贴 cc-switch Provider 链接。
3. 点击切换。客户端会使用原来的模型、协议和其他配置。

客户端需要已配置为支持的 API Key 模式。OAuth 切换、协议转换和客户端首次初始化
不在范围内；不支持的配置会提示原因。正在运行的客户端可能需要重启以读取新配置。
只操作用户级配置；项目配置、启动参数和另一个终端的环境变量仍可能覆盖它。

| 客户端 | 操作位置 | 前提 |
| --- | --- | --- |
| Claude Code | `~/.claude/settings.json` 的两个 env 字段 | 无外部 Key helper、云平台模式或冲突凭据 |
| Codex | 当前 Provider 的 `base_url`，以及既有 Key 位置 | 已选中自定义 Provider，并配置为 API Key 模式 |
| Gemini CLI | `~/.gemini/.env` 的地址和 Key | 已通过 `/auth` 选择 API Key 模式 |

支持 `CLAUDE_CONFIG_DIR`、`CODEX_HOME` 目录覆盖；不会更改这些环境变量。
本地 Provider 列表位于系统的 pp-switch 用户数据目录：macOS 为
`~/Library/Application Support/pp-switch`，Windows 为 `%LOCALAPPDATA%\pp-switch`，
Linux 为 `${XDG_DATA_HOME:-~/.local/share}/pp-switch`。

## 兼容范围

```text
ccswitch://v1/import?resource=provider&app=claude&name=Example&endpoint=https%3A%2F%2Fapi.example.com&apiKey=sk-example
```

- 支持 `claude`、`codex`、`gemini` 的 Provider 链接。
- 支持直接参数，以及 Base64 内嵌 JSON／TOML 配置中的地址、Key 提取。
- 多地址取第一个；模型、脚本和其他附加配置不应用。
- 不导入 MCP、Prompt、Skill，不共享 cc-switch 数据库，不注册系统链接关联。
- 不自动更改认证方式，不清理已有配置，也不修改 shell 环境。
- Key 保存在本地用户专属文件中；界面遮罩不是加密。

完整行为约定见 [SPEC.md](SPEC.md)。

## 性能预算

| 指标 | 目标 |
| --- | --- |
| 安装后应用自身总大小 | ≤ 5 MiB |
| 打开窗口时空闲总 RSS | ≤ 20 MiB |
| 启动／导入峰值总 RSS | ≤ 30 MiB |
| 冷启动到可操作 | ≤ 300 ms |
| 关闭窗口后 | 进程退出，驻留为 0 |

**以上是预算，不是宣传承诺。** 数值包含随应用分发的库和资源，内存包含子进程。
首轮 macOS ARM64 实测：安装后约 **1.12 MiB**；空闲总 RSS 约 **121 MiB**，内存预算未达标。
发布构建的硬件、测量方式、失败记录和实际结果见 [验证记录](docs/verification.md)。

## 开发

项目采用 spec 驱动开发：先明确 [SPEC.md](SPEC.md) 中的行为，再实现和验证。

技术栈：Rust + FLTK。需要 Rust ≥ 1.89、CMake 和平台 C++ 编译工具链。
Linux 的构建依赖见 [CI 工作流](.github/workflows/ci.yml)。

```sh
cargo run --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release --bins --examples
cargo run --locked --example ui_smoke
python3 scripts/measure.py target/release/examples/measure_ui --output target/metrics.json --enforce
```

Linux GUI 测试可使用 `xvfb-run -a`；Windows 示例二进制后缀为 `.exe`。
性能脚本运行独立临时配置，使用同一个窗口与事件循环，不修改你的客户端配置。
CI 保留未达标的测量结果，并把预算失败标记为失败。

## 致谢与许可

感谢 [cc-switch](https://github.com/farion1231/cc-switch) 的开源协议与实现参考。
pp-switch 是独立项目，与 cc-switch 无隶属关系。

界面基于 [fltk-rs](https://github.com/fltk-rs/fltk-rs) 与 [FLTK](https://www.fltk.org) 项目的工作。

[MIT](LICENSE)。第三方依赖保留各自许可证。
