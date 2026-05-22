/**
 * Hermes API Adapter — matches the original Electron preload hermesAPI surface.
 * Implementation uses Tauri invoke() / listen() under the hood.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── helpers ──────────────────────────────────────────────

/** Fire-and-forget event listener that returns a synchronous cleanup fn. */
function onEvent<T>(event: string, callback: (payload: T) => void): () => void {
  let unlisten: UnlistenFn | null = null;
  let cancelled = false;
  listen<T>(event, (e) => {
    if (!cancelled) callback(e.payload);
  }).then((fn) => {
    if (cancelled) fn();
    else unlisten = fn;
  });
  return () => {
    cancelled = true;
    unlisten?.();
  };
}

// ── types ────────────────────────────────────────────────

export interface InstallStatus {
  installed: boolean;
  configured: boolean;
  hasApiKey: boolean;
}

export interface InstallProgress {
  step: number;
  totalSteps: number;
  titleKey: string;
  detailKey: string;
  log: string;
}

export interface ConnectionConfig {
  mode: "local" | "remote" | "ssh";
  remoteUrl: string;
  hasApiKey: boolean;
  ssh: {
    host: string;
    port: number;
    username: string;
    keyPath: string;
    remotePort: number;
    localPort: number;
  };
}

export interface ModelConfig {
  provider: string;
  model: string;
  baseUrl: string;
}

export interface SessionSummary {
  id: string;
  source: string;
  startedAt: number;
  endedAt: number | null;
  messageCount: number;
  model: string;
  title: string | null;
  preview: string;
}

export interface SessionMessage {
  id: number;
  role: "user" | "assistant";
  content: string;
  timestamp: number;
  attachments?: unknown[];
}

export interface CachedSession {
  id: string;
  title: string;
  startedAt: number;
  source: string;
  messageCount: number;
  model: string;
}

export interface SearchResult {
  sessionId: string;
  title: string | null;
  startedAt: number;
  source: string;
  messageCount: number;
  model: string;
  snippet: string;
}

export interface Profile {
  name: string;
  path: string;
  isDefault: boolean;
  isActive: boolean;
  model: string;
  provider: string;
  hasEnv: boolean;
  hasSoul: boolean;
  skillCount: number;
  gatewayRunning: boolean;
}

export interface MemoryInfo {
  memory: { content: string; exists: boolean; lastModified: number | null };
  user: { content: string; exists: boolean; lastModified: number | null };
  stats: { totalSessions: number; totalMessages: number };
}

export interface ToolsetInfo {
  key: string;
  label: string;
  description: string;
  enabled: boolean;
}

export interface InstalledSkill {
  name: string;
  category: string;
  description: string;
  path: string;
}

export interface BundledSkill {
  name: string;
  description: string;
  category: string;
  source: string;
  installed: boolean;
}

export interface SavedModel {
  id: string;
  name: string;
  provider: string;
  model: string;
  baseUrl: string;
  createdAt: number;
}

export interface CronJob {
  id: string;
  name: string;
  schedule: string;
  prompt: string;
  state: "active" | "paused" | "completed";
  enabled: boolean;
  next_run_at: string | null;
  last_run_at: string | null;
  last_status: string | null;
  last_error: string | null;
  repeat: { times: number | null; completed: number } | null;
  deliver: string[];
  skills: string[];
  script: string | null;
}

export interface Claw3dStatus {
  cloned: boolean;
  installed: boolean;
  devServerRunning: boolean;
  adapterRunning: boolean;
  port: number;
  portInUse: boolean;
  wsUrl: string;
  running: boolean;
  error: string;
  remoteUrl?: string | null;
  remoteSource?: "ssh" | null;
}

export interface ChatMessage {
  role: string;
  content: string;
}

export interface MemoryProvider {
  name: string;
  description: string;
  installed: boolean;
  active: boolean;
  envVars: string[];
}

export interface McpServer {
  name: string;
  command: string;
  args: string[];
  enabled: boolean;
}

