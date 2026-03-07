# Changelog

## 0.2.2 - 2026-03-07

- Fixed shell hook handling so Start menu and other shell UI are no longer affected by stale snap state or invalid foreground targets.
- Cleared tracked snap state when windows are destroyed to prevent HWND reuse from affecting unrelated windows.
- Tightened the keyboard hook so snap hotkeys are only consumed when a real target window exists.

## 0.2.1 - 2026-03-07

- Fixed window targeting so snap actions ignore shell UI like the Start menu, tray popups, and tool windows.
- Added backend tests for snap-target filtering to prevent regressions.
- Included current 0.2.x improvements in the release branch, including start-hidden tray behavior and the repo validation scripts.

## 0.2.0

- Added the hidden tray startup flow for the main app window.
- Added frontend and backend validation scripts, tests, and coverage helpers.
