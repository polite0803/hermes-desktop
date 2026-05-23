import { useState, useEffect, useCallback } from "react";
import { useI18n } from "@renderer/components/useI18n";
import { hermesAPI, type McpServer } from "@shared/hermes-api";
import { Plus, Trash, Play, RefreshCw, Download } from "lucide-react";

export default function McpServers(): React.JSX.Element {
  const { t } = useI18n();
  const [servers, setServers] = useState<McpServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [formName, setFormName] = useState("");
  const [formCommand, setFormCommand] = useState("npx");
  const [formArgs, setFormArgs] = useState("");
  const [formError, setFormError] = useState("");
  const [testing, setTesting] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<Record<string, boolean | null>>(
    {},
  );

  const loadServers = useCallback(async () => {
    try {
      const list = await hermesAPI.listMcpServers();
      setServers(list);
    } catch {
      /* ignore */
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    void loadServers();
  }, [loadServers]);

  async function handleAdd(): Promise<void> {
    if (!formName.trim() || !formCommand.trim()) {
      setFormError(t("mcp.fillAllFields"));
      return;
    }
    try {
      const args = formArgs.trim() ? formArgs.split(/\s+/) : [];
      await hermesAPI.addMcpServer(formName.trim(), formCommand.trim(), args);
      setShowAdd(false);
      setFormName("");
      setFormCommand("npx");
      setFormArgs("");
      setFormError("");
      await loadServers();
    } catch (e) {
      setFormError(String(e));
    }
  }

  async function handleRemove(name: string): Promise<void> {
    try {
      await hermesAPI.removeMcpServer(name);
      await loadServers();
    } catch {
      /* ignore */
    }
  }

  async function handleToggle(name: string, enabled: boolean): Promise<void> {
    try {
      await hermesAPI.updateMcpServer(name, { enabled: !enabled });
      await loadServers();
    } catch {
      /* ignore */
    }
  }

  async function handleTest(name: string): Promise<void> {
    setTesting(name);
    try {
      const ok = await hermesAPI.testMcpServer(name);
      setTestResult((p) => ({ ...p, [name]: ok }));
    } catch {
      setTestResult((p) => ({ ...p, [name]: false }));
    }
    setTesting(null);
  }

  if (loading) {
    return (
      <div className="settings-container">
        <h1 className="settings-header">{t("navigation.mcpServers")}</h1>
        <div className="loading-spinner" />
      </div>
    );
  }

  return (
    <div className="settings-container">
      <div className="models-header">
        <div>
          <h1 className="settings-header models-title-tight">
            {t("navigation.mcpServers")}
          </h1>
          <p className="models-subtitle">{t("mcp.subtitle")}</p>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button
            className="btn btn-primary btn-sm"
            onClick={() => setShowAdd(true)}
          >
            <Plus size={14} /> {t("mcp.addServer")}
          </button>
          <button
            className="btn btn-sm"
            onClick={async () => {
              try {
                await hermesAPI.installComputerUseMcp();
                await loadServers();
              } catch (e) {
                alert(t("mcp.restartError", { error: String(e) }));
              }
            }}
          >
            <Download size={14} /> {t("mcp.installComputerUse")}
          </button>
        </div>
      </div>

      {showAdd && (
        <div className="models-card" style={{ padding: 16, marginBottom: 16 }}>
          <input
            className="models-search-input"
            style={{ marginBottom: 8, width: "100%" }}
            placeholder={t("mcp.serverName")}
            value={formName}
            onChange={(e) => setFormName(e.target.value)}
          />
          <input
            className="models-search-input"
            style={{ marginBottom: 8, width: "100%" }}
            placeholder={t("mcp.command")}
            value={formCommand}
            onChange={(e) => setFormCommand(e.target.value)}
          />
          <input
            className="models-search-input"
            style={{ marginBottom: 8, width: "100%" }}
            placeholder={t("mcp.args")}
            value={formArgs}
            onChange={(e) => setFormArgs(e.target.value)}
          />
          {formError && (
            <p style={{ color: "var(--error)", fontSize: 13 }}>{formError}</p>
          )}
          <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
            <button className="btn btn-primary btn-sm" onClick={handleAdd}>
              {t("common.save")}
            </button>
            <button className="btn btn-sm" onClick={() => setShowAdd(false)}>
              {t("common.cancel")}
            </button>
          </div>
        </div>
      )}

      {servers.length === 0 ? (
        <div className="models-empty">
          <p className="models-empty-text">{t("mcp.empty")}</p>
        </div>
      ) : (
        <div className="models-grid">
          {servers.map((s) => (
            <div
              key={s.name}
              className="models-card"
              style={{
                padding: 16,
                display: "flex",
                flexDirection: "column",
                gap: 8,
              }}
            >
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}
              >
                <div>
                  <div className="models-card-name">{s.name}</div>
                  <div
                    style={{
                      fontSize: 12,
                      color: "var(--text-muted)",
                      fontFamily: "monospace",
                    }}
                  >
                    {s.command} {s.args.join(" ")}
                  </div>
                </div>
                <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                  <span
                    style={{
                      fontSize: 11,
                      padding: "2px 8px",
                      borderRadius: 10,
                      background: s.enabled
                        ? "var(--accent-subtle)"
                        : "var(--bg-tertiary)",
                      color: s.enabled
                        ? "var(--accent-text)"
                        : "var(--text-muted)",
                    }}
                  >
                    {s.enabled ? t("mcp.enabled") : t("mcp.disabled")}
                  </span>
                </div>
              </div>
              <div style={{ display: "flex", gap: 6 }}>
                <button
                  className="btn btn-sm"
                  onClick={() => handleTest(s.name)}
                  disabled={testing === s.name}
                >
                  {testing === s.name ? (
                    <RefreshCw size={12} className="spin" />
                  ) : (
                    <Play size={12} />
                  )}{" "}
                  {t("mcp.test")}
                </button>
                {testResult[s.name] !== undefined && (
                  <span
                    style={{
                      fontSize: 12,
                      color: testResult[s.name]
                        ? "var(--accent)"
                        : "var(--error)",
                    }}
                  >
                    {testResult[s.name] ? t("common.yes") : t("common.no")}
                  </span>
                )}
                <button
                  className="btn btn-sm"
                  onClick={() => handleToggle(s.name, s.enabled)}
                >
                  {s.enabled ? t("common.disable") : t("common.enable")}
                </button>
                <button
                  className="btn btn-sm"
                  style={{ color: "var(--error)" }}
                  onClick={() => handleRemove(s.name)}
                >
                  <Trash size={12} />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
