import { useState, useEffect, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface HotkeyForFrontend {
  action: string;
  shortcut: string;
}

interface ConfigForFrontend {
  launchOnLogin: boolean;
  gapSize: number;
  screenEdgeGapTop: number;
  screenEdgeGapBottom: number;
  screenEdgeGapLeft: number;
  screenEdgeGapRight: number;
  screenEdgeGapsOnMainScreenOnly: boolean;
  taskbarGapCompensation: number;
  taskbarGapCompensationLeft: number;
  taskbarGapCompensationRight: number;
  applyGapsToMaximize: boolean;
  applyGapsToMaximizeHeight: boolean;
  thirdsLayout: string;
  hotkeys: HotkeyForFrontend[];
}

const SECTIONS: { title: string; actions: string[] }[] = [
  { title: "Halves", actions: ["LeftHalf", "RightHalf", "TopHalf", "BottomHalf"] },
  { title: "Quarters", actions: ["UpperLeft", "UpperRight", "LowerLeft", "LowerRight"] },
  {
    title: "Thirds",
    actions: [
      "FirstThird",
      "FirstTwoThirds",
      "CenterThird",
      "LastTwoThirds",
      "LastThird",
      "CenterTwoThirds",
    ],
  },
  {
    title: "Other",
    actions: ["Maximize", "Center", "Undo", "NextDisplay", "PreviousDisplay"],
  },
];

/** Map key from KeyboardEvent to our shortcut label (e.g. ArrowLeft -> Left). */
function keyToShortcutLabel(key: string): string {
  const map: Record<string, string> = {
    ArrowLeft: "Left",
    ArrowRight: "Right",
    ArrowUp: "Up",
    ArrowDown: "Down",
    Enter: "Enter",
    Delete: "Delete",
  };
  return map[key] ?? key;
}

/** Build shortcut string from a keydown event (e.g. "Win+Alt+Left"). */
function getShortcutFromEvent(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.metaKey) parts.push("Win");
  if (e.altKey) parts.push("Alt");
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.shiftKey) parts.push("Shift");
  parts.push(keyToShortcutLabel(e.key));
  return parts.join("+");
}

function SaveIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden style={{ pointerEvents: "none" }}>
      <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
      <polyline points="17 21 17 13 7 13 7 21" />
      <polyline points="7 3 7 8 15 8" />
    </svg>
  );
}

function RevertIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden style={{ pointerEvents: "none" }}>
      <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
      <path d="M3 3v5h5" />
    </svg>
  );
}

