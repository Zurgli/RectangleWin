# RectangleWin

<img src="app/src-tauri/icons/128x128.png" width="128" height="128" alt="RectangleWin icon" />

A **Windows port** of [Rectangle](https://github.com/rxhanson/Rectangle) — the keyboard-driven window tiling manager for macOS.

Full credit goes to **Ryan Hanson** and the original Rectangle app. I loved using it on macOS so much that I wanted the same experience on Windows. I had used FancyZones (PowerToys) before, but Rectangle's keyboard shortcuts are much more convenient: halves, quarters, thirds, maximize, center, restore, and move between displays, all from the keyboard without touching the mouse. This project aims to replicate that workflow on Windows.

RectangleWin is a minimal tray app (Rust + Tauri): global hotkeys (Win+Alt by default), optional launch at startup, and configurable gaps. Edit `config.json` to change shortcuts.

---

## Requirements

- Windows 10/11
- [Rust](https://rustup.rs/) toolchain (use the default `x86_64-pc-windows-msvc` target)
- [Node.js](https://nodejs.org/) (and npm or pnpm) for the frontend
- **Visual Studio Build Tools 2022** (recommended) or Visual Studio with the **“Desktop development with C++”** workload. The Rust MSVC target needs:
  - MSVC x64/x86 build tools
  - Windows 10/11 SDK
  - a working Developer shell environment

## Environment setup

The most reliable local setup on Windows is to use the VS 2022 Build Tools developer shell. This repo includes a helper that finds a usable installation and loads the right `PATH`, `INCLUDE`, and `LIB` values for the current PowerShell session.

From the repo root:

```powershell
. .\scripts\enter-vsdevshell.ps1
cd app\src-tauri
cargo test
```

If that script cannot find a usable toolchain, install or repair:

- Visual Studio Build Tools 2022
- Desktop development with C++
- MSVC v143 x64/x86 build tools
- Windows 10/11 SDK

### Making sure build tools are in PATH

The build uses the MSVC toolchain (`cl.exe`, Windows headers). Those are only available in a **Developer** environment. Use either:

1. **Start menu** → open **“x64 Native Tools Command Prompt for VS 2022”** (or your VS version), then run the build commands in that terminal, or  
2. **In a normal terminal**, run the VS dev script first, then build:
   ```powershell
   . .\scripts\enter-vsdevshell.ps1
   cd app
   npm run tauri build
   ```
   If VS is in a non-default path, pass `-PreferredInstallPath` to the helper script or use the “Developer PowerShell for VS” shortcut.

## Validation

Run both the frontend and backend checks:

```powershell
pwsh -File .\scripts\check.ps1
```

Run them separately:

```powershell
pwsh -File .\scripts\check.ps1 -Frontend
pwsh -File .\scripts\check.ps1 -Backend
```

## Coverage

Backend coverage uses `cargo-llvm-cov` and the same VS developer-shell bootstrap as the test workflow.

One-time prerequisites:

```powershell
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

Or let the helper install them:

```powershell
pwsh -File .\scripts\coverage.ps1 -InstallPrereqs
```

Generate an HTML report:

```powershell
pwsh -File .\scripts\coverage.ps1
```

The report is written to `app/src-tauri/target/llvm-cov-html/html/index.html`.

Generate LCOV output instead:

```powershell
pwsh -File .\scripts\coverage.ps1 -Lcov
```

The LCOV file is written to `app/src-tauri/target/llvm-cov.info`.

## Git hooks

Install the repo-local pre-commit hook:

```powershell
pwsh -File .\scripts\install-git-hooks.ps1
```

The hook runs targeted checks based on staged files:

- frontend changes under `app/` trigger `npm run build`
- Rust changes under `app/src-tauri/` trigger `cargo test`
- docs-only changes skip app checks

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
