# Hermes Desktop

<img width="100%" alt="HERMES DESKTOP" src="https://github.com/user-attachments/assets/80585955-3bae-4aee-af90-a1e61757ccb8" />

## 语言

- 英文：`README.md`
- 简体中文：`README.zh-CN.md`

> **本项目仍在积极开发中。** 功能可能会变化，部分内容也可能出现问题。如果你遇到 bug 或有新的想法，欢迎在 GitHub 上提交 issue。

Hermes Desktop 是一个原生桌面应用，用于安装、配置并与 [Hermes Agent](https://github.com/NousResearch/hermes-agent) 进行交互 —— 一个具有工具调用、多平台消息传递和闭环学习能力的自我改进 AI 助手。

应用将安装、提供商配置和日常使用整合到同一个图形界面中，而不是要求你手动维护 CLI。它调用官方 Hermes 安装脚本，将 Hermes 存储在 `~/.hermes` 中，并提供聊天、会话、档案、记忆、技能、工具、日程安排、消息网关等 GUI 功能。

## 安装

请从 [Releases](https://github.com/fathah/hermes-desktop/releases/) 页面下载最新构建版本。

| 平台  | 文件                  |
| ----- | --------------------- |
| macOS | `.dmg`                |
| Linux | `.AppImage` 或 `.deb` |
| Linux (Fedora) | `.rpm` |
| Windows | `.exe` (NSIS 安装程序) |

### Windows (winget)

当清单被接受到 [`microsoft/winget-pkgs`](https://github.com/microsoft/winget-pkgs) 后，可以使用以下命令安装：

```powershell
winget install NousResearch.HermesDesktop
```

在此之前，请从 Releases 页面下载 `.exe`。

> **macOS 用户：** 应用目前没有进行代码签名或 notarize，首次启动时 macOS 可能会阻止运行。安装后请执行：
>
> ```bash
> xattr -cr "/Applications/Hermes Agent.app"
> ```
>
> 或者右键应用，选择 **Open**，然后在弹窗中再次点击 **Open**。

## 功能包含

- **首次引导式安装** — 带进度跟踪和依赖解析的 Hermes Agent 安装向导
- **三种连接模式** — 本地模式 (127.0.0.1:8642)、远程模式（自定义 URL + API Key）、SSH 隧道模式（无需暴露端口）
- **多提供商支持** — OpenRouter、Anthropic、OpenAI、Google (Gemini)、xAI (Grok)、Nous Portal、Qwen、MiniMax、Hugging Face、Groq、DeepSeek、Mistral、Together AI、Fireworks AI、Cerebras、Perplexity、NVIDIA NIM、Z.ai/GLM，以及本地 OpenAI 兼容端点（LM Studio、Ollama、vLLM、llama.cpp）
- **流式聊天界面** — SSE 流式传输、工具进度指示器、Markdown 渲染、代码语法高亮
- **39+ 条斜杠命令** — `/new`、`/clear`、`/btw`、`/approve`、`/deny`、`/status`、`/reset`、`/compact`、`/undo`、`/retry`、`/fast`、`/compress`、`/usage`、`/debug`、`/goal`、`/steer`、`/queue`、`/update`、`/web`、`/image`、`/browse`、`/code`、`/file`、`/shell`、`/help`、`/tools`、`/skills`、`/reload-skills`、`/kanban`、`/curator`、`/model`、`/memory`、`/persona`、`/version` 等
- **Token 用量追踪** — 实时显示提示/补全 Token 数量、费用和限流信息
- **会话管理** — 全文搜索（SQLite FTS5）、按日期分组的历史记录、恢复和搜索对话
- **档案切换** — 创建、删除和切换具有独立配置的独立 Hermes 环境
- **16 个工具集** — 网页搜索、浏览器、终端、文件操作、代码执行、视觉、图片生成、TTS、技能、记忆、会话搜索、澄清、委托、定时任务、MoA 和待办事项管理
- **记忆系统** — 查看/编辑记忆条目、用户资料记忆、字符容量追踪，以及可发现的记忆提供商
- **人格编辑器** — 编辑和重置智能体 SOUL.md 个性
- **保存的模型** — 跨提供商模型配置的 CRUD 管理和自动发现
- **定时任务** — Cron 任务构建器，支持多种投递目标（ Telegram、Discord、邮件等）
- **16 个消息网关** — Telegram、Discord、Slack、WhatsApp、Signal、Matrix、Mattermost、邮件（IMAP/SMTP）、短信（Twilio/Vonage）、iMessage（BlueBubbles）、钉钉、飞书/Lark、企业微信、微信（iLink Bot）、Webhook、Home Assistant
- **MCP 服务器管理** — 添加、删除、测试和管理 Model Context Protocol 服务器
- **SSH 隧道支持** — 通过 SSH 隧道连接远程 Hermes，无需暴露端口或 API Key
- **Hermes Office (Claw3d)** — 3D 可视化界面，带开发服务器和适配器管理
- **Curator** — 自主后台技能库维护智能体
- **沙箱后端** — 支持 Docker、SSH、Modal、Daytona、Vercel Sandbox 用于代码执行
- **上下文文件** — 管理附加到对话的文件
- **看板** — 可视化任务管理，带看板和卡片
- **备份、导入和调试转储** — Settings 中的完整数据备份/恢复和系统诊断
- **日志查看器** — 直接从设置界面查看网关、智能体和错误日志
- **自动更新** — 检查并安装 Hermes Agent 更新
- **国际化** — 支持 8 种语言（英语、西班牙语、印尼语、日语、葡萄牙语 BR/PT、简体中文、繁体中文）
- **测试套件** — SSE 解析器、IPC 处理器、Preload API、安装程序工具和常量验证（使用 Vitest）

## 工作方式

首次启动时，应用会：

1. 询问你要**本地运行** Hermes、连接到**远程** Hermes API 服务器，还是使用 **SSH 隧道**
2. **本地模式：** 检查 `~/.hermes` 中是否已安装 Hermes；如未安装，则使用依赖解析（Git、uv、Python 3.11+）运行官方 Hermes 安装程序
3. **远程模式：** 提示输入远程 API URL 和 API Key，验证连接
4. **SSH 隧道模式：** 配置 SSH 连接以隧道到远程 Hermes 实例
5. 提示选择 API 提供商或本地模型端点
6. 通过 Hermes 配置文件保存提供商配置和 API Key
7. 设置完成后进入主工作区

在本地模式下，聊天请求通过 SSE 流式传输从 `http://127.0.0.1:8642` 发出。远程模式下，应用使用相同的流式协议与配置的远程 URL 通信。SSH 隧道模式创建到远程 Hermes 实例的本地端口转发。

## 主界面

| 界面 | 描述 |
| ----- | ----- |
| **Chat** | 带斜杠命令、工具进度和 Token 追踪的流式对话界面 |
| **Sessions** | 浏览、搜索和恢复历史对话 |
| **Agents** | 创建、删除和切换 Hermes 档案 |
| **Skills** | 浏览、安装和管理内置和已安装的技能 |
| **Models** | 跨提供商管理保存的模型配置，支持自动发现 |
| **Memory** | 查看/编辑记忆条目、用户资料，配置记忆提供商 |
| **Soul** | 编辑活动档案的人格（SOUL.md） |
| **Tools** | 启用或禁用单独的 16 个工具集 |
| **Schedules** | 创建和管理带投递目标的 Cron 任务 |
| **Gateway** | 配置和控制 16 个消息平台集成 |
| **MCP Servers** | 添加、删除、测试和管理 MCP 服务器 |
| **Office** | Claw3d 可视化界面设置和管理 |
| **Curator** | 查看和触发自主技能库维护 |
| **Plugins** | 启用/禁用 Hermes Agent 插件 |
| **Context Files** | 管理附加到对话的文件 |
| **Kanban** | 带看板的可视化任务管理 |
| **Security** | 安全设置和配置 |
| **Usage** | Token 用量统计和洞察 |
| **Settings** | 提供商配置、凭证池、SSH 隧道、备份/导入、日志查看器、网络设置、主题、语言 |

## 支持的提供商

### LLM 提供商

| 提供商 | 说明 |
| ----- | ---- |
| **OpenRouter** | 通过单一 API 访问 200+ 模型（推荐） |
| **Anthropic** | 直接访问 Claude |
| **OpenAI** | 直接访问 GPT |
| **OpenAI Codex** | GitHub Copilot 集成 |
| **Google (Gemini)** | Google AI Studio |
| **xAI (Grok)** | Grok 模型 |
| **Nous Portal** | 提供免费套餐 |
| **Qwen** | 通义千问模型（含 OAuth） |
| **MiniMax** | 全球和中国端点（含 OAuth） |
| **Hugging Face** | 通过 HF Inference 访问 20+ 开源模型 |
| **Groq** | 快速推理 |
| **DeepSeek** | DeepSeek 模型 |
| **Mistral** | Mistral AI 模型 |
| **Together AI** | 通过 Together 访问开源模型 |
| **Fireworks AI** | 快速推理 |
| **Cerebras** | 超快速推理 |
| **Perplexity** | 实时搜索模型 |
| **NVIDIA NIM** | NVIDIA NIM 推理端点 |
| **Z.ai / GLM** | 智谱/GLM 模型 |
| **本地/自定义** | 任何 OpenAI 兼容端点 |

内置本地预设：

- LM Studio
- Ollama
- vLLM
- llama.cpp

远程 API 预设：

- Groq
- DeepSeek
- Together AI
- Fireworks AI
- Cerebras
- Mistral

### 消息平台

Telegram、Discord、Slack、WhatsApp、Signal、Matrix/Element、Mattermost、邮件（IMAP/SMTP）、短信（Twilio & Vonage）、iMessage（BlueBubbles）、钉钉、飞书/Lark、企业微信、微信（iLink Bot）、Webhook 和 Home Assistant。

### 工具集成

Exa Search、Parallel API、Tavily、Firecrawl、FAL.ai（图片生成）、Honcho、Browserbase、Weights & Biases、Tinker 和语音工具。

## 开发

### 前置要求

- Node.js 18+ 和 npm
- Rust 1.70+（用于 Tauri 后端）
- 可运行 Hermes 安装程序的类 Unix shell 环境
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

UI 中支持的设置路径：

- `OpenRouter`
- `Anthropic`
- `OpenAI`
- `OpenAI Codex`
- `Google (Gemini)`
- `xAI (Grok)`
- `Nous Portal`
- 通过 OpenAI 兼容 Base URL 使用 `Local LLM`

内置本地模型预设：

- LM Studio
- Ollama
- vLLM
- llama.cpp

Hermes 相关文件位于：

- `~/.hermes`
- `~/.hermes/.env`
- `~/.hermes/config.yaml`
- `~/.hermes/hermes-agent`
- `~/.hermes/profiles/` — 命名的档案目录
- `~/.hermes/state.db` — 会话历史数据库
- `~/.hermes/cron/jobs.json` — 定时任务
- `~/.hermes/mcp.json` — MCP 服务器配置

## 技术栈

- **Tauri 2** — 跨平台桌面外壳（非 Electron）
- **React 19** — UI 框架
- **TypeScript 5.9** — 跨主进程和渲染器的类型安全
- **Tailwind CSS 4** — 实用优先样式
- **Vite 7** — 快速开发服务器和构建工具
- **rusqlite** — 带 SQLite FTS5 全文搜索的本地会话存储
- **i18next** — 国际化框架
- **Vitest** — 测试运行器

## 说明

- 桌面应用依赖上游 Hermes Agent 项目来完成智能体行为和工具执行。
- 内置安装器会以 `--skip-setup` 运行官方 Hermes 安装脚本，再在 GUI 中完成提供商配置。
- 本地模型提供商不需要 API Key，但兼容服务必须已经启动。
- 支持替代 npm registry 路由，适用于网络访问受限的环境。
- SSH 隧道模式需要通过 SSH 密钥无密码访问远程服务器。

## 贡献

欢迎贡献！请查看 [贡献指南](CONTRIBUTING.zh-CN.md) 开始参与。如果你不知道从哪里入手，可以先看看 [open issues](https://github.com/NousResearch/hermes-desktop/issues)。如果你发现 bug 或希望提出功能请求，也欢迎 [提交 issue](https://github.com/NousResearch/hermes-desktop/issues/new)。

## 相关项目

如需了解核心智能体、文档和 CLI 工作流，请查看 Hermes Agent 主仓库：

- https://github.com/NousResearch/hermes-agent
