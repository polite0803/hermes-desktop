<img width="100%" alt="HERMES DESKTOP" src="https://github.com/user-attachments/assets/80585955-3bae-4aee-af90-a1e61757ccb8" />

<br/>
<p align="center">
  <a href="https://hermes-agent.nousresearch.com/docs/"><img src="https://img.shields.io/badge/Docs-hermes--agent.nousresearch.com-FFD700?style=for-the-badge" alt="Documentation"></a>
  <a href="https://t.me/hermes_agent_desktop"><img src="https://img.shields.io/badge/Telegram-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white" alt="Telegram"></a>
  <a href="https://github.com/fathah/hermes-desktop/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License: MIT"></a>
  <a href="https://github.com/fathah/hermes-desktop/releases/"><img src="https://img.shields.io/badge/Download-Releases-FF6600?style=for-the-badge" alt="Releases"></a>
<a href="https://github.com/fathah/hermes-desktop/stargazers">
  <img src="https://img.shields.io/github/stars/fathah/hermes-desktop?style=for-the-badge&color=FFD700&label=Stars" alt="Stars">
</a>
  <a href="https://github.com/fathah/hermes-desktop/releases/">
  <img src="https://img.shields.io/github/downloads/fathah/hermes-desktop/total?style=for-the-badge&color=00B496&label=Total%20Downloads" alt="Downloads">
</a>
</p>

