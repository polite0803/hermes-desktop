# Hermes Desktop

<img width="100%" alt="HERMES DESKTOP" src="https://github.com/user-attachments/assets/80585955-3bae-4aee-af90-a1e61757ccb8" />

## 语言

- 英文：`README.md`
- 简体中文：`README.zh-CN.md`

> **本项目仍在积极开发中。** 功能可能会变化，部分内容也可能出现问题。如果你遇到 bug 或有新的想法，欢迎[提交 issue](https://github.com/fathah/hermes-desktop/issues)。欢迎贡献！

Hermes Desktop 是一个原生桌面应用，用于安装、配置并与 [Hermes Agent](https://github.com/NousResearch/hermes-agent) 进行交互——Hermes Agent 是一个具备工具调用、多平台消息和闭环学习能力的自我改进 AI 助手。

应用将安装、提供商配置和日常使用整合到同一个图形界面中，无需手动维护 CLI。它使用官方 Hermes 安装脚本，将 Hermes 存储在 `~/.hermes` 中，并提供聊天、会话、档案、记忆、技能、工具、定时任务、消息网关等 GUI 功能。

## 安装

请从 [Releases](https://github.com/fathah/hermes-desktop/releases/) 页面下载最新构建版本。

| 平台           | 文件                    |
| -------------- | ----------------------- |
| macOS          | `.dmg`                  |
| Linux (通用)   | `.AppImage`             |
| Linux (Debian) | `.deb`                  |
| Linux (Fedora) | `.rpm`                  |
| Windows        | `.exe` (NSIS 安装器)    |

### Windows (winget)

当清单被接受到 [`microsoft/winget-pkgs`](https://github.com/microsoft/winget-pkgs) 后，可以使用以下命令安装：

```powershell
winget install NousResearch.HermesDesktop
```

在此之前，请从 Releases 页面下载 `.exe`。

> **Windows 用户：** 安装器未进行代码签名。Windows SmartScreen 会在首次启动时发出警告——点击"更多信息"→"仍要运行"。

> **WSL 用户：** 如果安装器在 `Switching to root user to install dependencies...` 处卡住，说明 Playwright 正在等待 sudo 密码但没有 TTY 可读取。请为安装过程授予免密 sudo，完成后恢复：
>
> ```bash
> echo "$USER ALL=(ALL) NOPASSWD: ALL" | sudo tee /etc/sudoers.d/hermes-install
> # …重新运行安装器；完成后：
> sudo rm /etc/sudoers.d/hermes-install
> ```
>
> 追踪于 [#109](https://github.com/fathah/hermes-desktop/issues/109)。

### Fedora (RPM)

```bash
sudo dnf install ./hermes-desktop-<version>.rpm
```

> **Fedora 用户：** `.rpm` 未进行 GPG 签名。如果系统强制签名检查，请在安装命令后附加 `--nogpgcheck`。`.rpm` 构建不支持自动更新（`electron-updater` 的限制）；请重新安装新的 `.rpm` 来更新。

## 预览

<table>
<tr>
<td width="50%" align="center"><b>Office</b><br/><img width="100%" alt="Office" src="https://github.com/user-attachments/assets/214bfa60-48ec-4449-be40-370628205147" /></td>
<td width="50%" align="center"><b>Chat</b><br/><img width="100%" alt="Chat" src="https://github.com/user-attachments/assets/ca84a56c-4d14-4775-96bb-c725069988be" /></td>
</tr>
<tr>
<td width="50%" align="center"><b>Profiles</b><br/><img width="100%" alt="Profiles" src="https://github.com/user-attachments/assets/bd812e4a-bbdc-4141-b3a8-1ab5b0e561d4" /></td>
<td width="50%" align="center"><b>Tools</b><br/><img width="100%" alt="Tools" src="https://github.com/user-attachments/assets/ad051fbe-055d-40d2-b6dd-959c522412d2" /></td>
</tr>
<tr>
<td width="50%" align="center"><b>Settings</b><br/><img width="100%" alt="Settings" src="https://github.com/user-attachments/assets/b3f7e0d8-b087-4935-b57c-f8db30491f2e" /></td>
<td width="50%" align="center"><b>Skills</b><br/><img width="100%" alt="Skills" src="https://github.com/user-attachments/assets/508c3501-52eb-419d-8cfd-06268875ff62" /></td>
</tr>
</table>

## 功能特性

- **首次引导式安装** — Hermes Agent 安装向导，含进度追踪和依赖解析
- **三种连接模式** — 本地 (127.0.0.1:8642)、远程 (自定义 URL + API Key)、SSH 隧道 (无需开放端口)
- **多提供商支持** — 26 个提供商，包括 OpenRouter、Anthropic、OpenAI、OpenAI Codex、Google (Gemini)、xAI (Grok 及 OAuth)、Nous Portal、Qwen (含 OAuth)、MiniMax (含 OAuth)、Hugging Face、Groq、DeepSeek、Mistral、Together AI、Fireworks AI、Cerebras、Perplexity、NVIDIA NIM、Z.ai/GLM、Gemini CLI OAuth、Kimi Coding Plan，以及本地 OpenAI 兼容端点 (LM Studio、Ollama、vLLM、llama.cpp)
- **流式聊天界面** — SSE 流式传输、工具进度指示器、Markdown 渲染和语法高亮
- **33 个斜杠命令** — `/new`、`/clear`、`/btw`、`/approve`、`/deny`、`/status`、`/reset`、`/compact`、`/undo`、`/retry`、`/fast`、`/compress`、`/usage`、`/debug`、`/goal`、`/steer`、`/queue`、`/update`、`/web`、`/image`、`/browse`、`/code`、`/file`、`/shell`、`/help`、`/tools`、`/skills`、`/reload-skills`、`/kanban`、`/curator`、`/model`、`/memory`、`/persona`、`/version`
- **Token 用量追踪** — 实时 prompt/completion token 计数、费用显示和速率限制信息
- **会话管理** — 全文搜索 (SQLite FTS5)、按日期分组的历史记录、跨对话恢复和搜索
- **档案切换** — 创建、删除和切换独立的 Hermes 环境，配置隔离
- **16 个工具集** — 网页搜索、浏览器、终端、文件操作、代码执行、视觉理解、图像生成、TTS、技能调用、记忆访问、会话搜索、澄清查询、委托/调用其他代理、定时任务管理、混合代理 (MoA) 和待办管理
- **记忆系统** — 查看/编辑记忆条目、用户画像记忆、字符容量追踪和可发现的记忆提供商
- **人格编辑器** — 编辑和重置代理的 SOUL.md 人格；包含内置人格预设
- **已保存模型** — 跨提供商的模型配置 CRUD 管理，支持自动发现
- **定时任务** — Cron 任务构建器，支持多种投递目标 (Telegram、Discord、邮件等)
- **16 个消息网关** — Telegram、Discord、Slack、WhatsApp、Signal、Matrix、Mattermost、Email (IMAP/SMTP)、SMS (Twilio/Vonage)、iMessage (BlueBubbles)、钉钉、飞书/Lark、企业微信、微信 (iLink Bot)、Webhooks、Home Assistant
- **MCP 服务器管理** — 添加、移除、测试和管理 Model Context Protocol 服务器 (stdio 和 HTTP 传输)
- **SSH 隧道支持** — 通过 SSH 隧道连接远程 Hermes，无需开放端口或 API Key
- **Hermes Office (Claw3d)** — 可视化 3D 界面，含开发服务器和适配器管理
- **Curator** — 自主后台技能库维护代理，提供使用排名洞察
- **终端后端配置** — 通过 `config.yaml` 配置沙盒执行后端
- **上下文文件** — 管理附加到对话的文件，提供持久上下文
- **看板** — 可视化任务管理，支持看板创建、任务 CRUD、分配、阻塞、归档和评论
- **插件管理** — 从 UI 启用/禁用 Hermes Agent 插件
- **备份、导入与调试转储** — 完整数据备份/恢复和系统诊断，从设置中操作
- **日志查看器** — 直接从设置页面查看网关、代理和错误日志
- **自动更新** — 检查并安装 Hermes Agent 更新，含下载进度
- **凭证池** — 管理每个提供商的多个 API Key 用于轮换
- **安全设置** — 外部链接、Webview 导航和应用内导航的 URL 白名单控制
- **系统托盘** — 系统托盘图标，含显示窗口和退出操作
- **国际化支持** — 支持 8 种语言的国际化框架 (英文、西班牙文、印尼文、日文、葡萄牙文 BR/PT、简体中文、繁体中文)
- **测试套件** — 附件工具、i18n 提供者、键盘快捷键、提供商检测、i18n 索引和常量验证，使用 Vitest

## 工作方式

首次启动时，应用会：

1. 询问你要**本地**运行 Hermes、连接到**远程** Hermes API 服务器，还是使用 **SSH 隧道**
2. **本地模式：** 检查 `~/.hermes` 中是否已安装 Hermes；若未安装，运行官方安装器进行依赖解析 (Git、uv、Python 3.11+)
3. **远程模式：** 提示输入远程 API URL 和 API Key，验证连接
4. **SSH 隧道模式：** 配置 SSH 连接以隧道到远程 Hermes 实例
5. 提示选择 API 提供商或本地模型端点
6. 通过 Hermes 配置文件保存提供商配置和 API Key
7. 设置完成后启动主工作区

本地模式下，聊天请求通过 `http://127.0.0.1:8642` 以 SSE 流式传输。远程模式下，应用与你配置的远程 URL 使用相同的流式协议通信。SSH 隧道模式创建到远程 Hermes 实例的本地端口转发。

## 界面

UI 组织为 **7 个顶级分区**，每个分区包含子页面：

### 主要分区

| 分区 | 图标 | 子页面 |
| --- | --- | --- |
| **Chat** | ChatBubble | 流式对话，含斜杠命令、工具进度、Token 追踪、附件和模型选择器 |
| **Sessions** | Clock | 浏览、搜索 (FTS5)、恢复和删除历史对话 |
| **Profiles** | Users | 创建、删除、切换 Hermes 环境，配置隔离 |
| **AI Studio** | Sparkles | Models、Providers、Skills、Persona (SOUL.md)、Tools、Memory、Context Files |
| **Workspace** | Building | Kanban 看板、Office (Claw3d) |
| **Infrastructure** | KanbanIcon | Gateway (消息网关)、MCP Servers、Plugins、Curator、Schedules |
| **Settings** | SettingsIcon | General (提供商配置、凭证、SSH 隧道、备份/导入、日志、网络、主题、语言)、Security、Usage |

### 页面详情

| 页面             | 描述                                                                                          |
| ---------------- | ---------------------------------------------------------------------------------------------- |
| **Chat**         | 流式对话界面，含斜杠命令、工具进度、Token 追踪、模型选择器和文件附件                          |
| **Sessions**     | 浏览、全文搜索、恢复和删除历史对话                                                              |
| **Agents**       | 创建、删除和切换 Hermes 档案                                                                    |
| **Models**       | 管理跨提供商的已保存模型配置，支持自动发现                                                       |
| **Providers**    | 配置所有支持的 LLM 提供商的 API Key 和端点                                                      |
| **Skills**       | 浏览、安装、搜索 Skills Hub/HuggingFace，管理内置和已安装技能                                    |
| **Soul**         | 编辑当前档案的人格 (SOUL.md)；应用内置人格预设                                                   |
| **Tools**        | 启用或禁用单个工具集 (16 个工具集) + 查看已连接的 MCP 服务器                                     |
| **Memory**       | 查看/编辑记忆条目、用户画像记忆，配置记忆提供商                                                   |
| **Context Files** | 管理附加到对话的文件，提供持久上下文                                                            |
| **Kanban**       | 可视化任务管理，支持看板/任务 CRUD、分配、阻塞、归档和评论                                       |
| **Office**       | Claw3d 可视化界面设置、适配器管理、开发服务器控制                                                |
| **Gateway**      | 配置和控制 16 个消息平台集成，含启动/停止/状态                                                   |
| **MCP Servers**  | 添加、移除、更新、测试 MCP 服务器 (stdio 和 HTTP)；安装 computer-use MCP                        |
| **Plugins**      | 启用/禁用 Hermes Agent 插件                                                                     |
| **Curator**      | 查看自主技能库维护状态、触发运行和使用排名报告                                                   |
| **Schedules**    | 创建和管理 Cron 定时任务，含投递目标                                                            |
| **Security**     | 安全设置，包括外部链接/Webview/应用内导航的 URL 白名单                                          |
| **Usage**        | Token 使用统计和洞察                                                                            |
| **Settings**     | 提供商配置、凭证池、SSH 隧道、备份/导入、日志查看器、网络、主题、语言                           |

## 支持的提供商

### LLM 提供商 (共 26 个)

| 提供商              | 说明                                        |
| ------------------- | ------------------------------------------- |
| **OpenRouter**      | 通过单一 API 访问 200+ 模型 (推荐)          |
| **Anthropic**       | 直接访问 Claude                             |
| **OpenAI**          | 直接访问 GPT                               |
| **OpenAI Codex**    | GitHub Copilot 集成 (无需 API Key)          |
| **Google (Gemini)** | Google AI Studio                           |
| **xAI (Grok)**     | Grok 模型                                  |
| **xAI Grok (OAuth)** | 通过 OAuth 认证访问 Grok                  |
| **Nous Portal**     | 提供免费额度                                |
| **Qwen**            | 通义千问模型                                |
| **Qwen (OAuth)**    | 通过 OAuth 认证访问通义千问                 |
| **MiniMax**         | 全球及中国端点                              |
| **MiniMax (OAuth)** | 通过 OAuth 认证访问 MiniMax                 |
| **Hugging Face**    | 通过 HF Inference 访问 20+ 开源模型         |
| **Groq**            | 快速推理                                    |
| **DeepSeek**        | DeepSeek 模型                              |
| **Mistral**         | Mistral AI 模型                            |
| **Together AI**     | 通过 Together 访问开源模型                  |
| **Fireworks AI**    | 快速推理                                    |
| **Cerebras**        | 超快速推理                                  |
| **Perplexity**      | 实时搜索模型                                |
| **NVIDIA NIM**      | NVIDIA NIM 推理端点                         |
| **Z.ai / GLM**      | 智谱模型                                    |
| **Gemini (CLI OAuth)** | 通过 CLI OAuth 流程访问 Gemini           |
| **Kimi (Coding Plan)** | Kimi 编程计划集成                         |
| **Local/Custom**    | 任何 OpenAI 兼容端点                        |

内置本地预设：LM Studio、Ollama、vLLM、llama.cpp。
内置远程预设：Groq、DeepSeek、Together AI、Fireworks AI、Cerebras、Mistral。

### 消息平台 (共 16 个)

Telegram、Discord、Slack、WhatsApp、Signal、Matrix/Element、Mattermost、Email (IMAP/SMTP)、SMS (Twilio & Vonage)、iMessage (BlueBubbles)、钉钉、飞书/Lark、企业微信、微信 (iLink Bot)、Webhooks、Home Assistant。

### 工具集成 (16 个工具集)

网页搜索、浏览器自动化、终端/Shell、文件操作、代码执行、视觉/图像理解、图像生成、文本转语音 (TTS)、技能调用、记忆访问、会话搜索、澄清查询、委托/调用其他代理、定时任务管理、混合代理 (MoA) 和待办/任务管理。

### 其他工具 API 集成

Exa Search、Parallel API、Tavily、Firecrawl、FAL.ai (图像生成)、Honcho、Browserbase、Weights & Biases、Tinker 和语音工具。

## 开发

### 前置要求

- Node.js 18+ 和 npm
- Rust 1.70+ (用于 Tauri 后端)
- 类 Unix shell 环境 (用于 Hermes 安装器) 或 Windows 上的 PowerShell
- 首次安装时用于下载 Hermes 的网络访问能力

### 安装依赖

```bash
npm install
```

### 启动开发模式

```bash
npm run tauri:dev
```

### 运行检查

```bash
npm run lint
npm run typecheck
```

### 运行测试

```bash
npm run test
npm run test:watch
```

### 构建桌面应用

```bash
npm run tauri:build
```

平台构建：

```bash
npm run build:mac
npm run build:win
npm run build:linux
npm run build:rpm    # 仅 Fedora/RHEL .rpm
```

## 首次设置

应用首次打开时，会自动检测是否存在现有 Hermes 安装；如果没有，会引导你完成安装。

当前 UI 支持的设置路径包括：

- `OpenRouter`
- `Anthropic`
- `OpenAI`
- `OpenAI Codex`
- `Google (Gemini)`
- `xAI (Grok)`
- `Nous Portal`
- `Local LLM` — 通过 OpenAI 兼容 Base URL

内置的本地预设包括：

- LM Studio (`localhost:1234`)
- Ollama (`localhost:11434`)
- vLLM (`localhost:8000`)
- llama.cpp (`localhost:8080`)

内置的远程 API 预设 (用于自定义端点配置)：

- Groq、DeepSeek、Together AI、Fireworks AI、Cerebras、Mistral

Hermes 相关文件位于：

- `~/.hermes`
- `~/.hermes/.env`
- `~/.hermes/config.yaml`
- `~/.hermes/hermes-agent`
- `~/.hermes/profiles/` — 命名档案目录
- `~/.hermes/state.db` — 会话历史数据库 (SQLite，含 FTS5)
- `~/.hermes/cron/jobs.json` — 定时任务
- `~/.hermes/mcp.json` — MCP 服务器配置

## 技术栈

- **Tauri 2** — 跨平台桌面框架 (基于 Rust，非 Electron)
- **React 19** — UI 框架
- **TypeScript 5.9** — 主进程和渲染进程的类型安全
- **Tailwind CSS 4** — 实用优先的样式方案
- **Vite 7** — 快速开发服务器和构建工具
- **rusqlite 0.31** — 本地会话存储，使用 SQLite FTS5 全文搜索 (bundled)
- **i18next 25 + react-i18next 15** — 国际化框架
- **Lucide React** — 图标库
- **react-markdown 10 + remark-gfm** — Markdown 渲染，支持 GFM
- **react-syntax-highlighter 16** — 代码语法高亮
- **Vitest 4** — 测试运行器，使用 jsdom 环境
- **ESLint 9 + Prettier 3** — 代码质量和格式化

## 架构

### 前端 (src/renderer/)
- React 19 SPA，使用 TypeScript
- 基于 `screens/` 的组件化页面架构
- 共享常量、类型和 i18n 位于 `src/shared/`
- TabPage 组件用于各分区内的分组子导航
- 每个工具集的自定义 SVG 图标和每个提供商/网关平台的品牌 Logo

### 后端 (src-tauri/)
- 基于 Rust 的 Tauri 2 应用，包含 32 个模块
- IPC 命令用于所有前后端通信
- SQLite (rusqlite) 用于会话存储和全文搜索
- YAML 配置解析用于 Hermes `config.yaml`
- SSE 流式解析器用于实时聊天响应
- SSH 隧道管理，通过系统 SSH
- 附件暂存用于文件上传
- 自动更新系统，含下载进度追踪

## 说明

- 桌面应用依赖上游 Hermes Agent 项目来完成代理行为和工具执行。
- 内置安装器会以 `--skip-setup` 运行官方 Hermes 安装脚本，再在 GUI 中完成提供商配置。
- 本地模型提供商不需要 API Key，但兼容服务必须已经启动。
- 支持替代 npm 注册表路由，适用于网络受限环境。
- SSH 隧道模式需要通过 SSH Key 实现对远程服务器的免密访问。
- 远程专用模式会隐藏本地相关功能 (档案、会话、部分设置) 并显示提示。

## 贡献

欢迎贡献！请查看[贡献指南](CONTRIBUTING.zh-CN.md)开始参与。如果你不知道从哪里入手，可以先看看 [open issues](https://github.com/NousResearch/hermes-desktop/issues)。如果你发现 bug 或希望提出功能请求，也欢迎[提交 issue](https://github.com/NousResearch/hermes-desktop/issues/new)。

## 相关项目

如需了解核心代理、文档和 CLI 工作流，请查看 Hermes Agent 主仓库：

- https://github.com/NousResearch/hermes-agent
