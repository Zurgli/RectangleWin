# RectangleWin

<img src="app/src-tauri/icons/128x128.png" width="128" height="128" alt="RectangleWin icon" />

A **Windows port** of [Rectangle](https://github.com/rxhanson/Rectangle) — the keyboard-driven window tiling manager for macOS.

Full credit goes to **Ryan Hanson** and the original Rectangle app. I loved using it on macOS so much that I wanted the same experience on Windows. I had used FancyZones (PowerToys) before, but Rectangle's keyboard shortcuts are much more convenient: halves, quarters, thirds, maximize, center, restore, and move between displays, all from the keyboard without touching the mouse. This project aims to replicate that workflow on Windows.

RectangleWin is a minimal tray app (Rust + Tauri): global hotkeys (Win+Alt by default), optional launch at startup, and configurable gaps. Edit `config.json` to change shortcuts.

---

## Requirements

- Windows 10/11
- [Rust](https://rustup.rs/) toolchain
- [Node.js](https://nodejs.org/) (and npm or pnpm) for the frontend

## Build & run

From the repo root:

```bash
cd app
npm install
npm run tauri dev
```

Production build:

```bash
cd app
npm run tauri build
```

The built executable and installers are in `app/src-tauri/target/release/` (e.g. `app.exe`) and `app/src-tauri/target/release/bundle/` (NSIS and MSI installers).

## Installers

After running `npm run tauri build` in `app/`, installers are produced in `app/src-tauri/target/release/bundle/` (e.g. NSIS `.exe` and MSI). Use whichever you prefer for distribution.

## Config

Config path: `%LocalAppData%\RectangleWin\config.json`. You can change hotkeys (default: Win+Alt + key), gap size, and launch-at-startup. Restart the app after editing.

## Original project

- **Rectangle (macOS):** [github.com/rxhanson/Rectangle](https://github.com/rxhanson/Rectangle) — window management app based on Spectacle, by Ryan Hanson.

## License

MIT. See [LICENSE](LICENSE). This project is a Windows port inspired by Rectangle; Rectangle is Copyright (c) 2019–2025 Ryan Hanson (based on Spectacle, Copyright (c) 2017 Eric Czarny).
