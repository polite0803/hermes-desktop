import { useState, useEffect, useCallback } from "react";
import { useI18n } from "@renderer/components/useI18n";
import { hermesAPI } from "@shared/hermes-api";
import { Power } from "lucide-react";

export default function Plugins(): React.JSX.Element {
  const { t } = useI18n();
  const [plugins, setPlugins] = useState<
    {
      name: string;
      description: string;
      installed: boolean;
      enabled: boolean;
    }[]
  >([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    try {
      setPlugins(await hermesAPI.listPlugins());
    } catch {
      /* ignore */
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  async function toggle(name: string, enabled: boolean): Promise<void> {
    try {
      await (enabled
        ? hermesAPI.disablePlugin(name)
        : hermesAPI.enablePlugin(name));
      await load();
    } catch {
      /* ignore */
    }
  }

  if (loading)
    return (
      <div className="settings-container">
        <h1 className="settings-header">{t("navigation.plugins")}</h1>
        <div className="loading-spinner" />
      </div>
    );

  return (
    <div className="settings-container">
      <div className="models-header">
        <h1 className="settings-header models-title-tight">
          {t("navigation.plugins")}
        </h1>
        <p className="models-subtitle">{t("plugins.subtitle")}</p>
      </div>
      {plugins.length === 0 ? (
        <div className="models-empty">
          <p className="models-empty-text">{t("plugins.empty")}</p>
        </div>
      ) : (
        <div className="models-grid">
          {plugins.map((p) => (
            <div
              key={p.name}
              className="models-card"
              style={{
                padding: 16,
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
              }}
            >
              <div>
                <div className="models-card-name">{p.name}</div>
                <div style={{ fontSize: 12, color: "var(--text-muted)" }}>
                  {p.installed
                    ? t("plugins.installed")
                    : t("plugins.notInstalled")}
                </div>
              </div>
              <button
                className={`btn btn-sm ${p.enabled ? "btn-primary" : ""}`}
                onClick={() => toggle(p.name, p.enabled)}
                disabled={!p.installed}
              >
                <Power size={12} />{" "}
                {p.enabled ? t("common.disable") : t("common.enable")}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