// ══════════════════════════════════════════════════════════
// hermesAPI — exact preload surface
// ══════════════════════════════════════════════════════════

export const hermesAPI = {
  // ── Installation ──────────────────────────────────────
  checkInstall: (): Promise<InstallStatus> => invoke("check_install"),
  verifyInstall: (): Promise<boolean> => invoke("verify_install"),
  startInstall: (): Promise<{ success: boolean; error?: string }> =>
    invoke("start_install"),
  startPypiInstall: (): Promise<{ success: boolean; error?: string }> =>
    invoke("start_pypi_install"),
  onInstallProgress: (cb: (p: InstallProgress) => void): (() => void) =>
    onEvent("install-progress", cb),

  // ── Hermes engine ─────────────────────────────────────
  getHermesVersion: (): Promise<string | null> => invoke("get_hermes_version"),
  refreshHermesVersion: (): Promise<string | null> => invoke("refresh_hermes_version"),
  runHermesDoctor: (): Promise<string> => invoke("run_hermes_doctor"),
  runHermesUpdate: (): Promise<{ success: boolean; error?: string }> =>
    invoke("run_hermes_update"),

  // ── OpenClaw migration ────────────────────────────────
  checkOpenClaw: (): Promise<{ found: boolean; path: string | null }> =>
    invoke("check_openclaw"),
  runClawMigrate: (): Promise<{ success: boolean; error?: string }> =>
    invoke("run_hermes_migrate"),

  // ── Locale ────────────────────────────────────────────
  getLocale: (): Promise<string> => invoke("get_locale"),
  setLocale: (locale: string): Promise<string> =>
    invoke("set_locale", { locale }),

  // ── Config ────────────────────────────────────────────
  getEnv: (profile?: string): Promise<Record<string, string>> =>
    invoke("get_env_all", { profile }),
  setEnv: (key: string, value: string, profile?: string): Promise<boolean> =>
    invoke("set_env", { key, value, profile }),
  getConfig: (key: string, profile?: string): Promise<string | null> =>
    invoke("get_config_value", { key, profile }),
  setConfig: (key: string, value: string, profile?: string): Promise<boolean> =>
    invoke("set_config_value", { key, value, profile }),
  getHermesHome: (profile?: string): Promise<string> =>
    invoke("get_hermes_home"),
  getModelConfig: (profile?: string): Promise<ModelConfig> =>
    invoke("get_model_config", { profile }),
  setModelConfig: (provider: string, model: string, baseUrl: string, profile?: string): Promise<boolean> =>
    invoke("set_model_config", { provider, model, baseUrl, profile }),

  // ── Connection ────────────────────────────────────────
  isRemoteMode: (): Promise<boolean> => invoke("is_remote_mode"),
  isRemoteOnlyMode: (): Promise<boolean> => invoke("is_remote_only_mode"),
  getConnectionConfig: (): Promise<ConnectionConfig> =>
    invoke("get_connection_config"),
  setConnectionConfig: (mode: string, remoteUrl: string, apiKey?: string): Promise<boolean> =>
    invoke("set_connection_config", { mode, remoteUrl, apiKey }),
  setSshConfig: (host: string, port: number, username: string, keyPath: string, remotePort: number, localPort: number): Promise<boolean> =>
    invoke("set_connection_config", {
      config: { mode: "ssh", ssh: { host, port, username, keyPath, remotePort, localPort } }
    }).then(() => true),
  testRemoteConnection: (url: string, apiKey?: string): Promise<boolean> =>
    invoke("test_remote_connection", { url, apiKey }),
  testSshConnection: (host: string, port: number, username: string, keyPath: string, remotePort: number): Promise<boolean> =>
    invoke("test_ssh_connection"),
  isSshTunnelActive: (): Promise<boolean> => invoke("is_ssh_tunnel_active"),
  startSshTunnel: (): Promise<boolean> => invoke("start_ssh_tunnel").then(() => true),
  stopSshTunnel: (): Promise<boolean> => invoke("stop_ssh_tunnel").then(() => true),

  // ── Chat ──────────────────────────────────────────────
  sendMessage: (
    message: string,
    profile?: string,
    resumeSessionId?: string,
    history?: ChatMessage[],
    attachments?: unknown[],
  ): Promise<{ response: string; sessionId?: string }> =>
    invoke("send_message", { message, profile, resumeSessionId, history, attachments }),
  abortChat: (): Promise<void> => invoke("abort_chat"),

  getPathForFile: (file: File): string => {
    // Tauri v2: File objects from drag-drop include a `path` property
    try {
      const f = file as unknown as { path?: string; name: string };
      if (f.path) return f.path;
      // Fallback for files without an origin path (e.g. clipboard paste)
      return "";
    } catch {
      return "";
    }
  },

  // ── System info (replaces Electron's process.*) ──────
  getSystemInfo: (): Promise<{ platform: string; arch: string; appVersion: string }> =>
    invoke("get_system_info"),

  stageAttachment: (
    sessionId: string,
    filename: string,
    base64Bytes: string,
  ): Promise<string> =>
    invoke("stage_attachment", { sessionId, filename, kind: "file", mime: "application/octet-stream", base64Content: base64Bytes }),

  clearStagedAttachments: (sessionId: string): Promise<void> =>
    invoke("clear_staged_attachments", { sessionId }),

  discoverProviderModels: (
    provider: string,
    baseUrl?: string,
    apiKey?: string,
    profile?: string,
  ): Promise<{ models: string[]; status: "ok" | "no-key" | "unsupported" | "unknown-host"; cached: boolean }> =>
    invoke("discover_provider_models", { provider, baseUrl, apiKey, profile }),

  // ── Chat events ───────────────────────────────────────
  onChatChunk: (callback: (chunk: string) => void): (() => void) =>
    onEvent("chat-chunk", callback),
  onChatDone: (callback: (sessionId?: string) => void): (() => void) =>
    onEvent("chat-done", callback),
  onChatToolProgress: (callback: (tool: string) => void): (() => void) =>
    onEvent("chat-tool-progress", callback),
  onChatUsage: (callback: (usage: { promptTokens: number; completionTokens: number; totalTokens: number; cost?: number; rateLimitRemaining?: number; rateLimitReset?: number }) => void): (() => void) =>
    onEvent("chat-usage", callback),
  onChatError: (callback: (error: string) => void): (() => void) =>
    onEvent("chat-error", callback),

  // ── Gateway ───────────────────────────────────────────
  startGateway: (): Promise<boolean> => invoke("start_gateway").then(() => true),
  stopGateway: (): Promise<boolean> => invoke("stop_gateway").then(() => true),
  gatewayStatus: (): Promise<boolean> => invoke("gateway_status").then((s: string) => s === "running"),

  // ── Platform toggles ──────────────────────────────────
  getPlatformEnabled: (profile?: string): Promise<Record<string, boolean>> =>
    invoke("get_platform_enabled_all", { profile }),
  setPlatformEnabled: (platform: string, enabled: boolean, profile?: string): Promise<boolean> =>
    invoke("set_platform_enabled", { platform, enabled, profile }).then(() => true),

  // ── Sessions ──────────────────────────────────────────
  listSessions: (limit?: number, offset?: number): Promise<SessionSummary[]> =>
    invoke("list_sessions", { limit, offset }),
  getSessionMessages: (sessionId: string): Promise<SessionMessage[]> =>
    invoke("get_session_messages", { sessionId }),
  listCachedSessions: (limit?: number, offset?: number): Promise<CachedSession[]> =>
    invoke("list_cached_sessions", { limit, offset }),
  syncSessionCache: (): Promise<CachedSession[]> => invoke("sync_session_cache"),
  updateSessionTitle: (sessionId: string, title: string): Promise<void> =>
    invoke("update_session_title", { sessionId, title }),
  deleteSession: (sessionId: string): Promise<void> =>
    invoke("delete_session", { sessionId }),
  searchSessions: (query: string, limit?: number): Promise<SearchResult[]> =>
    invoke("search_sessions", { query, limit }),

  // ── Profiles ──────────────────────────────────────────
  listProfiles: (): Promise<Profile[]> => invoke("list_profiles"),
  createProfile: (name: string, clone: boolean): Promise<{ success: boolean; error?: string }> =>
    invoke("create_profile", { name, clone }),
  deleteProfile: (name: string): Promise<{ success: boolean; error?: string }> =>
    invoke("delete_profile", { name }),
  setActiveProfile: (name: string): Promise<boolean> =>
    invoke("set_active_profile", { name }).then(() => true),

  // ── Memory ────────────────────────────────────────────
  readMemory: (profile?: string): Promise<MemoryInfo> =>
    invoke("read_memory", { profile }),
  addMemoryEntry: (content: string, profile?: string): Promise<{ success: boolean; error?: string; errorKey?: string }> =>
    invoke("add_memory_entry", { content, profile }),
  updateMemoryEntry: (index: number, content: string, profile?: string): Promise<{ success: boolean; error?: string; errorKey?: string }> =>
    invoke("update_memory_entry", { index, content, profile }),
  removeMemoryEntry: (index: number, profile?: string): Promise<boolean> =>
    invoke("remove_memory_entry", { index, profile }),
  writeUserProfile: (content: string, profile?: string): Promise<{ success: boolean; error?: string; errorKey?: string }> =>
    invoke("write_user_profile", { content, profile }),

  // ── Soul ──────────────────────────────────────────────
  readSoul: (profile?: string): Promise<string> =>
    invoke("read_soul", { profile }),
  writeSoul: (content: string, profile?: string): Promise<boolean> =>
    invoke("write_soul", { content, profile }).then(() => true),
  resetSoul: (profile?: string): Promise<string> =>
    invoke("reset_soul", { profile }),

  // ── Toolsets ──────────────────────────────────────────
  getToolsets: (profile?: string): Promise<ToolsetInfo[]> =>
    invoke("get_toolsets", { profile }),
  setToolsetEnabled: (key: string, enabled: boolean, profile?: string): Promise<boolean> =>
    invoke("set_toolset_enabled", { name: key, enabled, profile }).then(() => true),

  // ── Skills ────────────────────────────────────────────
  listInstalledSkills: (profile?: string): Promise<InstalledSkill[]> =>
    invoke("list_installed_skills", { profile }),
  listBundledSkills: (): Promise<BundledSkill[]> =>
    invoke("list_bundled_skills"),
  getSkillContent: (skillPath: string): Promise<string> =>
    invoke("get_skill_content", { path: skillPath }),
  installSkill: (identifier: string, profile?: string): Promise<{ success: boolean; error?: string }> =>
    invoke("install_skill", { name: identifier, profile }),
  uninstallSkill: (name: string, profile?: string): Promise<{ success: boolean; error?: string }> =>
    invoke("uninstall_skill", { name, profile }),

  // ── Models ────────────────────────────────────────────
  listModels: (): Promise<SavedModel[]> => invoke("list_models"),
  addModel: (name: string, provider: string, model: string, baseUrl: string): Promise<SavedModel> =>
    invoke("add_model", { name, provider, model, baseUrl }),
  removeModel: (id: string): Promise<boolean> =>
    invoke("remove_model", { id }).then(() => true),
  updateModel: (id: string, fields: Record<string, string>): Promise<boolean> =>
    invoke("update_model", { id, ...fields }).then(() => true),

  // ── Claw3D ────────────────────────────────────────────
  claw3dStatus: (): Promise<Claw3dStatus> => invoke("claw3d_status"),
  claw3dSetup: (): Promise<{ success: boolean; error?: string }> =>
    invoke("claw3d_setup"),
  onClaw3dSetupProgress: (callback: (p: InstallProgress) => void): (() => void) =>
    onEvent("claw3d-setup-progress", callback),
  claw3dGetPort: (): Promise<number> => invoke("claw3d_get_port"),
  claw3dSetPort: (port: number): Promise<boolean> =>
    invoke("claw3d_set_port", { port }).then(() => true),
  claw3dGetWsUrl: (): Promise<string> => invoke("claw3d_get_ws_url"),
  claw3dSetWsUrl: (url: string): Promise<boolean> =>
    invoke("claw3d_set_ws_url", { url }).then(() => true),
  claw3dStartAll: (profile?: string): Promise<{ success: boolean; error?: string }> =>
    invoke("claw3d_start_all", { profile }),
  claw3dStopAll: (): Promise<boolean> =>
    invoke("claw3d_stop_all").then(() => true),
  claw3dGetLogs: (): Promise<string> => invoke("claw3d_get_logs"),
  claw3dStartDev: (): Promise<boolean> =>
    invoke("claw3d_start_dev").then(() => true),
  claw3dStopDev: (): Promise<boolean> =>
    invoke("claw3d_stop_dev").then(() => true),
  claw3dStartAdapter: (): Promise<boolean> =>
    invoke("claw3d_start_adapter").then(() => true),
  claw3dStopAdapter: (): Promise<boolean> =>
    invoke("claw3d_stop_adapter").then(() => true),

  // ── Updates ───────────────────────────────────────────
  checkForUpdates: (): Promise<string | null> =>
    invoke("check_for_updates"),
  downloadUpdate: (): Promise<boolean> =>
    invoke("download_update"),
  installUpdate: (): Promise<void> =>
    invoke("install_update"),
  getAppVersion: (): Promise<string> => invoke("get_app_version"),

  onUpdateAvailable: (callback: (info: { version: string; releaseNotes: string }) => void): (() => void) =>
    onEvent("update-available", callback),
  onUpdateDownloadProgress: (callback: (info: { percent: number }) => void): (() => void) =>
    onEvent("update-download-progress", callback),
  onUpdateDownloaded: (callback: () => void): (() => void) =>
    onEvent("update-downloaded", callback),
  onUpdateError: (callback: (message: string) => void): (() => void) =>
    onEvent("update-error", callback),

  // ── Menu events ───────────────────────────────────────
  onMenuNewChat: (callback: () => void): (() => void) =>
    onEvent("menu-new-chat", callback),
  onMenuSearchSessions: (callback: () => void): (() => void) =>
    onEvent("menu-search-sessions", callback),

  // ── Cron Jobs ─────────────────────────────────────────
  listCronJobs: (includeDisabled?: boolean, profile?: string): Promise<CronJob[]> =>
    invoke("list_cron_jobs", { includeDisabled, profile }),
  createCronJob: (schedule: string, prompt?: string, name?: string, deliver?: string, profile?: string): Promise<{ success: boolean; error?: string }> =>
    invoke("create_cron_job", { name, schedule, prompt, deliver, profile }),
  removeCronJob: (jobId: string, profile?: string): Promise<{ success: boolean; error?: string }> =>
    invoke("remove_cron_job", { id: jobId, profile }),
  pauseCronJob: (jobId: string, profile?: string): Promise<{ success: boolean; error?: string }> =>
    invoke("pause_cron_job", { id: jobId, profile }),
  resumeCronJob: (jobId: string, profile?: string): Promise<{ success: boolean; error?: string }> =>
    invoke("resume_cron_job", { id: jobId, profile }),
  triggerCronJob: (jobId: string, profile?: string): Promise<{ success: boolean; error?: string }> =>
    invoke("trigger_cron_job", { id: jobId, profile }),

  // ── Kanban ────────────────────────────────────────────
  kanbanListBoards: (includeArchived?: boolean, profile?: string): Promise<{ success: boolean; data?: unknown[]; error?: string }> =>
    invoke("kanban_list_boards", { includeArchived, profile }),
  kanbanCurrentBoard: (profile?: string): Promise<unknown> =>
    invoke("kanban_current_board", { profile }),
  kanbanSwitchBoard: (slug: string, profile?: string): Promise<unknown> =>
    invoke("kanban_switch_board", { slug, profile }),
  kanbanCreateBoard: (slug: string, name?: string, switchAfter?: boolean, profile?: string): Promise<unknown> =>
    invoke("kanban_create_board", { slug, name, switchAfter, profile }),
  kanbanRemoveBoard: (slug: string, hardDelete?: boolean, profile?: string): Promise<unknown> =>
    invoke("kanban_remove_board", { slug, hardDelete, profile }),
  kanbanListTasks: (filters?: { status?: string; assignee?: string; tenant?: string; includeArchived?: boolean; profile?: string }): Promise<{ success: boolean; data?: unknown[]; error?: string }> =>
    invoke("kanban_list_tasks", { filters }),
  kanbanGetTask: (taskId: string, profile?: string): Promise<{ success: boolean; data?: unknown; error?: string }> =>
    invoke("kanban_get_task", { taskId, profile }),
  kanbanCreateTask: (input: { title: string; body?: string; assignee?: string; priority?: number; tenant?: string; workspace?: string; triage?: boolean; skills?: string[]; maxRetries?: number }, profile?: string): Promise<{ success: boolean; data?: unknown; error?: string }> =>
    invoke("kanban_create_task", { input, profile }),
  selectFolder: (): Promise<string | null> => invoke("select_folder"),
  kanbanAssignTask: (taskId: string, assignee: string | null, profile?: string): Promise<unknown> =>
    invoke("kanban_assign_task", { taskId, assignee, profile }),
  kanbanCompleteTask: (taskId: string, result?: string, profile?: string): Promise<unknown> =>
    invoke("kanban_complete_task", { taskId, result, profile }),
  kanbanBlockTask: (taskId: string, reason?: string, profile?: string): Promise<unknown> =>
    invoke("kanban_block_task", { taskId, reason, profile }),
  kanbanUnblockTask: (taskId: string, profile?: string): Promise<unknown> =>
    invoke("kanban_unblock_task", { taskId, profile }),
  kanbanArchiveTask: (taskId: string, profile?: string): Promise<unknown> =>
    invoke("kanban_archive_task", { taskId, profile }),
  kanbanSpecifyTask: (taskId: string, profile?: string): Promise<unknown> =>
    invoke("kanban_specify_task", { taskId, profile }),
  kanbanReclaimTask: (taskId: string, reason?: string, profile?: string): Promise<unknown> =>
    invoke("kanban_reclaim_task", { taskId, reason, profile }),
  kanbanCommentTask: (taskId: string, body: string, profile?: string): Promise<unknown> =>
    invoke("kanban_comment_task", { taskId, body, profile }),
  kanbanDispatchOnce: (dryRun?: boolean, profile?: string): Promise<unknown> =>
    invoke("kanban_dispatch_once", { dryRun, profile }),

  // ── Shell / External ──────────────────────────────────
  openExternal: async (url: string): Promise<void> => {
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      await open(url);
    } catch {
      // Fallback: try invoke
      void invoke("open_external", { url });
    }
  },

  // ── Backup / Import ───────────────────────────────────
  runHermesBackup: (profile?: string): Promise<{ success: boolean; path?: string; error?: string }> =>
    invoke("run_hermes_backup", { profile }),
  runHermesImport: (archivePath: string, profile?: string): Promise<{ success: boolean; error?: string }> =>
    invoke("run_hermes_import", { path: archivePath, profile }),
  runHermesDump: (): Promise<string> => invoke("run_hermes_dump"),

  // ── Memory providers ──────────────────────────────────
  discoverMemoryProviders: (profile?: string): Promise<MemoryProvider[]> =>
    invoke("discover_memory_providers", { profile }),

  // ── MCP servers ───────────────────────────────────────
  // ── Skills Hub ───────────────────────────────────
  searchSkillsHub: (query: string): Promise<{ name: string; description: string; category: string; author: string; downloads: number; installed: boolean }[]> =>
    invoke("search_skills_hub", { query }),
  installFromHub: (name: string, profile?: string): Promise<{ success: boolean; error?: string }> =>
    invoke("install_from_hub", { name, profile }),

  // ── Personality Presets ───────────────────────────
  listPersonalities: (profile?: string): Promise<{ name: string; description: string }[]> =>
    invoke("list_personalities", { profile }),
  applyPersonality: (name: string, profile?: string): Promise<void> =>
    invoke("apply_personality", { name, profile }),

  // ── Plugins ──────────────────────────────────────
  listPlugins: (): Promise<{ name: string; description: string; installed: boolean; enabled: boolean }[]> =>
    invoke("list_plugins"),
  enablePlugin: (name: string): Promise<void> =>
    invoke("enable_plugin", { name }),
  disablePlugin: (name: string): Promise<void> =>
    invoke("disable_plugin", { name }),

  // ── Usage ─────────────────────────────────────────
  getUsageStats: (): Promise<{ totalSessions: number; totalMessages: number; activeSkills: number; memoryEntries: number }> =>
    invoke("get_usage_stats"),
  getInsights: (): Promise<string> =>
    invoke("get_insights"),

  // ── Context Files ─────────────────────────────────
  listContextFiles: (): Promise<string[]> =>
    invoke("list_context_files"),
  readContextFile: (name: string): Promise<{ name: string; content: string }> =>
    invoke("read_context_file", { name }),
  writeContextFile: (name: string, content: string): Promise<void> =>
    invoke("write_context_file", { name, content }),

  // ── MCP Servers ──────────────────────────────────
  listMcpServers: (): Promise<McpServer[]> =>
    invoke("list_mcp_servers"),
  addMcpServer: (name: string, command: string, args: string[]): Promise<McpServer> =>
    invoke("add_mcp_server", { name, command, args }),
  removeMcpServer: (name: string): Promise<void> =>
    invoke("remove_mcp_server", { name }),
  updateMcpServer: (name: string, updates: { command?: string; args?: string[]; enabled?: boolean }): Promise<McpServer> =>
    invoke("update_mcp_server", { name, command: updates.command, args: updates.args, enabled: updates.enabled }),
  testMcpServer: (name: string): Promise<boolean> =>
    invoke("test_mcp_server", { name }),
  installComputerUseMcp: (): Promise<boolean> =>
    invoke("install_computer_use_mcp"),

  // ── Curator ──────────────────────────────────────
  curatorStatus: (): Promise<string> => invoke("curator_status"),
  curatorTrigger: (): Promise<string> => invoke("curator_trigger"),
  curatorReport: (): Promise<string> => invoke("curator_report"),

  // ── Proxy ─────────────────────────────────────────
  startProxy: (): Promise<boolean> => invoke("start_proxy"),

  // ── HuggingFace Skills ────────────────────────────
  searchHuggingfaceSkills: (query: string): Promise<{ name: string; description: string; category: string; author: string; downloads: number; installed: boolean }[]> =>
    invoke("search_huggingface_skills", { query }),

  // ── Sandbox Backend ──────────────────────────────
  getTerminalBackend: (): Promise<string> =>
    invoke("get_terminal_backend"),
  setTerminalBackend: (backend: string, config?: Record<string, string>): Promise<boolean> =>
    invoke("set_terminal_backend", { backend, configJson: config ? JSON.stringify(config) : null }),

  // ── Log viewer ────────────────────────────────────────
  readLogs: (logFile?: string, lines?: number): Promise<{ content: string; path: string }> =>
    invoke("read_logs", { logFile, lines }),
};
