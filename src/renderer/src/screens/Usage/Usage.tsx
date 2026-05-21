import { useState, useEffect, useCallback } from "react";
import { useI18n } from "@renderer/components/useI18n";
import { hermesAPI } from "@shared/hermes-api";
import { BarChart3 } from "lucide-react";

export default function Usage(): React.JSX.Element {
  const { t } = useI18n();
  const [stats, setStats] = useState<{ totalSessions: number; totalMessages: number; activeSkills: number; memoryEntries: number } | null>(null);
  const [insights, setInsights] = useState("");
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    try { setStats(await hermesAPI.getUsageStats()); } catch {}
    try { setInsights(await hermesAPI.getInsights()); } catch {}
    setLoading(false);
  }, []);

  useEffect(() => { void load(); }, [load]);

  if (loading) return <div className="settings-container"><h1 className="settings-header">{t("navigation.usage")}</h1><div className="loading-spinner" /></div>;

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
          <div className="models-card" style={{ padding: 20, textAlign: "center" }}>
            <div style={{ fontSize: 32, fontWeight: 700, color: "var(--accent)" }}>{stats.totalSessions.toLocaleString()}</div>
            <div style={{ fontSize: 13, color: "var(--text-muted)" }}>{t("usage.sessions")}</div>
          </div>
          <div className="models-card" style={{ padding: 20, textAlign: "center" }}>
            <div style={{ fontSize: 32, fontWeight: 700, color: "var(--accent)" }}>{stats.totalMessages.toLocaleString()}</div>
            <div style={{ fontSize: 13, color: "var(--text-muted)" }}>{t("usage.messages")}</div>
          </div>
          <div className="models-card" style={{ padding: 20, textAlign: "center" }}>
            <div style={{ fontSize: 32, fontWeight: 700, color: "var(--accent)" }}>{stats.activeSkills}</div>
            <div style={{ fontSize: 13, color: "var(--text-muted)" }}>{t("usage.skills")}</div>
          </div>
          <div className="models-card" style={{ padding: 20, textAlign: "center" }}>
            <div style={{ fontSize: 32, fontWeight: 700, color: "var(--accent)" }}>{stats.memoryEntries}</div>
            <div style={{ fontSize: 13, color: "var(--text-muted)" }}>{t("usage.memories")}</div>
          </div>
        </div>
      )}

      {insights && (
        <div className="models-card" style={{ padding: 16, marginTop: 16, whiteSpace: "pre-wrap", fontFamily: "monospace", fontSize: 12, maxHeight: 400, overflow: "auto" }}>
          {insights}
        </div>
      )}
    </div>
  );
}
