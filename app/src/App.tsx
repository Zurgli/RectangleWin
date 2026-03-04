import { useState, useEffect, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import appIconUrl from "../src-tauri/icons/Icon.svg?url";
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

/** Display label for Thirds-section actions: show "Fifth(s)" when layout is Fifths. */
function thirdsSectionActionLabel(action: string, layout: string): string {
  if (layout !== "Fifths" && !layout.toLowerCase().includes("fifth")) return action;
  const map: Record<string, string> = {
    FirstThird: "FirstFifth",
    FirstTwoThirds: "FirstTwoFifths",
    CenterThird: "CenterFifth",
    LastTwoThirds: "LastTwoFifths",
    LastThird: "LastFifth",
    CenterTwoThirds: "CenterTwoFifths",
  };
  return map[action] ?? action;
}

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

function ChevronDownIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden style={{ pointerEvents: "none" }}>
      <path d="M6 9l6 6 6-6" />
    </svg>
  );
}

function OpenFolderIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden style={{ pointerEvents: "none" }}>
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      <path d="M2 10h20" />
    </svg>
  );
}

/** Mini layout icon for a shortcut action (matches original app tile style). */
function ShortcutTileIcon({ action }: { action: string }) {
  const size = 22;
  if (action === "Undo") {
    return (
      <span
        className="shortcut-tile shortcut-tile-undo"
        aria-hidden
        style={{
          position: "relative",
          display: "inline-block",
          width: size,
          height: size,
          boxSizing: "border-box",
        }}
      >
        <span
          className="shortcut-tile-undo-inner"
          style={{
            position: "absolute",
            top: 1,
            left: 1,
            right: 1,
            bottom: 1,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M3 10h10a5 5 0 0 1 5 5v0a5 5 0 0 1-5 5H3" />
            <path d="M3 10l4-4M3 10l4 4" />
          </svg>
        </span>
      </span>
    );
  }
  type GridSpec = { templateColumns: string; templateRows: string; col: number; row: number; colSpan?: number; rowSpan?: number };
  const specs: Record<string, GridSpec> = {
    LeftHalf: { templateColumns: "1fr 1fr", templateRows: "1fr", col: 1, row: 1 },
    RightHalf: { templateColumns: "1fr 1fr", templateRows: "1fr", col: 2, row: 1 },
    TopHalf: { templateColumns: "1fr", templateRows: "1fr 1fr", col: 1, row: 1 },
    BottomHalf: { templateColumns: "1fr", templateRows: "1fr 1fr", col: 1, row: 2 },
    UpperLeft: { templateColumns: "1fr 1fr", templateRows: "1fr 1fr", col: 1, row: 1 },
    UpperRight: { templateColumns: "1fr 1fr", templateRows: "1fr 1fr", col: 2, row: 1 },
    LowerLeft: { templateColumns: "1fr 1fr", templateRows: "1fr 1fr", col: 1, row: 2 },
    LowerRight: { templateColumns: "1fr 1fr", templateRows: "1fr 1fr", col: 2, row: 2 },
    FirstThird: { templateColumns: "1fr 1fr 1fr", templateRows: "1fr", col: 1, row: 1, colSpan: 1 },
    FirstTwoThirds: { templateColumns: "1fr 1fr 1fr", templateRows: "1fr", col: 1, row: 1, colSpan: 2 },
    CenterThird: { templateColumns: "1fr 1fr 1fr", templateRows: "1fr", col: 2, row: 1, colSpan: 1 },
    LastTwoThirds: { templateColumns: "1fr 1fr 1fr", templateRows: "1fr", col: 2, row: 1, colSpan: 2 },
    LastThird: { templateColumns: "1fr 1fr 1fr", templateRows: "1fr", col: 3, row: 1, colSpan: 1 },
    CenterTwoThirds: { templateColumns: "1fr 1fr 1fr 1fr 1fr 1fr", templateRows: "1fr", col: 2, row: 1, colSpan: 4 },
    Maximize: { templateColumns: "1fr", templateRows: "1fr", col: 1, row: 1 },
    Center: { templateColumns: "1fr 1fr 1fr", templateRows: "1fr 1fr 1fr", col: 2, row: 2, colSpan: 1, rowSpan: 1 },
    NextDisplay: { templateColumns: "1fr 1fr", templateRows: "1fr", col: 2, row: 1 },
    PreviousDisplay: { templateColumns: "1fr 1fr", templateRows: "1fr", col: 1, row: 1 },
  };
  const spec = specs[action];
  if (!spec) return null;
  const { templateColumns, templateRows, col, row, colSpan = 1, rowSpan = 1 } = spec;
  return (
    <span
      className="shortcut-tile shortcut-tile-grid"
      aria-hidden
      style={{
        position: "relative",
        display: "inline-block",
        width: size,
        height: size,
        boxSizing: "border-box",
      }}
    >
      <span
        className="shortcut-tile-grid-inner"
        style={{
          position: "absolute",
          top: 0.5,
          left: 1,
          right: 0.5,
          bottom: 1,
          display: "inline-grid",
          gridTemplateColumns: templateColumns,
          gridTemplateRows: templateRows,
          gap: 1,
        }}
      >
        <span
          className="shortcut-tile-window"
          style={{
            gridColumn: `${col} / span ${colSpan}`,
            gridRow: `${row} / span ${rowSpan}`,
          }}
        />
      </span>
    </span>
  );
}

function App() {
  const [config, setConfig] = useState<ConfigForFrontend | null>(null);
  const [savedConfig, setSavedConfig] = useState<ConfigForFrontend | null>(null);
  const [justSaved, setJustSaved] = useState(false);
  const [showQuitConfirm, setShowQuitConfirm] = useState(false);
  const [showRevertConfirm, setShowRevertConfirm] = useState(false);
  const [configPath, setConfigPath] = useState("");
  const configRef = useRef(config);
  const quitBtnRef = useRef<HTMLButtonElement>(null);
  const revertConfirmBtnRef = useRef<HTMLButtonElement>(null);
  const thirdsDropdownRef = useRef<HTMLDivElement>(null);
  configRef.current = config;

  const [thirdsDropdownOpen, setThirdsDropdownOpen] = useState(false);

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

  async function openConfigFileLocation() {
    try {
      await invoke("open_config_file_location");
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

  function openRevertConfirm() {
    setShowRevertConfirm(true);
  }

  async function confirmRevert() {
    setShowRevertConfirm(false);
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

  function minimizeToTray() {
    getCurrentWebviewWindow().hide();
  }

  function quit() {
    setShowQuitConfirm(true);
  }

  function confirmQuit() {
    setShowQuitConfirm(false);
    invoke("exit_app");
  }

  async function setThirdsLayout(value: "Thirds" | "Fifths") {
    if (!config) return;
    setThirdsDropdownOpen(false);
    const next = { ...config, thirdsLayout: value };
    setConfig(next);
    setSavedConfig(next);
    try {
      const c = await invoke<ConfigForFrontend>("save_config", { payload: next });
      setConfig(c);
      setSavedConfig(c);
    } catch (e) {
      console.error(e);
    }
  }

  useEffect(() => {
    if (!thirdsDropdownOpen) return;
    const onPointerDown = (e: PointerEvent) => {
      if (thirdsDropdownRef.current?.contains(e.target as Node)) return;
      setThirdsDropdownOpen(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    return () => document.removeEventListener("pointerdown", onPointerDown);
  }, [thirdsDropdownOpen]);

  useEffect(() => {
    if (!showQuitConfirm) return;
    const focusBtn = () => quitBtnRef.current?.focus();
    const id = requestAnimationFrame(focusBtn);
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        setShowQuitConfirm(false);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      cancelAnimationFrame(id);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [showQuitConfirm]);

  useEffect(() => {
    if (!showRevertConfirm) return;
    const focusBtn = () => revertConfirmBtnRef.current?.focus();
    const id = requestAnimationFrame(focusBtn);
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        setShowRevertConfirm(false);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      cancelAnimationFrame(id);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [showRevertConfirm]);

  return (
    <div className="app-root">
      {showQuitConfirm && (
        <div className="quit-overlay" role="dialog" aria-modal="true" aria-labelledby="quit-dialog-title">
          <div className="quit-dialog">
            <p id="quit-dialog-title" className="quit-dialog-title">Quit RectangleWin?</p>
            <div className="quit-dialog-actions">
              <button type="button" className="btn" onClick={() => setShowQuitConfirm(false)}>
                Cancel
              </button>
              <button ref={quitBtnRef} type="button" className="btn btn-danger" onClick={confirmQuit}>
                Quit
              </button>
            </div>
          </div>
        </div>
      )}
      {showRevertConfirm && (
        <div className="quit-overlay" role="dialog" aria-modal="true" aria-labelledby="revert-dialog-title">
          <div className="quit-dialog">
            <p id="revert-dialog-title" className="quit-dialog-title">Revert all settings to defaults?</p>
            <div className="quit-dialog-actions">
              <button type="button" className="btn" onClick={() => setShowRevertConfirm(false)}>
                Cancel
              </button>
              <button ref={revertConfirmBtnRef} type="button" className="btn btn-danger" onClick={confirmRevert}>
                Revert
              </button>
            </div>
          </div>
        </div>
      )}
      <header className="titlebar">
        <div className="titlebar-drag" data-tauri-drag-region>
          <img src={appIconUrl} alt="" className="titlebar-icon" />
          <span className="titlebar-title-text">RectangleWin</span>
        </div>
        <div className="titlebar-controls">
          <button
            type="button"
            className="titlebar-btn"
            onClick={minimizeToTray}
            title="Minimize to tray"
            aria-label="Minimize to tray"
          >
            _
          </button>
          <button
            type="button"
            className="titlebar-btn titlebar-btn-close"
            onClick={quit}
            title="Quit"
            aria-label="Quit"
          >
            X
          </button>
        </div>
      </header>
      <main className="scroll-main">
        <div className="container">
        <section className="card">
        <div className="card-header">
          <h2>Settings</h2>
          <div className="card-actions">
            <button
              type="button"
              className="icon-btn"
              onClick={openRevertConfirm}
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
                min={-5}
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
              {section.title === "Thirds" ? (
                <div className="section-dropdown" ref={thirdsDropdownRef}>
                  <button
                    type="button"
                    className="section-dropdown-trigger"
                    onClick={() => setThirdsDropdownOpen((o) => !o)}
                    aria-expanded={thirdsDropdownOpen}
                    aria-haspopup="listbox"
                    aria-label="Thirds layout"
                  >
                    <span>{config?.thirdsLayout === "Fifths" ? "Fifths" : "Thirds"}</span>
                    <span className={`section-dropdown-chevron ${thirdsDropdownOpen ? "open" : ""}`}>
                      <ChevronDownIcon />
                    </span>
                  </button>
                  {thirdsDropdownOpen && (
                    <div className="section-dropdown-menu" role="listbox">
                      <button
                        type="button"
                        className="section-dropdown-option"
                        role="option"
                        aria-selected={config?.thirdsLayout !== "Fifths"}
                        onClick={() => setThirdsLayout("Thirds")}
                      >
                        Thirds
                      </button>
                      <button
                        type="button"
                        className="section-dropdown-option"
                        role="option"
                        aria-selected={config?.thirdsLayout === "Fifths"}
                        onClick={() => setThirdsLayout("Fifths")}
                      >
                        Fifths
                      </button>
                    </div>
                  )}
                </div>
              ) : (
                <h3 className="section-title">{section.title}</h3>
              )}
              <ul className="shortcut-list">
                {section.actions.map((action) => {
                  const shortcut = actionToShortcut[action];
                  if (!shortcut) return null;
                  const label =
                    section.title === "Thirds"
                      ? thirdsSectionActionLabel(action, config?.thirdsLayout ?? "Thirds")
                      : action;
                  return (
                    <li key={action} className="shortcut-row">
                      <span className="shortcut-row-label">
                        <ShortcutTileIcon action={action} />
                        <span>{label}</span>
                      </span>
                      <span className="shortcut-key">{shortcut}</span>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </div>
      </section>

      <section className="card card-config">
        <p className="muted small">Edit hotkeys, gaps, and other settings in JSON:</p>
        <button type="button" className="link link-neutral" onClick={openConfig}>
          Open config.json
        </button>
        <button type="button" className="btn btn-neutral" onClick={reloadConfig}>
          Reload from config
        </button>
        {configPath && (
          <p className="muted small config-path-row" style={{ marginTop: 8 }}>
            <span>{configPath}</span>
            <button
              type="button"
              className="icon-btn icon-btn-sm"
              onClick={openConfigFileLocation}
              title="Open file location"
              aria-label="Open file location"
            >
              <OpenFolderIcon />
            </button>
          </p>
        )}
      </section>
        </div>
      </main>
    </div>
  );
}

export default App;
