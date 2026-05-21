import { useState, useEffect, useRef, useCallback } from "react";
import { Search, X, Download, Trash, Refresh } from "../../assets/icons";
import { AgentMarkdown } from "../../components/AgentMarkdown";
import { useI18n } from "../../components/useI18n";
import { hermesAPI } from "@shared/hermes-api";

interface InstalledSkill {
  name: string;
  category: string;
  description: string;
  path: string;
}

interface BundledSkill {
  name: string;
  description: string;
  category: string;
  source: string;
  installed: boolean;
}

interface SkillsProps {
  profile?: string;
}

type Tab = "installed" | "browse" | "hub";

function Skills({ profile }: SkillsProps): React.JSX.Element {
  const { t } = useI18n();
  const [tab, setTab] = useState<Tab>("installed");
  const [installedSkills, setInstalledSkills] = useState<InstalledSkill[]>([]);
  const [bundledSkills, setBundledSkills] = useState<BundledSkill[]>([]);
  const [search, setSearch] = useState("");
  const [categoryFilter, setCategoryFilter] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [detailSkill, setDetailSkill] = useState<InstalledSkill | null>(null);
  const [detailContent, setDetailContent] = useState("");
  const [actionInProgress, setActionInProgress] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [hubSearch, setHubSearch] = useState("");
  const [hubResults, setHubResults] = useState<{ name: string; description: string; category: string; author: string; downloads: number; installed: boolean }[]>([]);
  const [hubLoading, setHubLoading] = useState(false);
  const searchRef = useRef<HTMLInputElement>(null);

  async function searchHub(): Promise<void> {
    if (!hubSearch.trim()) return;
    setHubLoading(true);
    try { setHubResults(await hermesAPI.searchSkillsHub(hubSearch.trim())); } catch {}
    setHubLoading(false);
  }

  async function installFromHub(name: string): Promise<void> {
    setActionInProgress(name);
    try { await hermesAPI.installFromHub(name, profile); await loadInstalled(); } catch {}
    setActionInProgress(null);
  }

  const loadInstalled = useCallback(async (): Promise<void> => {
    const list = await hermesAPI.listInstalledSkills(profile);
    setInstalledSkills(list);
  }, [profile]);

  const loadBundled = useCallback(async (): Promise<void> => {
    const list = await hermesAPI.listBundledSkills();
    setBundledSkills(list);
  }, []);

  const loadAll = useCallback(async (): Promise<void> => {
    setLoading(true);
    await Promise.all([loadInstalled(), loadBundled()]);
    setLoading(false);
  }, [loadInstalled, loadBundled]);

  useEffect(() => {
    loadAll();
  }, [loadAll]);

  async function handleViewDetail(skill: InstalledSkill): Promise<void> {
    setDetailSkill(skill);
    const content = await hermesAPI.getSkillContent(skill.path);
    setDetailContent(content);
  }

  async function handleInstall(name: string): Promise<void> {
    setActionInProgress(name);
    setError("");
    const result = await hermesAPI.installSkill(name, profile);
    setActionInProgress(null);
    if (result.success) {
      await loadInstalled();
    } else {
      setError(result.error || t("skills.installFailed"));
    }
  }

  async function handleUninstall(name: string): Promise<void> {
    setActionInProgress(name);
    setError("");
    const result = await hermesAPI.uninstallSkill(name, profile);
    setActionInProgress(null);
    if (result.success) {
      setDetailSkill(null);
      await loadInstalled();
    } else {
      setError(result.error || t("skills.uninstallFailed"));
    }
  }

  const installedNames = new Set(
    installedSkills.map((s) => s.name.toLowerCase()),
  );

  // Filter logic
  const filteredInstalled = installedSkills.filter((s) => {
    if (search) {
      const q = search.toLowerCase();
      return (
        s.name.toLowerCase().includes(q) ||
        s.description.toLowerCase().includes(q) ||
        s.category.toLowerCase().includes(q)
      );
    }
    return true;
  });

  const filteredBundled = bundledSkills.filter((s) => {
    let matches = true;
    if (search) {
      const q = search.toLowerCase();
      matches =
        s.name.toLowerCase().includes(q) ||
        s.description.toLowerCase().includes(q) ||
        s.category.toLowerCase().includes(q);
    }
    if (categoryFilter) {
      matches = matches && s.category === categoryFilter;
    }
    return matches;
  });

  // Get unique categories for filter pills
  const categories = Array.from(
    new Set(bundledSkills.map((s) => s.category)),
  ).sort();

  if (loading) {
    return (
      <div className="skills-container">
        <div className="skills-loading">
          <div className="loading-spinner" />
        </div>
      </div>
    );
  }

  return (
    <div className="skills-container">
      {/* Detail Panel */}
      {detailSkill && (
        <div
          className="skills-detail-overlay"
          onClick={() => setDetailSkill(null)}
        >
          <div className="skills-detail" onClick={(e) => e.stopPropagation()}>
            <div className="skills-detail-header">
              <div>
                <div className="skills-detail-name">{detailSkill.name}</div>
                <div className="skills-detail-category">
                  {detailSkill.category}
                </div>
              </div>
              <div className="skills-detail-actions">
                <button
                  className="btn btn-secondary btn-sm"
                  onClick={() => handleUninstall(detailSkill.name)}
                  disabled={actionInProgress === detailSkill.name}
                >
                  {actionInProgress === detailSkill.name ? (
                    t("skills.removing")
                  ) : (
                    <>
                      <Trash size={13} />
                      {t("skills.uninstall")}
                    </>
                  )}
                </button>
                <button
                  className="btn-ghost"
                  onClick={() => setDetailSkill(null)}
                >
                  <X size={18} />
                </button>
              </div>
            </div>
            <div className="skills-detail-content">
              <AgentMarkdown>{detailContent}</AgentMarkdown>
            </div>
          </div>
        </div>
      )}

      <div className="skills-header">
        <div>
          <h2 className="skills-title">{t("skills.title")}</h2>
          <p className="skills-subtitle">{t("skills.subtitle")}</p>
        </div>
        <button className="btn btn-secondary btn-sm" onClick={loadAll}>
          <Refresh size={14} />
          {t("skills.refresh")}
        </button>
      </div>

      {error && (
        <div className="skills-error">
          {error}
          <button className="btn-ghost" onClick={() => setError("")}>
            <X size={14} />
          </button>
        </div>
      )}

      {/* Tabs */}
      <div className="skills-tabs">
        <button
          className={`skills-tab ${tab === "installed" ? "active" : ""}`}
          onClick={() => setTab("installed")}
        >
          {t("skills.installedTab")} ({installedSkills.length})
        </button>
        <button
          className={`skills-tab ${tab === "browse" ? "active" : ""}`}
          onClick={() => setTab("browse")}
        >
          {t("skills.browseTab")} ({bundledSkills.length})
        </button>
        <button
          className={`skills-tab ${tab === "hub" ? "active" : ""}`}
          onClick={() => setTab("hub")}
        >
          Skills Hub
        </button>
      </div>

      {/* Search */}
      <div className="skills-search">
        <Search size={15} />
        <input
          ref={searchRef}
          className="skills-search-input"
          type="text"
          placeholder={
            tab === "installed"
              ? t("skills.filterInstalled")
              : t("skills.search")
          }
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        {search && (
          <button
            className="btn-ghost skills-search-clear"
            onClick={() => {
              setSearch("");
              searchRef.current?.focus();
            }}
          >
            <X size={14} />
          </button>
        )}
      </div>

      {/* Category filter pills (browse tab only) */}
      {tab === "browse" && categories.length > 0 && (
        <div className="skills-category-pills">
          <button
            className={`skills-pill ${categoryFilter === null ? "active" : ""}`}
            onClick={() => setCategoryFilter(null)}
          >
            {t("skills.all")}
          </button>
          {categories.map((cat) => (
            <button
              key={cat}
              className={`skills-pill ${categoryFilter === cat ? "active" : ""}`}
              onClick={() =>
                setCategoryFilter(categoryFilter === cat ? null : cat)
              }
            >
              {cat}
            </button>
          ))}
        </div>
      )}

      {/* Grid */}
      {tab === "installed" ? (
        filteredInstalled.length === 0 ? (
          <div className="skills-empty">
            <p className="skills-empty-text">
              {search
                ? t("skills.noMatchingInstalled")
                : t("skills.noInstalled")}
            </p>
            <p className="skills-empty-hint">
              {search
                ? t("skills.noMatchingHint")
                : t("skills.noInstalledHint")}
            </p>
          </div>
        ) : (
          <div className="skills-grid">
            {filteredInstalled.map((skill) => (
              <button
                key={`${skill.category}/${skill.name}`}
                className="skills-card"
                onClick={() => handleViewDetail(skill)}
              >
                <div className="skills-card-category">{skill.category}</div>
                <div className="skills-card-name">{skill.name}</div>
                {skill.description && (
                  <div className="skills-card-description">
                    {skill.description}
                  </div>
                )}
              </button>
            ))}
          </div>
        )
      ) : filteredBundled.length === 0 ? (
        <div className="skills-empty">
          <p className="skills-empty-text">{t("skills.noBrowseResults")}</p>
          <p className="skills-empty-hint">{t("skills.noBrowseResultsHint")}</p>
        </div>
      ) : (
        <div className="skills-grid">
          {tab === "hub" && (
          <div style={{ marginTop: 12 }}>
            <div className="skills-search">
              <Search size={15} />
              <input className="skills-search-input" type="text" placeholder="Search agentskills.io..." value={hubSearch}
                onChange={(e) => setHubSearch(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") searchHub(); }} />
              <button className="btn btn-primary btn-sm" onClick={searchHub} disabled={hubLoading}>
                {hubLoading ? <Refresh size={13} /> : <Search size={13} />} Search
              </button>
            </div>
            <div className="skills-grid" style={{ marginTop: 12 }}>
              {hubResults.map((skill) => {
                const alreadyInstalled = installedNames.has(skill.name.toLowerCase());
                return (
                  <div key={skill.name} className="skills-card" style={{ padding: 14 }}>
                    <div className="skills-card-header">
                      <div className="skills-card-name">{skill.name}</div>
                      <span className="skills-card-category" style={{ fontSize: 11, background: "var(--bg-tertiary)", padding: "2px 8px", borderRadius: 10 }}>{skill.category}</span>
                    </div>
                    <div className="skills-card-desc" style={{ fontSize: 12, marginTop: 4 }}>{skill.description}</div>
                    <div style={{ fontSize: 11, color: "var(--text-muted)", marginTop: 4 }}>by {skill.author} · {skill.downloads} downloads</div>
                    <div style={{ marginTop: 8 }}>
                      {alreadyInstalled ? (
                        <span style={{ fontSize: 12, color: "var(--accent)" }}>✓ Installed</span>
                      ) : (
                        <button className="btn btn-primary btn-sm" onClick={() => installFromHub(skill.name)} disabled={actionInProgress === skill.name}>
                          <Download size={12} /> {actionInProgress === skill.name ? "Installing..." : "Install"}
                        </button>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}
        {filteredBundled.map((skill) => {
            const isInstalled = installedNames.has(skill.name.toLowerCase());
            const isActioning = actionInProgress === skill.name;
            return (
              <div
                key={`${skill.category}/${skill.name}`}
                className="skills-card"
              >
                <div className="skills-card-category">{skill.category}</div>
                <div className="skills-card-name">{skill.name}</div>
                {skill.description && (
                  <div className="skills-card-description">
                    {skill.description}
                  </div>
                )}
                <div className="skills-card-footer">
                  {isInstalled ? (
                    <span className="skills-card-installed-badge">
                      {t("skills.installedBadge")}
                    </span>
                  ) : (
                    <button
                      className="btn btn-primary btn-sm skills-card-install-btn"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleInstall(skill.name);
                      }}
                      disabled={isActioning}
                    >
                      {isActioning ? (
                        t("skills.installing")
                      ) : (
                        <>
                          <Download size={13} />
                          {t("skills.install")}
                        </>
                      )}
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

export default Skills;
