# Pure Provider Switch (pp-switch) 使用指南

[English](user-guide.md) · 简体中文 · [日本語](user-guide-ja.md)

pp-switch 保存服务商，切换 **Base URL 与 API Key**；0.0.5 开发版还可同步你明确填写的可选模型。
0.0.5 还支持限定的原生／API 路由切换；不管理账号、不转换协议。服务商需要兼容所选模型和协议。[下载](https://github.com/phonowell/pp-switch/releases)。

## 第一次切换

1. 先在客户端本身完成初始化；0.0.5 也支持下文的原生账号路由。
2. 打开 pp-switch，在顶部标签选择客户端。
3. 「添加」，填写名称、HTTP(S) 地址和非空 Key，检查可选模型后保存。
4. 点击该行铅笔图标后面的「切换」图标。客户端没有重读配置时，重启客户端。

保存只改变本地服务商列表。蓝色「使用中」表示客户端的**地址、Key 和已填写的模型设置**均与该条目匹配；
状态从客户端文件读取，不依赖记住的活动 ID。每行的铅笔按钮打开编辑弹窗；取消会丢弃草稿，保存不会应用。当前已匹配的服务商不可重复切换。
删除只删除本地条目，不恢复或更改客户端。
同一客户端的名称不能重复（忽略首尾空格和大小写）；同名导入自动使用首个可用数字后缀，如 `名称 (2)`。
相同连接仍可用不同名称保存；已有同名记录保留。地址、Key 和已填写模型设置均匹配的条目都会显示「使用中」，每条仍可独立编辑。
列表取消独立选中态。已移除底部状态和全局切换区，所有组件均无悬浮提示。
客户端标签右侧的小齿轮进入同一窗口中的设置页；「返回」回到列表。
界面跟随系统浅色／深色模式，包括已经打开的弹窗；切换外观会保留草稿、文本选择和焦点。

**「使用中」只确认受支持的用户级文件。** 应用不验证运行中的会话，也不发送测试请求。新服务商必须支持保留的模型；Claude 启动时的未知模型目录警告本身不表示切换失败。请将实际请求错误与启动时的模型名称区分开。

## 可选模型（0.0.5 开发版）

模型均为可选，包括 Claude。已保存的无模型条目直接切换 URL／Key；这些字段匹配时显示「使用中」。
已有记录和导入预览优先保留明确模型；Claude 缺失默认值时先取已提供的 Sonnet、Opus、Haiku 映射。
否则，五个客户端都只在 URL 和 Key 完全匹配时，从受支持的本机配置预填模型；Claude 的分级别名按已有映射解析，不沿用其他连接的模型。

Claude 新增、编辑和导入没有具体模型时，在输入框实际预填可编辑的 `claude-sonnet-5`。
这是产品建议，不保证所有服务商兼容；请确认支持或替换。清空可保留客户端模型。
预填仍是未保存的草稿，需要先保存、再切换。其他客户端的新建模型字段保持空白；DSH 从原生账号切到 API 时默认使用 `deepseek-flash`。

Claude 同步顶层 `model`、`env.ANTHROPIC_MODEL` 和 Haiku／Sonnet／Opus 映射，可展开映射分别覆盖；
既有 `ANTHROPIC_SMALL_FAST_MODEL` 跟随 Haiku。Codex 修改根级 `config.toml.model`；
Gemini 同步 `.env.GEMINI_MODEL` 和 `settings.json.model.name`，两文件写入可恢复。
Grok 修改当前自定义 profile 的模型；从原生模式建立 PPS profile 时要求明确默认模型。
DSH 修改共同 `agent-default-model` 的模型。清空保存值表示以后不管理它，不删除客户端值或恢复旧设置。
不查询模型目录、不探测兼容性，不编造上下文窗口或 `behavesAs`；未知模型警告可能仍然存在。
旧的无模型记录仍可直接切换。0.0.4 下载版不含这些功能。

## 客户端前提与精确写入范围

| 客户端 | 用户级位置 | 管理的值 |
| --- | --- | --- |
| Claude Code | `~/.claude/settings.json` | `env.ANTHROPIC_BASE_URL` 与既有 `env.ANTHROPIC_AUTH_TOKEN` 或 `env.ANTHROPIC_API_KEY` |
| Codex | `~/.codex/config.toml`，必要时加 `~/.codex/auth.json` | 当前自定义服务商的 `base_url`；既有 `experimental_bearer_token` 或 `OPENAI_API_KEY` |
| Gemini CLI | `~/.gemini/.env` | `GOOGLE_GEMINI_BASE_URL` 与 `GEMINI_API_KEY` |
| Grok Build | `~/.grok/config.toml` | 当前自定义 profile 的 `base_url` 和内联 `api_key` |
| DeepSeek Harness CLI／Web | `$DSH_HOME/cordis.patch.yml` 和 `.credentials.yaml`（默认 `~/.dsh`） | 共享路由的 `baseURL` 和已有 `refs.<apiKeyEnv>` |

`~` 是各系统的用户主目录。Claude 支持 `CLAUDE_CONFIG_DIR`，Codex 支持 `CODEX_HOME`。
Gemini 的设置还包含限定的认证选择；明确模型才修改 `settings.json.model.name`。

**Claude：** 两种 Key 字段都不存在时使用 `ANTHROPIC_AUTH_TOKEN`。
同时存在两个字段、使用 `apiKeyHelper` 或启用 Bedrock / Vertex / Foundry 时拒绝切换。
不管理 OAuth 登录，不转换云平台认证。

**Codex：** 兼容自定义路由保留选择和协议。Bearer 路由不能同时要求 OpenAI 认证；
文件 Key 路由要求已有 API Key 认证、无活动 OAuth、无外部／keyring Key 存储。
从原生登录切换时，改用专属 `pps` Responses／bearer 表，OAuth 凭据和凭据存储设置保持原样。
非 PPS 的同名表、活动 profile 或强制登录覆盖会阻止此切换；仍不支持 `env_key` 等外部 Key 来源。

**Gemini：** 支持原生／未设置、`oauth-personal` 和 `gemini-api-key`。
切换服务商时将 `security.auth.selectedType` 设为 `gemini-api-key`，拒绝云平台／Vertex 认证及冲突的其他 Key／URL 来源。
明确模型才修改 `settings.json.model.name`；OAuth 缓存保持原样。

**Grok：** 既有内联 Key 自定义 profile 保留选择和协议；从原生模式切换时选择专属 `pps` Responses profile，并要求明确默认模型。
其他 profile 和 `auth.json` 保持原样。

**DSH：** 先初始化含 `@deepseek-ai/dsh-base` 的 `headless`／`web` profile。
home patch 在 profile patch 后加载，CLI／Web 共用。支持的 API 路由保留原 Key 引用；
从原生账号切换时选择 `deepseek-official`，使用既有引用（默认 `DEEPSEEK_API_KEY`），仅在缺失时建立 version-1 API Key 容器。
拒绝没有共同覆盖的 profile 路由冲突和外部凭据；保留账号记录、manifest、profile patch 和完整继承目录。
禁用 HMR 时重启 dsh；命令行 patch、自定义 bundle、其他进程环境仍可能覆盖文件匹配状态。

## 官方还原（0.0.5 开发版）

五个客户端列表第一行固定为「官方」，只有切换按钮。它恢复原生路由，不是历史配置备份。
不执行登录或网络请求；必要时通过客户端重新登录并重启。

- Claude 删除 API URL／Key／Token、五个受管理模型环境字段和顶层模型；保留原生凭据、hooks、权限、模型目录、现有代理和上下文设置。拒绝外部 helper／云平台模式。
- Codex 清除根级服务商／模型选择、仅 API 的强制登录及 `auth.json.OPENAI_API_KEY`／API 认证方式；保留 OAuth Token 和未启用的自定义表。
- Gemini 清除受管理 URL／Key／模型环境项，选择 `oauth-personal` 并删除 `model.name`；保留 OAuth 缓存和其他设置。
- Grok 清除默认选择及指向同一 profile 的摘要选择；保留未启用的 profile 表和原生凭据。
- DSH 选择 `deepseek-account`／`deepseek-flash`，通过完整共同覆盖清除账号适配器的自定义地址；保留 API／账号凭据、模型目录和两份 profile patch。

「官方」状态只检查文件中的原生路由／认证，不证明账号已登录。不支持的配置在写入前拒绝。
多文件修改采用与服务商切换相同的可恢复事务。

仅编辑用户级配置。项目配置、启动参数和客户端环境变量可能优先。
pp-switch 能看到的覆盖会被提示；另一个终端的环境可能不同。
请自行处理启动环境覆盖；应用不修改 shell 配置或系统环境变量。

## 添加服务商

手动填写名称、地址和 Key。0.0.5 开发版支持浏览器打开
`ccswitch://v1/import` / `ppswitch://v1/import` 服务商链接，显示未保存的预览。
核对名称、地址、遮罩 Key 和可选模型后点击「保存」；保存不修改客户端配置。
不请求远程配置、不导入其他设置。已打开的草稿保持原样；完成或取消后重新打开链接。
「设置 → 注册导入链接」只注册无人使用的协议，保留其他已安装应用的处理程序。
启动时不注册。0.0.4 下载版不支持此流程；已有服务商记录保留。

sub2api 当前生成的 Claude 链接不含模型。如果受支持的 Claude 配置中已有完全相同的 URL 和 Key，预览会预填其模型；否则建议 `claude-sonnet-5`，可替换或清空。单独保存不会应用客户端文件。
仅含 Claude 分级映射的链接依次用 Sonnet、Opus、Haiku 作为可见默认模型；明确的默认值优先。
丢弃余额脚本，不执行它。

## 语言与本地数据

Linux 请安装 Noto Sans CJK（Debian/Ubuntu：`sudo apt install fonts-noto-cjk`）或 WenQuanYi Zen Hei。Xft 不会自动为缺失字形回退，必须有已安装的中日韩字体。应用不分发字体文件。

界面支持英文、简体中文、繁体中文、日语。首次跟随系统语言，不支持的语言回退英文。
「设置 → 语言」可即时更换标签、对话框和状态语言，保留草稿与当前编辑记录。

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

macOS 0.0.2 需手动下载这次更新：旧版严格文件列表不接受新的应用图标；0.0.3 后续更新会覆盖该资源。

1. 点击「设置 → 检查更新」。启动时和后台均不会自动检查。
2. 找到兼容的新版本后，核对版本号，确认「安装并重启」；取消不产生修改。
   预览版可接收后续预览版。先保存编辑，重启会丢弃未保存内容。
3. 只下载本仓库对应系统 / 架构的 GitHub 包，校验 GitHub SHA-256 摘要，
   暂存自有文件，再启动临时助手。助手等待应用退出，带备份安装，并重新打开。

更新不发送服务商和 Key 数据，只替换应用自有文件。
安装失败会恢复旧文件；如果外部修改冲突，保留备份并提示。

从解压后的完整包启动，不要仅复制一个二进制。安装目录必须可写。
关闭其他 pp-switch 窗口，更新时不要编辑应用文件。
只读目录需要移到用户可写位置或手动替换。网络故障 / 限流可稍后手动重试。
重启失败则自行重新打开。包未签名：GitHub 摘要保证下载完整性，不提供独立发布者签名。
macOS 未进行 Apple Developer 签名或公证。

## 排错

| 情况 | 处理 |
| --- | --- |
| 配置不完整或不受支持 | 对照上面的客户端前提，通过客户端本身初始化；被拒绝的配置保持原样。 |
| 客户端仍使用旧服务商 | 重启客户端，检查项目配置、启动参数与环境变量覆盖。 |
| 文件已变更或符号链接被拒绝 | 重试前检查目标；应用不会覆盖链接或外部改动。 |
| 更新无法连接 GitHub / HTTP 403 | 稍后手动重试，出口可能耗尽 GitHub 匿名 API 额度；应用不要求 GitHub 登录。 |
| 更新安装或重启失败 | 保管提示的备份，检查完整包及写入权限，重新打开应用或手动下载。 |
| Linux 缺少中日韩字形 | 安装上面列出的 CJK 字体。 |

Codex 多文件写入可恢复，但不具备跨文件全局原子性。
下次使用会尝试恢复；冲突时保留日志，不覆盖外部改动。
关闭主窗口或选择 macOS「退出」会结束进程及所有弹窗，没有常驻助手。
当前性能结果与限制见[验证记录](verification.md)，预算定义见 [SPEC PP-11](../SPEC.md#acceptance-contract)。

[README](../README_ZH.md) · [英文规范](../SPEC.md)
