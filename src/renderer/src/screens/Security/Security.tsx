import { useState, useEffect, useCallback } from "react";
import { useI18n } from "@renderer/components/useI18n";
import { hermesAPI } from "@shared/hermes-api";

export default function Security(): React.JSX.Element {
  const { t } = useI18n();
  const [config, setConfig] = useState<Record<string, boolean>>({});
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    try {
      const platforms = await hermesAPI.getPlatformEnabled();
      setConfig(platforms);
    } catch {}
    setLoading(false);
  }, []);

  useEffect(() => { void load(); }, [load]);

  async function toggle(key: string, enabled: boolean): Promise<void> {
    try { await hermesAPI.setPlatformEnabled(key, !enabled); await load(); } catch {}
  }

  if (loading) return <div className="settings-container"><h1 className="settings-header">{t("navigation.security")}</h1><div className="loading-spinner" /></div>;

  return (
    <div className="settings-container">
      <div className="models-header">
        <h1 className="settings-header models-title-tight">{t("navigation.security")}</h1>
        <p className="models-subtitle">{t("security.subtitle")}</p>
      </div>
      <div className="models-grid">
        {Object.entries(config).map(([key, enabled]) => (
          <div key={key} className="models-card" style={{ padding: 16, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <div>
              <div className="models-card-name">{key}</div>
            </div>
            <button className={`btn btn-sm ${enabled ? "btn-primary" : ""}`} onClick={() => toggle(key, enabled)}>
              {enabled ? t("common.disable") : t("common.enable")}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
