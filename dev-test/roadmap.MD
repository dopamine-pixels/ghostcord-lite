# Ghostcord Lite -> Vesktop Parity Roadmap

This roadmap targets feature parity with Vesktop while keeping the current Tauri app usable throughout.

## Goals
- Ship a stable Discord wrapper with Vencord injection, persistent settings, and basic OS integrations.
- Reach cross-platform parity where feasible; track hard gaps explicitly.
- Keep work incremental with testable milestones.

## Scope Assumptions
- Shell: Tauri 2.x with WRY.
- Target platforms: Windows first, then Linux/macOS.
- Screenshare custom picker and venmic are treated as advanced milestones.

## Milestone 0: Baseline Cleanup (1-2 days)
- Refactor `src-tauri/src/main.rs` into modules: `app`, `windows`, `settings`, `ipc`.
- Move the large injection script into a dedicated file and load it at runtime.
- Add logging and configuration paths in one place.

Deliverables
- `src-tauri/src/app.rs` bootstraps the app.
- `src-tauri/src/windows.rs` creates windows and lifecycle hooks.
- `src-tauri/src/injection/ghostcord_init.js` or similar file.

## Milestone 1: Settings + State System (3-5 days)
- Add a SettingsStore in Rust with change listeners.
- Persist settings and state JSON under app data.
- Wire settings to runtime updates (toggle perf CSS, theme CSS, blockers).

Deliverables
- `src-tauri/src/settings.rs` (settings + change listeners).
- `src-tauri/src/state.rs` (window bounds, last state, updater snooze).
- Commands: get/set settings, notify renderer on change.

## Milestone 2: Vencord Loader + Injection (5-8 days)
- Implement Vencord download cache (GitHub API + assets).
- Store Vencord files under a data directory.
- Inject Vencord preload and renderer scripts into the webview.
- Add a settings toggle to enable/disable Vencord loading.

Deliverables
- `src-tauri/src/vencord.rs` download + cache.
- Injection logic to eval JS/CSS on page load.
- Config UI toggle connected to loader.

## Milestone 3: App Integrations (5-8 days)
- System tray and minimize-to-tray.
- Autostart (Windows/macOS via plugin; Linux desktop entry).
- Protocol handling for `discord://` and app deep links.

Deliverables
- `src-tauri/src/tray.rs`
- `src-tauri/src/autostart.rs`
- Protocol registration and handling.

## Milestone 4: Assets + Static Views (3-6 days)
- Custom protocol for `vesktop://static` and user assets.
- Add a splash screen window with custom user assets.
- Implement an updater window using local HTML (static views).

Deliverables
- `src-tauri/src/protocols.rs` for static + user assets.
- `src-tauri/src/splash.rs`
- `static/views/*` with updater UI.

## Milestone 5: Updater (3-5 days)
- Integrate Tauri updater.
- Provide snooze/ignore behavior in settings/state.
- Wire update progress to the updater window UI.

Deliverables
- `src-tauri/src/updater.rs`
- Updated config state for updater choices.

## Milestone 6: Platform Polish (5-10 days)
- Badge count / attention support.
- Media permissions prompts (macOS).
- Custom user agent and chromium flags parity where possible.

Deliverables
- `src-tauri/src/badge.rs`
- Platform specific permission handling.

## Milestone 7: Advanced Features (R&D)
- Screenshare picker parity (Electron has desktopCapturer; Tauri does not).
- venmic (PipeWire routing) and arRPC equivalent.

Deliverables
- Prototype screen capture UX decisions.
- Decide on sidecar vs Rust crate for audio routing.

## Risks / Gaps
- Custom screenshare picker may not be feasible without a native bridge.
- venmic parity is complex and likely requires a dedicated audio pipeline.
- Electron features do not map 1:1 to Tauri APIs.

## Suggested Order (Minimum Viable Parity)
1. Milestone 1 (Settings + State)
2. Milestone 2 (Vencord loader)
3. Milestone 3 (Tray + Autostart)
4. Milestone 4 (Assets + Splash)
5. Milestone 5 (Updater)

## Notes
- Keep JS injection code small and avoid heavy DOM scraping where possible.
- Prefer async Rust APIs to avoid UI hangs.
- Build in feature flags so parity can be staged per platform.
