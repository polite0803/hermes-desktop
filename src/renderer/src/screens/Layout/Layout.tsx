import { useState, useCallback, useEffect } from "react";
import Chat, { ChatMessage } from "../Chat/Chat";
import Sessions from "../Sessions/Sessions";
import Agents from "../Agents/Agents";
import Settings from "../Settings/Settings";
import Skills from "../Skills/Skills";
import Soul from "../Soul/Soul";
import Memory from "../Memory/Memory";
import Tools from "../Tools/Tools";
import Gateway from "../Gateway/Gateway";
import Office from "../Office/Office";
import Models from "../Models/Models";
import Providers from "../Providers/Providers";
import Schedules from "../Schedules/Schedules";
import Kanban from "../Kanban/Kanban";
import McpServers from "../McpServers/McpServers";
import Plugins from "../Plugins/Plugins";
import Usage from "../Usage/Usage";
import ContextFiles from "../ContextFiles/ContextFiles";

import Curator from "../Curator/Curator";
import RemoteNotice from "../../components/RemoteNotice";
import VerifyWarningBanner from "../../components/VerifyWarningBanner";
import { TabPage } from "../../components/TabPage";
import hermeslogo from "../../assets/hermes.png";
import {
  ChatBubble,
  Clock,
  Users,
  Settings as SettingsIcon,
  Sparkles,
  Building,
  Kanban as KanbanIcon,
  Download,
  PanelLeftClose,
  PanelLeftOpen,
} from "../../assets/icons";
import type { LucideIcon } from "lucide-react";
import { useI18n } from "../../components/useI18n";
import { hermesAPI } from "@shared/hermes-api";

type View =
  | "chat"
  | "sessions"
  | "profiles"
  | "aiStudio"
  | "workspace"
  | "infra"
  | "settings";

const NAV_ITEMS: { view: View; icon: LucideIcon; labelKey: string }[] = [
  { view: "chat", icon: ChatBubble, labelKey: "navigation.chat" },
  { view: "sessions", icon: Clock, labelKey: "navigation.sessions" },
  { view: "profiles", icon: Users, labelKey: "navigation.profiles" },
  { view: "aiStudio", icon: Sparkles, labelKey: "navigation.aiStudio" },
  { view: "workspace", icon: Building, labelKey: "navigation.workspace" },
  { view: "infra", icon: KanbanIcon, labelKey: "navigation.infra" },
  { view: "settings", icon: SettingsIcon, labelKey: "navigation.settings" },
];

interface LayoutProps {
  verifyWarning?: boolean;
  onReinstall?: () => void;
  onDismissVerifyWarning?: () => void;
}

