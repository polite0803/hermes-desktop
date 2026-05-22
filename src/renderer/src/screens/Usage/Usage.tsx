import { useState, useEffect, useCallback } from "react";
import { useI18n } from "@renderer/components/useI18n";
import { hermesAPI } from "@shared/hermes-api";
import { BarChart3, MessageSquare, BookOpen, Brain, Clock } from "lucide-react";

export default function Usage(): React.JSX.Element {
  const { t } = useI18n();
  const [stats, setStats] = useState<{ totalSessions: number; totalMessages: number; activeSkills: number; memoryEntries: number } | null>(null);
  const [insights, setInsights] = useState("");
  const [version, setVersion] = useState("");
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    try { setStats(await hermesAPI.getUsageStats()); } catch {}
    try { setInsights(await hermesAPI.getInsights()); } catch {}
    try { setVersion(await hermesAPI.getAppVersion() || ""); } catch {}
    setLoading(false);
  }, []);

  useEffect(() => { void load(); }, [load]);

  if (loading) return (
    <div className="settings-container">
      <h1 className="settings-header">{t("navigation.usage")}</h1>
      <div className="loading-spinner" />
    </div>
  );

  return (
    <div className="settings-container">
      <div className="models-header">
        <div>
          <h1 className="settings-header models-title-tight">{t("navigation.usage")}</h1>
          <p className="models-subtitle">{t("usage.subtitle")}</p>
        </div>
        <button className="btn btn-primary btn-sm" onClick={load}><BarChart3 size={14} /> {t("common.refresh")}</button>
      </div>

      {stats && (
        <div className="models-grid">
          <div className="models-card" style={{ padding: 20 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
              <MessageSquare size={18} style={{ color: "var(--accent)" }} />
              <span style={{ fontSize: 13, color: "var(--text-muted)" }}>{t("usage.totalSessions")}</span>
            </div>
            <div style={{ fontSize: 32, fontWeight: 700 }}>{stats.totalSessions.toLocaleString()}</div>
          </div>
          <div className="models-card" style={{ padding: 20 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
              <BarChart3 size={18} style={{ color: "var(--accent)" }} />
              <span style={{ fontSize: 13, color: "var(--text-muted)" }}>{t("usage.totalMessages")}</span>
            </div>
            <div style={{ fontSize: 32, fontWeight: 700 }}>{stats.totalMessages.toLocaleString()}</div>
            {stats.totalSessions > 0 && (
              <div style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 4 }}>
                {t("usage.avgPerSession", { avg: Math.round(stats.totalMessages / stats.totalSessions) })}
              </div>
            )}
          </div>
          <div className="models-card" style={{ padding: 20 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
              <BookOpen size={18} style={{ color: "var(--accent)" }} />
              <span style={{ fontSize: 13, color: "var(--text-muted)" }}>{t("usage.activeSkills")}</span>
            </div>
            <div style={{ fontSize: 32, fontWeight: 700 }}>{stats.activeSkills}</div>
          </div>
          <div className="models-card" style={{ padding: 20 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
              <Brain size={18} style={{ color: "var(--accent)" }} />
              <span style={{ fontSize: 13, color: "var(--text-muted)" }}>{t("usage.memoryEntries")}</span>
            </div>
            <div style={{ fontSize: 32, fontWeight: 700 }}>{stats.memoryEntries}</div>
          </div>
        </div>
      )}

      {version && (
        <div className="models-card" style={{ padding: "10px 16px", marginTop: 16, display: "flex", alignItems: "center", gap: 8 }}>
          <Clock size={14} style={{ color: "var(--text-muted)" }} />
          <span style={{ fontSize: 13, color: "var(--text-muted)" }}>{t("usage.desktopVersion")}: {version}</span>
        </div>
      )}

      {insights && (
        <div className="models-card" style={{ padding: 16, marginTop: 16 }}>
          <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 8 }}>{t("usage.insights")}</h3>
          <div style={{ whiteSpace: "pre-wrap", fontFamily: "monospace", fontSize: 12, maxHeight: 400, overflow: "auto", color: "var(--text-secondary)" }}>
            {insights}
          </div>
        </div>
      )}
    </div>
  );
}
