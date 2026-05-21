import { useState, useEffect, useCallback } from "react";
import { useI18n } from "@renderer/components/useI18n";
import { hermesAPI } from "@shared/hermes-api";
import { RefreshCw, Play } from "lucide-react";

export default function Curator(): React.JSX.Element {
  const { t } = useI18n();
  const [status, setStatus] = useState("");
  const [report, setReport] = useState("");
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    try { setStatus(await hermesAPI.curatorStatus()); } catch {}
    try { setReport(await hermesAPI.curatorReport()); } catch {}
    setLoading(false);
  }, []);

  useEffect(() => { void load(); }, [load]);

  async function trigger(): Promise<void> {
    setLoading(true);
    try { await hermesAPI.curatorTrigger(); await load(); } catch {}
    setLoading(false);
  }

  if (loading) return <div className="settings-container"><h1 className="settings-header">Curator</h1><div className="loading-spinner" /></div>;

  return (
    <div className="settings-container">
      <div className="models-header">
        <div>
          <h1 className="settings-header models-title-tight">Curator</h1>
          <p className="models-subtitle">Autonomous skill library maintenance — grades, prunes, and consolidates your skills</p>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-sm" onClick={load}><RefreshCw size={14} /> Refresh</button>
          <button className="btn btn-primary btn-sm" onClick={trigger}><Play size={14} /> Run Curator</button>
        </div>
      </div>

      {status && (
        <div className="models-card" style={{ padding: 16, marginBottom: 16, whiteSpace: "pre-wrap", fontFamily: "monospace", fontSize: 12, maxHeight: 300, overflow: "auto" }}>
          {status}
        </div>
      )}

      {report && (
        <div className="models-card" style={{ padding: 16, whiteSpace: "pre-wrap", fontFamily: "monospace", fontSize: 12, maxHeight: 500, overflow: "auto" }}>
          {report}
        </div>
      )}
    </div>
  );
}