> **This project is in active development.** Features may change, and some things might break. If you run into a problem or have an idea, [open an issue](https://github.com/fathah/hermes-desktop/issues). Contributions are welcome!

## Languages

- English: `README.md`
- 简体中文: `README.zh-CN.md`

Hermes Desktop is a native desktop app for installing, configuring, and chatting with [Hermes Agent](https://github.com/NousResearch/hermes-agent) — a self-improving AI assistant with tool use, multi-platform messaging, and a closed learning loop.

Instead of managing the CLI by hand, the app walks through install, provider setup, and day-to-day usage in one place. It uses the official Hermes install script, stores Hermes in `~/.hermes`, and gives you a GUI for chat, sessions, profiles, memory, skills, tools, scheduling, messaging gateways, and more.

## Install

Download the latest build from the [Releases](https://github.com/fathah/hermes-desktop/releases/) page.

| Platform       | File                    |
| -------------- | ----------------------- |
| macOS          | `.dmg`                  |
| Linux (any)    | `.AppImage`             |
| Linux (Debian) | `.deb`                  |
| Linux (Fedora) | `.rpm`                  |
| Windows        | `.exe` (NSIS installer) |

### Windows (winget)

Once the manifest has been accepted into [`microsoft/winget-pkgs`](https://github.com/microsoft/winget-pkgs), you can install with:

```powershell
winget install NousResearch.HermesDesktop
```

Until then, download the `.exe` from the Releases page.

> **Windows users:** The installer is not code-signed. Windows SmartScreen will warn on first launch — click "More info" → "Run anyway".

> **WSL users:** If the installer stalls at `Switching to root user to install dependencies...`, Playwright is waiting for a sudo password that has no TTY to read from. Grant passwordless sudo for the install, then revert when finished:
>
> ```bash
> echo "$USER ALL=(ALL) NOPASSWD: ALL" | sudo tee /etc/sudoers.d/hermes-install
> # …re-run the installer; once it finishes:
> sudo rm /etc/sudoers.d/hermes-install
> ```
>
> Tracked in [#109](https://github.com/fathah/hermes-desktop/issues/109).

### Fedora (RPM)

```bash
sudo dnf install ./hermes-desktop-<version>.rpm
```

> **Fedora users:** The `.rpm` is not GPG-signed. If your system enforces signature checking, append `--nogpgcheck` to the install command. Auto-update is not supported for `.rpm` builds (limitation of `electron-updater`); reinstall the new `.rpm` to update.

## Preview

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

## Features

- **Guided first-run install** for Hermes Agent with progress tracking and dependency resolution
- **Three connection modes** — local (127.0.0.1:8642), remote (custom URL + API key), SSH tunnel (no exposed ports)
- **Multi-provider support** — 26 providers including OpenRouter, Anthropic, OpenAI, OpenAI Codex, Google (Gemini), xAI (Grok & OAuth), Nous Portal, Qwen (including OAuth), MiniMax (including OAuth), Hugging Face, Groq, DeepSeek, Mistral, Together AI, Fireworks AI, Cerebras, Perplexity, NVIDIA NIM, Z.ai/GLM, Gemini CLI OAuth, Kimi Coding Plan, and local OpenAI-compatible endpoints (LM Studio, Ollama, vLLM, llama.cpp)
- **Streaming chat UI** with SSE streaming, tool progress indicators, markdown rendering, and syntax highlighting
- **33 slash commands** — `/new`, `/clear`, `/btw`, `/approve`, `/deny`, `/status`, `/reset`, `/compact`, `/undo`, `/retry`, `/fast`, `/compress`, `/usage`, `/debug`, `/goal`, `/steer`, `/queue`, `/update`, `/web`, `/image`, `/browse`, `/code`, `/file`, `/shell`, `/help`, `/tools`, `/skills`, `/reload-skills`, `/kanban`, `/curator`, `/model`, `/memory`, `/persona`, `/version`
- **Token usage tracking** — live prompt/completion token counts, cost display, and rate limit info in chat footer
- **Session management** — full-text search (SQLite FTS5), date-grouped history, resume and search across conversations
- **Profile switching** — create, delete, and switch between separate Hermes environments with isolated config
- **16 toolsets** — web search, browser, terminal, file operations, code execution, vision, image generation, TTS, skills, memory, session search, clarify, delegation, cron jobs, MoA, and todo management
- **Memory system** — view/edit memory entries, user profile memory, character capacity tracking, and discoverable memory providers
- **Persona editor** — edit and reset your agent's SOUL.md personality; includes bundled personality presets
- **Saved models** — CRUD management for model configurations across providers with auto-discovery
- **Scheduled tasks** — cron job builder with multiple delivery targets (Telegram, Discord, email, etc.)
- **16 messaging gateways** — Telegram, Discord, Slack, WhatsApp, Signal, Matrix, Mattermost, Email (IMAP/SMTP), SMS (Twilio/Vonage), iMessage (BlueBubbles), DingTalk, Feishu/Lark, WeCom, WeChat (iLink Bot), Webhooks, Home Assistant
- **MCP server management** — add, remove, test, and manage Model Context Protocol servers (stdio & HTTP transport)
- **SSH tunnel support** — tunnel to remote Hermes over SSH without exposed ports or API keys
- **Hermes Office (Claw3d)** — visual 3D interface with dev server and adapter management
- **Curator** — autonomous background skill library maintenance agent with usage-ranked insights
- **Terminal backend configuration** — configurable sandbox execution backend via `config.yaml`
- **Context files** — manage files attached to conversations for persistent context
- **Kanban boards** — visual task management with board creation, task CRUD, assignment, blocking, archiving, and comments
- **Plugin management** — enable/disable Hermes Agent plugins from the UI
- **Backup, import & debug dump** — full data backup/restore and system diagnostics from Settings
- **Log viewer** — view gateway, agent, and error logs directly from the Settings screen
- **Auto-updater** — check for and install Hermes Agent updates with download progress
- **Credential pools** — manage multiple API keys per provider for rotation
- **Security settings** — URL allowlist controls for external links, webview navigation, and in-app navigation
- **Tray icon** — system tray with show window and quit actions
- **i18n ready** — internationalization framework with 8 languages (English, Spanish, Indonesian, Japanese, Portuguese BR/PT, Simplified Chinese, Traditional Chinese)
- **Test suite** — attachment utilities, i18n provider, keyboard shortcuts, provider detection, i18n index, and constants validation with Vitest

## How It Works

On first launch, the app:

1. Asks whether you want to run Hermes **locally**, connect to a **remote** Hermes API server, or use **SSH tunnel**
2. **Local mode:** checks whether Hermes is already installed in `~/.hermes`; if not, runs the official Hermes installer with dependency resolution (Git, uv, Python 3.11+)
3. **Remote mode:** prompts for the remote API URL and API key, validates the connection
4. **SSH tunnel mode:** configures SSH connection to tunnel to a remote Hermes instance
5. Prompts for an API provider or local model endpoint
6. Saves provider config and API keys through Hermes config files
7. Launches the main workspace once setup is complete

In local mode, chat requests go through `http://127.0.0.1:8642` with SSE streaming. In remote mode, the app talks to your configured remote URL with the same streaming protocol. SSH tunnel mode creates a local port forward to the remote Hermes instance.

## Screens

The UI is organized into **7 top-level sections**, each with sub-pages:

### Main Sections

| Section | Icon | Sub-Pages |
| --- | --- | --- |
| **Chat** | ChatBubble | Streaming conversation with slash commands, tool progress, token tracking, attachments, and model picker |
| **Sessions** | Clock | Browse, search (FTS5), resume, and delete past conversations |
| **Profiles** | Users | Create, delete, switch between Hermes environments with isolated config |
| **AI Studio** | Sparkles | Models, Providers, Skills, Persona (SOUL.md), Tools, Memory, Context Files |
| **Workspace** | Building | Kanban boards, Office (Claw3d) |
| **Infrastructure** | KanbanIcon | Gateway (messaging), MCP Servers, Plugins, Curator, Schedules |
| **Settings** | SettingsIcon | General (provider config, credentials, SSH tunnel, backup/import, logs, network, theme, language), Security, Usage |

### Screen Details

| Screen           | Description                                                                                      |
| ---------------- | ------------------------------------------------------------------------------------------------ |
| **Chat**         | Streaming conversation UI with slash commands, tool progress, token tracking, model picker, and file attachments |
| **Sessions**     | Browse, full-text search, resume, and delete past conversations                                   |
| **Agents**       | Create, delete, and switch between Hermes profiles                                               |
| **Models**       | Manage saved model configurations per provider with auto-discovery                                |
| **Providers**    | Configure API keys and endpoints for all supported LLM providers                                  |
| **Skills**       | Browse, install, search Skills Hub/HuggingFace, and manage bundled and installed skills           |
| **Soul**         | Edit the active profile's persona (SOUL.md); apply bundled personality presets                   |
| **Tools**        | Enable or disable individual toolsets (16 toolsets) + view connected MCP servers                 |
| **Memory**       | View/edit memory entries, user profile memory, and configure memory providers                    |
| **Context Files** | Manage files attached to conversations for persistent context                                    |
| **Kanban**       | Visual task management with board/task CRUD, assignment, blocking, archiving, and comments       |
| **Office**       | Claw3d visual interface setup, adapter management, dev server control                            |
| **Gateway**      | Configure and control 16 messaging platform integrations with start/stop/status                  |
| **MCP Servers**  | Add, remove, update, test MCP servers (stdio & HTTP); install computer-use MCP                  |
| **Plugins**      | Enable/disable Hermes Agent plugins                                                              |
| **Curator**      | View autonomous skill library maintenance status, trigger runs, and usage-ranked reports         |
| **Schedules**    | Create and manage cron jobs with delivery targets                                                |
| **Security**     | Security settings including URL allowlists for external/webview/app navigation                  |
| **Usage**        | Token usage statistics and insights                                                              |
| **Settings**     | Provider config, credential pools, SSH tunnel, backup/import, log viewer, network, theme, language|

## Supported Providers

### LLM Providers (26 total)

| Provider            | Notes                                      |
| ------------------- | ------------------------------------------ |
| **OpenRouter**      | 200+ models via single API (recommended)   |
| **Anthropic**       | Direct Claude access                       |
| **OpenAI**          | Direct GPT access                         |
| **OpenAI Codex**    | GitHub Copilot integration (no API key needed) |
| **Google (Gemini)** | Google AI Studio                          |
| **xAI (Grok)**     | Grok models                               |
| **xAI Grok (OAuth)** | Grok via OAuth authentication             |
| **Nous Portal**     | Free tier available                        |
| **Qwen**            | QwenAI models                              |
| **Qwen (OAuth)**    | Qwen via OAuth authentication              |
| **MiniMax**         | Global and China endpoints                 |
| **MiniMax (OAuth)** | MiniMax via OAuth authentication           |
| **Hugging Face**    | 20+ open models via HF Inference           |
| **Groq**            | Fast inference                             |
| **DeepSeek**        | DeepSeek models                           |
| **Mistral**         | Mistral AI models                         |
| **Together AI**     | Open models via Together                   |
| **Fireworks AI**    | Fast inference                            |
| **Cerebras**        | Ultra-fast inference                      |
| **Perplexity**      | Real-time search models                   |
| **NVIDIA NIM**      | NVIDIA NIM inference endpoints             |
| **Z.ai / GLM**      | Chinese models                            |
| **Gemini (CLI OAuth)** | Gemini via CLI OAuth flow               |
| **Kimi (Coding Plan)** | Kimi coding plan integration             |
| **Local/Custom**    | Any OpenAI-compatible endpoint             |

Local presets are included for LM Studio, Ollama, vLLM, and llama.cpp.
Remote presets include Groq, DeepSeek, Together AI, Fireworks AI, Cerebras, and Mistral.

### Messaging Platforms (16 total)

Telegram, Discord, Slack, WhatsApp, Signal, Matrix/Element, Mattermost, Email (IMAP/SMTP), SMS (Twilio & Vonage), iMessage (BlueBubbles), DingTalk, Feishu/Lark, WeCom, WeChat (iLink Bot), Webhooks, and Home Assistant.

### Tool Integrations (16 toolsets)

Web search, browser automation, terminal/shell, file operations, code execution, vision/image understanding, image generation, text-to-speech (TTS), skills invocation, memory access, session search, clarification queries, delegation/calling other agents, cron job management, Mixture of Agents (MoA), and todo/task management.

### Additional Tool API Integrations

Exa Search, Parallel API, Tavily, Firecrawl, FAL.ai (image generation), Honcho, Browserbase, Weights & Biases, Tinker, and voice tools.

## Development

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ (for Tauri backend)
- A Unix-like shell environment for the Hermes installer (or PowerShell on Windows)
- Network access for downloading Hermes during first-run install

### Install dependencies

```bash
npm install
```

### Start the app in development

```bash
npm run tauri:dev
```

### Run checks

```bash
npm run lint
npm run typecheck
```

### Run tests

```bash
npm run test
npm run test:watch
```

### Build the desktop app

```bash
npm run tauri:build
```

Platform packaging:

```bash
npm run build:mac
npm run build:win
npm run build:linux
npm run build:rpm    # Fedora/RHEL .rpm only
```

## First-Time Setup

When the app opens for the first time, it will either detect an existing Hermes installation or offer to install it for you.

Supported setup paths in the UI:

- `OpenRouter`
- `Anthropic`
- `OpenAI`
- `OpenAI Codex`
- `Google (Gemini)`
- `xAI (Grok)`
- `Nous Portal`
- `Local LLM` via an OpenAI-compatible base URL

Built-in presets for local models:

- LM Studio (`localhost:1234`)
- Ollama (`localhost:11434`)
- vLLM (`localhost:8000`)
- llama.cpp (`localhost:8080`)

Remote API presets (for custom endpoint configuration):

- Groq, DeepSeek, Together AI, Fireworks AI, Cerebras, Mistral

Hermes files are managed in:

- `~/.hermes`
- `~/.hermes/.env`
- `~/.hermes/config.yaml`
- `~/.hermes/hermes-agent`
- `~/.hermes/profiles/` — named profile directories
- `~/.hermes/state.db` — session history database (SQLite with FTS5)
- `~/.hermes/cron/jobs.json` — scheduled tasks
- `~/.hermes/mcp.json` — MCP server configuration

## Tech Stack

- **Tauri 2** — cross-platform desktop shell (Rust-based, not Electron)
- **React 19** — UI framework
- **TypeScript 5.9** — type safety across main and renderer processes
- **Tailwind CSS 4** — utility-first styling
- **Vite 7** — fast dev server and build tooling
- **rusqlite 0.31** — local session storage with SQLite FTS5 full-text search (bundled)
- **i18next 25 + react-i18next 15** — internationalization framework
- **Lucide React** — icon library
- **react-markdown 10 + remark-gfm** — Markdown rendering with GFM support
- **react-syntax-highlighter 16** — code syntax highlighting
- **Vitest 4** — test runner with jsdom environment
- **ESLint 9 + Prettier 3** — code quality and formatting

## Architecture

### Frontend (src/renderer/)
- React 19 SPA with TypeScript
- Component-based screen architecture under `screens/`
- Shared constants, types, and i18n under `src/shared/`
- TabPage component for grouped sub-navigation within each section
- Custom SVG icons per toolset and brand logos per provider/gateway platform

### Backend (src-tauri/)
- Rust-based Tauri 2 application with 32 modules
- IPC commands for all frontend-backend communication
- SQLite (rusqlite) for session storage and full-text search
- YAML config parsing for Hermes `config.yaml`
- SSE streaming parser for real-time chat responses
- SSH tunnel management via system SSH
- Attachment staging for file uploads
- Auto-update system with download progress tracking

## Notes

- The desktop app depends on the upstream Hermes Agent project for agent behavior and tool execution.
- The built-in installer runs the official Hermes install script with `--skip-setup`, then completes provider configuration in the GUI.
- Local model providers do not require an API key, but the compatible server must already be running.
- Alternative npm registry routes are supported for environments with restricted network access.
- SSH tunnel mode requires passwordless SSH access to the remote server (via SSH keys).
- Remote-only mode hides local-specific features (profiles, sessions, some settings) and shows a notice.

## Contributing

Contributions are welcome! Check out the [Contributing Guide](CONTRIBUTING.md) to get started. If you're not sure where to begin, take a look at the [open issues](https://github.com/NousResearch/hermes-desktop/issues). Found a bug or have a feature request? [File an issue](https://github.com/NousResearch/hermes-desktop/issues/new).

## Related Project

For the core agent, docs, and CLI workflows, see the main Hermes Agent repository:

- https://github.com/NousResearch/hermes-agent