function App() {
  const [config, setConfig] = useState<ConfigForFrontend | null>(null);
  const [savedConfig, setSavedConfig] = useState<ConfigForFrontend | null>(null);
  const [justSaved, setJustSaved] = useState(false);
  const [configPath, setConfigPath] = useState("");
  const [lastAction, setLastAction] = useState<string | null>(null);
  const configRef = useRef(config);
  configRef.current = config;

  const settingsDirty =
    config != null &&
    savedConfig != null &&
    (config.launchOnLogin !== savedConfig.launchOnLogin ||
      config.gapSize !== savedConfig.gapSize);

  async function loadConfig() {
    try {
      const c = await invoke<ConfigForFrontend>("load_config");
      setConfig(c);
      setSavedConfig(c);
    } catch (e) {
      console.error(e);
    }
  }

  async function reloadConfig() {
    try {
      const c = await invoke<ConfigForFrontend>("reload_config");
      setConfig(c);
      setSavedConfig(c);
    } catch (e) {
      console.error(e);
    }
  }

  async function openConfig() {
    try {
      await invoke("open_config_in_editor");
    } catch (e) {
      console.error(e);
    }
  }

  function saveConfig() {
    const current = configRef.current;
    if (!current) return;
    setJustSaved(true);
    invoke<ConfigForFrontend>("save_config", { payload: current })
      .then((c) => {
        setConfig(c);
        setSavedConfig(c);
        setTimeout(() => setJustSaved(false), 1000);
      })
      .catch((e) => {
        setJustSaved(false);
        console.error(e);
        window.alert(`Save failed: ${e instanceof Error ? e.message : String(e)}`);
      });
  }

  async function revertToDefaults() {
    if (!window.confirm("Revert all settings to defaults?")) return;
    try {
      const c = await invoke<ConfigForFrontend>("revert_to_defaults");
      setConfig(c);
      setSavedConfig(c);
    } catch (e) {
      console.error(e);
    }
  }

  const actionToShortcut =
    (config?.hotkeys ?? []).reduce(
      (acc, h) => {
        acc[h.action] = h.shortcut;
        return acc;
      },
      {} as Record<string, string>
    );

  const shortcutToAction = useMemo(
    () =>
      (config?.hotkeys ?? []).reduce(
        (acc, h) => {
          acc[h.shortcut] = h.action;
          return acc;
        },
        {} as Record<string, string>
      ),
    [config?.hotkeys]
  );

  useEffect(() => {
    loadConfig();
    invoke<string>("get_config_path").then(setConfigPath).catch(console.error);
  }, []);

  // When our window has focus, WH_KEYBOARD_LL often doesn't fire; handle shortcuts in the frontend.
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const shortcut = getShortcutFromEvent(e);
      const action = shortcutToAction[shortcut];
      if (action) {
        e.preventDefault();
        e.stopPropagation();
        invoke("run_action", { action }).catch(console.error);
      }
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [shortcutToAction]);

  return (
    <main className="container">
      <h1>RectangleWin</h1>

      <section className="card">
        <div className="card-header">
          <h2>Settings</h2>
          <div className="card-actions">
            <button
              type="button"
              className="icon-btn"
              onClick={revertToDefaults}
              title="Revert to defaults"
              aria-label="Revert to defaults"
            >
              <RevertIcon />
            </button>
            {(settingsDirty || justSaved) && (
              <button
                type="button"
                className={`icon-btn ${justSaved ? "saved" : ""}`}
                onClick={saveConfig}
                title="Save settings"
                aria-label="Save settings"
              >
                <SaveIcon />
              </button>
            )}
          </div>
        </div>
        <div className="setting-row">
          <span>Launch at startup</span>
          <label className="switch">
            <input
              type="checkbox"
              checked={config?.launchOnLogin ?? true}
              onChange={(e) =>
                setConfig((c) => (c ? { ...c, launchOnLogin: e.target.checked } : null))
              }
            />
            <span className="switch-slider" />
          </label>
        </div>
        <div className="setting-row setting-row-slider">
          <span className="setting-label-with-tip">
            Gaps between windows (px)
            <span
              className="tooltip-trigger"
              title="Negative = overdraw to hide semi-transparent window edges."
              aria-label="Negative = overdraw to hide semi-transparent window edges."
            >
              ?
            </span>
          </span>
          <div className="slider-wrap">
            <span className="slider-value">{config?.gapSize ?? 0}</span>
            <div className="slider-track">
              <span className="slider-tick" aria-hidden />
              <input
                type="range"
                min={-10}
                max={20}
                step={1}
                value={config?.gapSize ?? 0}
                onChange={(e) =>
                  setConfig((c) =>
                    c ? { ...c, gapSize: parseFloat(e.target.value) || 0 } : null
                  )
                }
              />
            </div>
          </div>
        </div>
      </section>

      <section className="card">
        <h2>Shortcuts</h2>
        <p className="muted small">Win+Alt by default. Edit below or in config.json.</p>
        <div className="shortcuts-grid">
          {SECTIONS.map((section) => (
            <div key={section.title} className="shortcut-section">
              <h3 className="section-title">{section.title}</h3>
              <ul className="shortcut-list">
                {section.actions.map((action) => {
                  const shortcut = actionToShortcut[action];
                  if (!shortcut) return null;
                  const label = action;
                  return (
                    <li key={action} className="shortcut-row">
                      <span>{label}</span>
                      <span className="shortcut-key">{shortcut}</span>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </div>
      </section>

      <section className="card">
        <p className="row">
          <span className="muted">Last shortcut:</span>
          <span>{lastAction ?? "(none yet)"}</span>
        </p>
        <button type="button" className="btn" onClick={reloadConfig}>
          Reload from config
        </button>
        <p className="muted small">Edit raw settings (hotkeys, gaps, etc.) in JSON:</p>
        <button type="button" className="link" onClick={openConfig}>
          Open config.json
        </button>
        {configPath && (
          <p className="muted small" style={{ marginTop: 8 }}>
            Config: {configPath}
          </p>
        )}
      </section>
    </main>
  );
}

export default App;
