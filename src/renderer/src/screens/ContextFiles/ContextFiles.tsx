import { useState, useEffect, useCallback, useRef } from "react";
import { useI18n } from "@renderer/components/useI18n";
import { hermesAPI } from "@shared/hermes-api";
import { Plus } from "lucide-react";

export default function ContextFiles(): React.JSX.Element {
  const { t } = useI18n();
  const [files, setFiles] = useState<string[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [content, setContent] = useState("");
  const [loading, setLoading] = useState(true);
  const [saved, setSaved] = useState(false);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const loaded = useRef(false);

  const load = useCallback(async () => {
    try {
      setFiles(await hermesAPI.listContextFiles());
    } catch {
      /* ignore */
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    if (!selected) return;
    (async () => {
      loaded.current = false;
      const { content: c } = await hermesAPI.readContextFile(selected);
      setContent(c);
      setTimeout(() => {
        loaded.current = true;
      }, 300);
    })();
  }, [selected]);

  useEffect(() => {
    if (!loaded.current || !selected) return;
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(async () => {
      await hermesAPI.writeContextFile(selected, content);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    }, 500);
    return () => {
      if (saveTimer.current) clearTimeout(saveTimer.current);
    };
  }, [content, selected]);

  async function handleCreate(): Promise<void> {
    const name = prompt(t("contextFiles.fileNamePrompt"));
    if (!name) return;
    await hermesAPI.writeContextFile(name, "");
    await load();
    setSelected(name);
  }

  if (loading)
    return (
      <div className="settings-container">
        <h1 className="settings-header">{t("navigation.contextFiles")}</h1>
        <div className="loading-spinner" />
      </div>
    );

  return (
    <div
      className="settings-container"
      style={{ flexDirection: "row", gap: 0 }}
    >
      <div
        style={{
          width: 200,
          borderRight: "1px solid var(--border)",
          padding: 12,
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: 8,
          }}
        >
          <h2 style={{ fontSize: 14, fontWeight: 600 }}>
            {t("navigation.contextFiles")}
          </h2>
          <button className="btn btn-sm" onClick={handleCreate}>
            <Plus size={12} />
          </button>
        </div>
        {files.map((f) => (
          <div
            key={f}
            className={`sidebar-nav-item ${selected === f ? "active" : ""}`}
            onClick={() => setSelected(f)}
            style={{ cursor: "pointer", padding: "8px 10px", fontSize: 13 }}
          >
            {f}
          </div>
        ))}
      </div>
      <div style={{ flex: 1, padding: 12 }}>
        {selected ? (
          <div>
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                marginBottom: 8,
              }}
            >
              <h2 style={{ fontSize: 16, fontWeight: 600 }}>{selected}</h2>
              {saved && (
                <span style={{ fontSize: 11, color: "var(--accent)" }}>
                  {t("common.saved")}
                </span>
              )}
            </div>
            <textarea
              className="soul-textarea"
              value={content}
              onChange={(e) => setContent(e.target.value)}
              style={{
                width: "100%",
                height: "calc(100vh - 160px)",
                background: "var(--bg-tertiary)",
                color: "var(--text-primary)",
                border: "1px solid var(--border)",
                borderRadius: 8,
                padding: 16,
                fontFamily: "monospace",
                fontSize: 13,
                resize: "none",
              }}
            />
          </div>
        ) : (
          <div className="models-empty">
            <p className="models-empty-text">{t("contextFiles.selectFile")}</p>
          </div>
        )}
      </div>
    </div>
  );
}
