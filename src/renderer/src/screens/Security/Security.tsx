import { useState, useEffect, useCallback } from "react";
import { useI18n } from "@renderer/components/useI18n";
import { hermesAPI } from "@shared/hermes-api";
import { Shield, Check, AlertTriangle, Loader2 } from "lucide-react";

export default function Security(): React.JSX.Element {
  const { t } = useI18n();
  const [loading, setLoading] = useState(true);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState("");
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{
    ok: boolean;
    msg: string;
  } | null>(null);
  const [tunnelActive, setTunnelActive] = useState(false);
  const [config, setConfig] = useState({
    mode: "local",
    remoteUrl: "",
    apiKey: "",
    ssh: {
      host: "",
      port: "22",
      username: "root",
      keyPath: "",
      remotePort: "8642",
    },
  });

  const load = useCallback(async () => {
    setError("");
    try {
      const conn = await hermesAPI.getConnectionConfig();
      setConfig({
        mode: conn.mode,
        remoteUrl: conn.remoteUrl || "",
        apiKey: "",
        ssh: {
          host: conn.ssh?.host || "",
          port: String(conn.ssh?.port || 22),
          username: conn.ssh?.username || "root",
          keyPath: conn.ssh?.keyPath || "",
          remotePort: String(conn.ssh?.remotePort || 8642),
        },
      });
    } catch (e) {
      setError(String(e));
    }
    try {
      setTunnelActive(await hermesAPI.isSshTunnelActive());
    } catch (e) {
      setError(String(e));
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  async function save(): Promise<void> {
    await hermesAPI.setConnectionConfig(
      config.mode,
      config.remoteUrl,
      config.apiKey || undefined,
    );
    if (config.mode === "ssh") {
      await hermesAPI.setSshConfig(
        config.ssh.host,
        parseInt(config.ssh.port) || 22,
        config.ssh.username,
        config.ssh.keyPath,
        parseInt(config.ssh.remotePort) || 8642,
        parseInt(config.ssh.remotePort) || 8642,
      );
    }
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  async function testConnection(): Promise<void> {
    setTesting(true);
    setTestResult(null);
    try {
      const ok = await hermesAPI.testRemoteConnection(
        config.remoteUrl,
        config.apiKey || undefined,
      );
      setTestResult({
        ok,
        msg: ok ? t("security.testOk") : t("security.testFail"),
      });
    } catch (e) {
      setTestResult({ ok: false, msg: String(e) });
    }
    setTesting(false);
  }

  async function toggleTunnel(): Promise<void> {
    try {
      if (tunnelActive) {
        await hermesAPI.stopSshTunnel();
        setTunnelActive(false);
      } else {
        await hermesAPI.startSshTunnel();
        setTunnelActive(true);
      }
    } catch (e) {
      setError(String(e));
    }
  }

  if (loading)
    return (
      <div className="settings-container">
        <h1 className="settings-header">{t("navigation.security")}</h1>
        <div className="loading-spinner" />
      </div>
    );

  return (
    <div className="settings-container">
      <div className="models-header">
        <div>
          <h1 className="settings-header models-title-tight">
            {t("navigation.security")}
          </h1>
          <p className="models-subtitle">{t("security.subtitle")}</p>
        </div>
        {saved && (
          <span style={{ fontSize: 12, color: "var(--accent)" }}>
            {t("security.saved")}
          </span>
        )}
      </div>

      {error && (
        <div
          style={{
            padding: "8px 12px",
            marginBottom: 12,
            fontSize: 12,
            color: "var(--error)",
            background: "var(--bg-tertiary)",
            borderRadius: 6,
          }}
        >
          {error}
          <button
            className="btn-ghost"
            onClick={() => setError("")}
            style={{ marginLeft: 8 }}
          >
            ✕
          </button>
        </div>
      )}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 16,
          maxWidth: 520,
        }}
      >
        <div className="models-card" style={{ padding: 16 }}>
          <label
            style={{
              fontSize: 13,
              fontWeight: 600,
              marginBottom: 8,
              display: "block",
            }}
          >
            {t("security.connectionMode")}
          </label>
          <select
            className="models-search-input"
            value={config.mode}
            onChange={(e) => setConfig({ ...config, mode: e.target.value })}
          >
            <option value="local">{t("security.modeLocal")}</option>
            <option value="remote">{t("security.modeRemote")}</option>
            <option value="ssh">{t("security.modeSSH")}</option>
          </select>
        </div>

        {config.mode !== "local" && (
          <div className="models-card" style={{ padding: 16 }}>
            <label
              style={{
                fontSize: 13,
                fontWeight: 600,
                marginBottom: 8,
                display: "block",
              }}
            >
              {t("security.remoteUrl")}
            </label>
            <input
              className="models-search-input"
              value={config.remoteUrl}
              onChange={(e) =>
                setConfig({ ...config, remoteUrl: e.target.value })
              }
              placeholder={t("security.remoteUrlPlaceholder")}
            />
            {config.mode === "remote" && (
              <div
                style={{
                  marginTop: 8,
                  display: "flex",
                  gap: 8,
                  alignItems: "center",
                }}
              >
                <button
                  className="btn btn-sm"
                  onClick={testConnection}
                  disabled={testing}
                >
                  {testing ? (
                    <Loader2 size={13} className="spin" />
                  ) : (
                    <Check size={13} />
                  )}{" "}
                  {t("security.testConnection")}
                </button>
                {testResult && (
                  <span
                    style={{
                      fontSize: 12,
                      color: testResult.ok ? "var(--accent)" : "var(--error)",
                    }}
                  >
                    {testResult.ok ? (
                      <Check size={12} />
                    ) : (
                      <AlertTriangle size={12} />
                    )}{" "}
                    {testResult.msg}
                  </span>
                )}
              </div>
            )}
          </div>
        )}

        <div className="models-card" style={{ padding: 16 }}>
          <label
            style={{
              fontSize: 13,
              fontWeight: 600,
              marginBottom: 8,
              display: "block",
            }}
          >
            {t("security.apiKey")}{" "}
            {config.mode === "local" ? `(${t("security.optional")})` : ""}
          </label>
          <input
            className="models-search-input"
            type="password"
            value={config.apiKey}
            onChange={(e) => setConfig({ ...config, apiKey: e.target.value })}
            placeholder={t("security.apiKeyPlaceholder")}
          />
        </div>

        {config.mode === "ssh" && (
          <>
            <div className="models-card" style={{ padding: 16 }}>
              <label
                style={{
                  fontSize: 14,
                  fontWeight: 600,
                  marginBottom: 12,
                  display: "block",
                }}
              >
                {t("security.sshConfig")}
              </label>
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <input
                  className="models-search-input"
                  placeholder={t("security.sshHost")}
                  value={config.ssh.host}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      ssh: { ...config.ssh, host: e.target.value },
                    })
                  }
                />
                <div style={{ display: "flex", gap: 8 }}>
                  <input
                    className="models-search-input"
                    placeholder={t("security.sshPort")}
                    value={config.ssh.port}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        ssh: { ...config.ssh, port: e.target.value },
                      })
                    }
                    style={{ width: 80 }}
                  />
                  <input
                    className="models-search-input"
                    placeholder={t("security.sshUser")}
                    value={config.ssh.username}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        ssh: { ...config.ssh, username: e.target.value },
                      })
                    }
                    style={{ flex: 1 }}
                  />
                </div>
                <input
                  className="models-search-input"
                  placeholder={t("security.sshKeyPath")}
                  value={config.ssh.keyPath}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      ssh: { ...config.ssh, keyPath: e.target.value },
                    })
                  }
                />
              </div>
              <div
                style={{
                  marginTop: 8,
                  display: "flex",
                  gap: 8,
                  alignItems: "center",
                }}
              >
                <button className="btn btn-sm" onClick={toggleTunnel}>
                  <Shield size={13} />{" "}
                  {tunnelActive
                    ? t("security.stopTunnel")
                    : t("security.startTunnel")}
                </button>
                <span
                  style={{
                    fontSize: 12,
                    color: tunnelActive ? "var(--accent)" : "var(--text-muted)",
                  }}
                >
                  {tunnelActive
                    ? t("security.tunnelActive")
                    : t("security.tunnelInactive")}
                </span>
              </div>
            </div>
          </>
        )}

        <button
          className="btn btn-primary btn-sm"
          onClick={save}
          style={{ alignSelf: "flex-start" }}
        >
          {t("security.saveSettings")}
        </button>
      </div>
    </div>
  );
}