function Layout({
  verifyWarning,
  onReinstall,
  onDismissVerifyWarning,
}: LayoutProps = {}): React.JSX.Element {
  const { t } = useI18n();
  const [collapsed, setCollapsed] = useState(false);
  const [view, setView] = useState<View>("chat");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
  const [activeProfile, setActiveProfile] = useState("default");
  const [visitedViews, setVisitedViews] = useState<Set<View>>(
    () => new Set<View>(["chat"]),
  );
  const [remoteMode, setRemoteMode] = useState(false);

  // Auto-update state
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);
  const [updateState, setUpdateState] = useState<
    "available" | "downloading" | "ready" | "error" | null
  >(null);
  const [downloadPercent, setDownloadPercent] = useState(0);
  const [updateError, setUpdateError] = useState<string | null>(null);

  const paneStyle = (target: View): React.CSSProperties => ({
    display: view === target ? "flex" : "none",
    flex: 1,
    flexDirection: "column",
    overflow: "hidden",
  });

  const goTo = useCallback((v: View) => {
    setVisitedViews((prev) => (prev.has(v) ? prev : new Set(prev).add(v)));
    setView(v);
  }, []);

  useEffect(() => {
    hermesAPI
      .isRemoteOnlyMode()
      .then(setRemoteMode)
      .catch(() => {});
  }, []);

  useEffect(() => {
    const a = hermesAPI.onUpdateAvailable((info) => {
      setUpdateVersion(info.version);
      setUpdateState("available");
      setUpdateError(null);
      setDownloadPercent(0);
    });
    const b = hermesAPI.onUpdateDownloadProgress((info) => {
      setDownloadPercent(info.percent);
    });
    const c = hermesAPI.onUpdateDownloaded(() => {
      setUpdateState("ready");
      setUpdateError(null);
    });
    const d = hermesAPI.onUpdateError((m) => {
      setUpdateState("error");
      setUpdateError(m);
      setDownloadPercent(0);
    });
    return () => {
      a();
      b();
      c();
      d();
    };
  }, []);

  // Menu events
  useEffect(() => {
    const a = hermesAPI.onMenuNewChat(() => {
      hermesAPI.abortChat();
      setMessages([]);
      setCurrentSessionId(null);
      goTo("chat");
    });
    const b = hermesAPI.onMenuSearchSessions(() => goTo("sessions"));
    return () => {
      a();
      b();
    };
  }, [goTo]);

  async function handleUpdate(): Promise<void> {
    if (updateState === "available" || updateState === "error") {
      setUpdateError(null);
      setDownloadPercent(0);
      setUpdateState("downloading");
      try {
        const ok = await hermesAPI.downloadUpdate();
        if (!ok) setUpdateState("error");
      } catch (e) {
        setUpdateError(e instanceof Error ? e.message : String(e));
        setUpdateState("error");
      }
    } else if (updateState === "ready") {
      await hermesAPI.installUpdate();
    }
  }

  const handleResumeSession = useCallback(
    async (sessionId: string) => {
      const db = await hermesAPI.getSessionMessages(sessionId);
      const msgs: ChatMessage[] = db.map((m) => ({
        id: `db-${m.id}`,
        role: m.role === "user" ? "user" : "agent",
        content: m.content,
        ...(m.attachments?.length ? { attachments: m.attachments } : {}),
      }));
      setMessages(msgs);
      setCurrentSessionId(sessionId);
      goTo("chat");
    },
    [goTo],
  );

  return (
    <div className="layout">
      <aside className={`sidebar ${collapsed ? "sidebar-collapsed" : ""}`}>
        <div className="sidebar-brand">
          {collapsed ? (
            <div className="sidebar-brand-icon">H</div>
          ) : (
            <img src={hermeslogo} height={30} alt="" />
          )}
          <button
            className="sidebar-toggle"
            onClick={() => setCollapsed((c) => !c)}
            title={
              collapsed ? t("navigation.expand") : t("navigation.collapse")
            }
          >
            {collapsed ? (
              <PanelLeftOpen size={14} />
            ) : (
              <PanelLeftClose size={14} />
            )}
          </button>
        </div>
        <nav className="sidebar-nav">
          {NAV_ITEMS.map(({ view: v, icon: Icon, labelKey }) => (
            <button
              key={v}
              className={`sidebar-nav-item ${view === v ? "active" : ""}`}
              onClick={() => goTo(v)}
              title={collapsed ? t(labelKey) : undefined}
            >
              <Icon size={16} />
              <span className="sidebar-label">{t(labelKey)}</span>
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          {updateState && (
            <button
              className={`sidebar-update-btn ${updateState === "error" ? "error" : ""}`}
              onClick={handleUpdate}
              disabled={updateState === "downloading"}
              title={updateError ?? undefined}
            >
              <Download size={13} />
              {updateState === "available" && (
                <span>
                  {t("common.updateAvailable", { version: updateVersion })}
                </span>
              )}
              {updateState === "downloading" && (
                <span>
                  {t("common.downloading", { percent: downloadPercent })}
                </span>
              )}
              {updateState === "ready" && (
                <span>{t("common.restartToUpdate")}</span>
              )}
              {updateState === "error" && (
                <span>{t("common.updateFailed")}</span>
              )}
            </button>
          )}
          <div className="sidebar-footer-text">
            {activeProfile === "default" ? t("common.appName") : activeProfile}
          </div>
        </div>
      </aside>

      <main className="content">
        {verifyWarning && onReinstall && onDismissVerifyWarning && (
          <VerifyWarningBanner
            onReinstall={onReinstall}
            onDismiss={onDismissVerifyWarning}
          />
        )}

        <div style={paneStyle("chat")}>
          <Chat
            messages={messages}
            setMessages={setMessages}
            sessionId={currentSessionId}
            profile={activeProfile}
            onNewChat={() => {
              hermesAPI.abortChat();
              setMessages([]);
              setCurrentSessionId(null);
            }}
          />
        </div>

        {visitedViews.has("sessions") && (
          <div style={paneStyle("sessions")}>
            {remoteMode ? (
              <RemoteNotice feature={t("navigation.sessions")} />
            ) : (
              <Sessions
                onResumeSession={handleResumeSession}
                onNewChat={() => {
                  hermesAPI.abortChat();
                  setMessages([]);
                  setCurrentSessionId(null);
                  goTo("chat");
                }}
                currentSessionId={currentSessionId}
                visible={view === "sessions"}
              />
            )}
          </div>
        )}

        {visitedViews.has("profiles") && (
          <div style={paneStyle("profiles")}>
            {remoteMode ? (
              <RemoteNotice feature={t("navigation.profiles")} />
            ) : (
              <Agents
                activeProfile={activeProfile}
                onSelectProfile={(n) => {
                  setActiveProfile(n);
                  setMessages([]);
                  setCurrentSessionId(null);
                }}
                onChatWith={(n) => {
                  setActiveProfile(n);
                  goTo("chat");
                }}
              />
            )}
          </div>
        )}

        {visitedViews.has("aiStudio") && (
          <div style={paneStyle("aiStudio")}>
            <TabPage
              tabs={[
                { key: "models", label: t("navigation.models") },
                { key: "providers", label: t("navigation.providers") },
                { key: "skills", label: t("navigation.skills") },
                { key: "persona", label: t("navigation.persona") },
                { key: "tools", label: t("navigation.tools") },
                { key: "memory", label: t("navigation.memory") },
                { key: "context", label: t("navigation.contextFiles") },
              ]}
            >
              {(tab) => {
                switch (tab) {
                  case "models":
                    return <Models visible />;
                  case "providers":
                    return remoteMode ? (
                      <RemoteNotice feature={t("navigation.providers")} />
                    ) : (
                      <Providers profile={activeProfile} visible />
                    );
                  case "skills":
                    return remoteMode ? (
                      <RemoteNotice feature={t("navigation.skills")} />
                    ) : (
                      <Skills profile={activeProfile} />
                    );
                  case "persona":
                    return remoteMode ? (
                      <RemoteNotice feature={t("navigation.persona")} />
                    ) : (
                      <Soul profile={activeProfile} />
                    );
                  case "tools":
                    return remoteMode ? (
                      <RemoteNotice feature={t("navigation.tools")} />
                    ) : (
                      <Tools profile={activeProfile} />
                    );
                  case "memory":
                    return remoteMode ? (
                      <RemoteNotice feature={t("navigation.memory")} />
                    ) : (
                      <Memory profile={activeProfile} />
                    );
                  case "context":
                    return <ContextFiles />;
                  default:
                    return null;
                }
              }}
            </TabPage>
          </div>
        )}

        {visitedViews.has("workspace") && (
          <div style={paneStyle("workspace")}>
            <TabPage
              tabs={[
                { key: "kanban", label: t("navigation.kanban") },
                { key: "office", label: t("navigation.office") },
              ]}
            >
              {(tab) => {
                switch (tab) {
                  case "kanban":
                    return remoteMode ? (
                      <RemoteNotice feature={t("navigation.kanban")} />
                    ) : (
                      <Kanban profile={activeProfile} visible />
                    );
                  case "office":
                    return <Office profile={activeProfile} visible />;
                  default:
                    return null;
                }
              }}
            </TabPage>
          </div>
        )}

        {visitedViews.has("infra") && (
          <div style={paneStyle("infra")}>
            <TabPage
              tabs={[
                { key: "gateway", label: t("navigation.gateway") },
                { key: "mcp", label: t("navigation.mcpServers") },
                { key: "plugins", label: t("navigation.plugins") },
                { key: "curator", label: t("navigation.curator") },
                { key: "schedules", label: t("navigation.schedules") },
              ]}
            >
              {(tab) => {
                switch (tab) {
                  case "gateway":
                    return <Gateway profile={activeProfile} />;
                  case "mcp":
                    return <McpServers />;
                  case "plugins":
                    return <Plugins />;
                  case "curator":
                    return <Curator />;
                  case "schedules":
                    return <Schedules profile={activeProfile} />;
                  default:
                    return null;
                }
              }}
            </TabPage>
          </div>
        )}

        {visitedViews.has("settings") && (
          <div style={paneStyle("settings")}>
            <TabPage
              tabs={[
                { key: "general", label: t("navigation.settings") },
                { key: "usage", label: t("navigation.usage") },
              ]}
            >
              {(tab) => {
                switch (tab) {
                  case "general":
                    return <Settings profile={activeProfile} />;
                  case "usage":
                    return <Usage />;
                  default:
                    return null;
                }
              }}
            </TabPage>
          </div>
        )}
      </main>
    </div>
  );
}

export default Layout;
