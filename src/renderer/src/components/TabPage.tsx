import { useState } from "react";

interface TabDef {
  key: string;
  label: string;
}

interface TabPageProps {
  tabs: TabDef[];
  children: (activeTab: string) => React.ReactNode;
  defaultTab?: string;
}

export function TabPage({
  tabs,
  children,
  defaultTab,
}: TabPageProps): React.JSX.Element {
  const [active, setActive] = useState(defaultTab || tabs[0]?.key || "");

  return (
    <div className="tab-page">
      <div className="tab-bar">
        {tabs.map((t) => (
          <button
            key={t.key}
            className={`tab-bar-item ${active === t.key ? "active" : ""}`}
            onClick={() => setActive(t.key)}
          >
            {t.label}
          </button>
        ))}
      </div>
      <div className="tab-content">{children(active)}</div>
    </div>
  );
}
