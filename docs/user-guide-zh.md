# pp-switch 使用指南

[English](user-guide.md) · 简体中文 · [日本語](user-guide-ja.md)

pp-switch 保存服务商，并且**只切换当前生效的 Base URL 与 API Key**。
不初始化客户端、不选模型、不改变认证方式、不转换协议。服务商需要兼容客户端现有的模型和协议。[下载](https://github.com/phonowell/pp-switch/releases)。

## 第一次切换

1. 先在客户端本身配置受支持的 API Key 模式。
2. 打开 pp-switch，在左侧选择客户端。
3. 「新增」，填写名称、HTTP(S) 地址和非空 Key，点击「保存」。
4. 「切换服务商」。客户端没有重读配置时，重启客户端。

保存只改变本地服务商列表。`*` 表示客户端的**地址和 Key**均与该条目匹配；
状态从客户端文件读取，不依赖记住的活动 ID。切换前必须保存编辑内容。
删除只删除本地条目，不恢复或更改客户端。

## 客户端前提与精确写入范围

| 客户端 | 用户级位置 | 管理的值 |
| --- | --- | --- |
| Claude Code | `~/.claude/settings.json` | `env.ANTHROPIC_BASE_URL` 与既有 `env.ANTHROPIC_AUTH_TOKEN` 或 `env.ANTHROPIC_API_KEY` |
| Codex | `~/.codex/config.toml`，必要时加 `~/.codex/auth.json` | 当前自定义服务商的 `base_url`；既有 `experimental_bearer_token` 或 `OPENAI_API_KEY` |
| Gemini CLI | `~/.gemini/.env` | `GOOGLE_GEMINI_BASE_URL` 与 `GEMINI_API_KEY` |

`~` 是各系统的用户主目录。Claude 支持 `CLAUDE_CONFIG_DIR`，Codex 支持 `CODEX_HOME`。
Gemini 的 `settings.json` 只读，不写入。

**Claude：** 两种 Key 字段都不存在时使用 `ANTHROPIC_AUTH_TOKEN`。
同时存在两个字段、使用 `apiKeyHelper` 或启用 Bedrock / Vertex / Foundry 时拒绝切换。
不管理 OAuth 登录，不转换云平台认证。

**Codex：** 已有 `model_provider` 必须指向自定义 `model_providers` 条目。
不支持 `env_key`。使用 bearer token 时，不能同时设置 `requires_openai_auth=true`。
文件 API Key 认证则要求该标志为 true、已有非空 `auth.json.OPENAI_API_KEY`、
无活动 OAuth 凭据，且 `cli_auth_credentials_store` 未设置或为 `file`。
不支持外部 / keyring 存储。不改变服务商选择、`wire_api`、模型或认证方式。
先通过 Codex 配好兼容设置；导入链接不会替你初始化。

**Gemini：** 先通过 `/auth` 选择 API Key；
`security.auth.selectedType` 已经是 `gemini-api-key`。
拒绝 OAuth / Vertex 模式，`settings.json` 保持不变。

仅编辑用户级配置。项目配置、启动参数和客户端环境变量可能优先。
pp-switch 能看到的覆盖会被提示；另一个终端的环境可能不同。
请自行处理启动环境覆盖；应用不修改 shell 配置或系统环境变量。

## 导入已有链接

把服务商链接粘贴进遮罩输入框，点击「导入」：

```text
ccswitch://v1/import?resource=provider&app=claude&name=Example&endpoint=https%3A%2F%2Fapi.example.com&apiKey=sk-example
```

也支持 `ppswitch://`。app 值支持 `claude`、`codex`、`gemini`。
支持直接参数和 Base64 内嵌 JSON/TOML（标准 / URL 安全，有 / 无填充）；
非空显式参数优先，多地址取第一个。仅名称、客户端、地址、Key 进入编辑器；
模型、脚本、工具等附加字段丢弃。核对后「保存」，再「切换服务商」。

导入只是预览，不改客户端文件，也尚未保存条目。
拒绝仅远程配置、其他资源、不支持的版本 / 客户端和不完整输入。
不共享 cc-switch 数据库，不注册系统链接关联。不监控剪贴板，只接受你正常粘贴。
真实链接含凭据，应按密钥保护；上面的链接是假数据。

## 语言与本地数据

Linux 请安装 Noto Sans CJK（Debian/Ubuntu：`sudo apt install fonts-noto-cjk`）或 WenQuanYi Zen Hei。Xft 不会自动为缺失字形回退，必须有已安装的中日韩字体。应用不分发字体文件。

界面支持英文、简体中文、繁体中文、日语。首次跟随系统语言，不支持的语言回退英文。
右上角可即时更换标签、对话框和状态语言，保留草稿与当前选择。

| 系统 | pp-switch 数据目录 |
| --- | --- |
| macOS | `~/Library/Application Support/pp-switch` |
| Windows | `%LOCALAPPDATA%\pp-switch` |
| Linux | `${XDG_DATA_HOME:-~/.local/share}/pp-switch` |

`providers.json` 保存连接和 Key；`ui.json` 只保存语言。
权限仅限当前用户（Unix 0600 / Windows 用户 ACL）。**遮罩不是加密。**
锁与恢复文件用于协调写入，错误数据会提示，不会悄悄覆盖。
需要备份时私密保管整个目录，不要公开或粘贴到诊断记录中。

## 手动更新

1. 点击底部「检查更新」。启动时和后台均不会自动检查。
2. 找到兼容的新版本后，核对版本号，确认「安装并重启」；取消不产生修改。
   预览版可接收后续预览版。先保存编辑，重启会丢弃未保存内容。
3. 只下载本仓库对应系统 / 架构的 GitHub 包，校验 GitHub SHA-256 摘要，
   暂存自有文件，再启动临时助手。助手等待应用退出，带备份安装，并重新打开。

不发送服务商和 Key 数据。压缩包上限 10 MiB，自有安装文件上限 5 MiB。
拒绝路径穿越、链接、额外文件和平台不匹配的可执行文件。
更新不触碰服务商 / 客户端文件。安装失败会恢复旧文件；如果外部修改冲突，保留备份并提示。

从解压后的完整包启动，不要仅复制一个二进制。安装目录必须可写。
关闭其他 pp-switch 窗口，更新时不要编辑应用文件。
只读目录需要移到用户可写位置或手动替换。网络故障 / 限流可稍后手动重试。
重启失败则自行重新打开。包未签名：GitHub 摘要保证下载完整性，不提供独立发布者签名。
macOS 未进行 Apple Developer 签名或公证。

## 限制、恢复与性能

拒绝的配置保持原样。通过客户端修复已有设置后再试。
文件已变更 / 符号链接错误需要检查目标；应用不会覆盖链接或外部改动。
Codex 多文件写入可恢复，但不具备跨文件全局原子性。
下次应用操作会尝试恢复；冲突时保留日志，不覆盖他人的改动。

关掉窗口，进程退出，没有托盘或常驻助手。
原预算：安装 ≤5 MiB、空闲 RSS ≤20 MiB、启动 / 导入峰值 ≤30 MiB、冷启动可操作 ≤300 ms。
**这是目标，macOS 内存曾未达标。** [验证记录](verification.md) 公开各版本 / 平台、硬件、方法与原始数据。
新进程事件循环就绪不代表冷启动或输入延迟；导入压力峰值未验证。
不要用历史数据或另一系统的数据替代当前版本实测。

[README](../README_ZH.md) · [英文规范](../SPEC.md)
