import { useState, useEffect, useCallback } from "react";
import { useI18n } from "@renderer/components/useI18n";
import { hermesAPI } from "@shared/hermes-api";

export default function Security(): React.JSX.Element {
  const { t } = useI18n();
  const [loading, setLoading] = useState(true);
  const [isRemote, setIsRemote] = useState(false);
  const [saved, setSaved] = useState(false);
  const [config, setConfig] = useState({ mode: "local", remoteUrl: "", apiKey: "" });

  const load = useCallback(async () => {
    try {
      const conn = await hermesAPI.getConnectionConfig();
      setConfig({ mode: conn.mode, remoteUrl: conn.remoteUrl, apiKey: "" });
      setIsRemote(conn.mode !== "local");
    } catch {}
    setLoading(false);
  }, []);

  useEffect(() => { void load(); }, [load]);

  async function save(): Promise<void> {
    await hermesAPI.setConnectionConfig(config.mode, config.remoteUrl, config.apiKey || undefined);
    setSaved(true); setTimeout(() => setSaved(false), 2000);
  }

  if (loading) return <div className="settings-container"><h1 className="settings-header">{t("navigation.security")}</h1><div className="loading-spinner" /></div>;

  return (
    <div className="settings-container">
      <div className="models-header">
        <div>
          <h1 className="settings-header models-title-tight">{t("navigation.security")}</h1>
          <p className="models-subtitle">{t("security.subtitle")}</p>
        </div>
        {saved && <span style={{ fontSize: 12, color: "var(--accent)" }}>✓ Saved</span>}
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 12, maxWidth: 500 }}>
        <div>
          <label style={{ fontSize: 13, fontWeight: 500, marginBottom: 4, display: "block" }}>Connection Mode</label>
          <select className="models-search-input" value={config.mode} onChange={(e) => setConfig({ ...config, mode: e.target.value })}>
            <option value="local">Local</option>
            <option value="remote">Remote</option>
            <option value="ssh">SSH</option>
          </select>
        </div>

        {config.mode !== "local" && (
          <div>
            <label style={{ fontSize: 13, fontWeight: 500, marginBottom: 4, display: "block" }}>Remote URL</label>
            <input className="models-search-input" value={config.remoteUrl} onChange={(e) => setConfig({ ...config, remoteUrl: e.target.value })} placeholder="http://host:8642" />
          </div>
        )}

        <div>
          <label style={{ fontSize: 13, fontWeight: 500, marginBottom: 4, display: "block" }}>API Key {config.mode === "local" ? "(optional)" : ""}</label>
          <input className="models-search-input" type="password" value={config.apiKey} onChange={(e) => setConfig({ ...config, apiKey: e.target.value })} placeholder="sk-..." />
        </div>

        <button className="btn btn-primary btn-sm" onClick={save} style={{ alignSelf: "flex-start" }}>
          Save Settings
        </button>
      </div>
    </div>
  );
}
