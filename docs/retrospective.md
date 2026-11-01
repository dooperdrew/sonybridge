# SonyBridge — Retrospective (M0 → v0.3.0)

> **Date:** October 2026 · **Scope:** Milestones M0–M12, releases v0.1.0 / v0.2.0 / v0.3.0.  
> **Repository:** [https://github.com/DooperDrew/sonybridge](https://github.com/DooperDrew/sonybridge)  
> **Method:** Transcript mining + git history + documentation review across agent sessions.  
> **Generic Lessons:** Promoted to the `project-kickstart` plugin and `docs/lessons-learned.md`.

---

## 1. Timeline & Milestone Execution

| Milestone | Scope & Deliverables | Primary Deliverable |
|---|---|---|
| **M0** | Hardware protocol & Windows audio spike (`sonybridge-cli probe`) | Discovered MDR v1 RFCOMM framing (`0x3E`..`0x3C`), checksum modulo-256, and Core Audio COM `IPolicyConfig` |
| **M1** | Pure Core Protocol & Deterministic State Machine | `crates/sonybridge-core` with `#![forbid(unsafe_code)]`, canonical framing, battery inquiry, and 5-band EQ encoder |
| **M2** | WinRT Bluetooth RFCOMM / StreamSocket Transport | `crates/sonybridge-windows/src/bluetooth.rs` via `Windows::Devices::Bluetooth::Rfcomm` |
| **M3** | Windows Core Audio Endpoint Sanitizer | `AudioEndpointSanitizer` enforcing high-res Stereo (`A2DP`) over muffled Hands-Free (`HFP`) |
| **M4** | Tauri v2 System Tray & IPC Bridge | Glassmorphic desktop companion with tray icon, flyout, and single-instance mutex |
| **M5** | ANC, Ambient Sound & 5-Band Equalizer UI | 3-tab responsive UI, Clear Bass (`-10..+10`), and 1-click sound profiles |
| **M6** | Bilingual Localization (EN / IT) & Guided Setup Wizard | `locales/{en,it}.toml`, `i18n.rs`, and Windows Setup Wizard (`setup-wizard.ps1`) |
| **M7** | Registry Autostart & Power Controls | Per-user `HKCU\...\Run` autostart toggle, touch sensor lock, and auto-off timer |
| **M8** | Release Packaging, Portable ZIP & Package Manifests | `scripts/package-release.ps1` producing portable `.zip`, WinGet, and Scoop manifests |
| **M9** | Empirical Physical Hardware QA (WH-1000XM3) | Test matrix with diagnostic probe, battery telemetry, and audio routing (`docs/06-qa-and-validation.md`) |
| **M10** | Autonomous UI/UX & Accessibility QA (`ui-qa-tester`) | 4 QA cycles, 15 states crawled, 0 defects; fixed clipping, ARIA labels, contrast, and dead button |
| **M11** | Multi-Model Expansion & Cloud Copilot Agent Workflow | `.github/copilot-instructions.md`, `agent_task.yml`, and collaborative cloud methodology |
| **M12** | Single Unified Executable, Dynamic Tray Telemetry & Setup Ergonomics | Unified CLI/GUI binary dispatch, live tray tooltip updates, in-app maintenance buttons, and `SonyBridge-Setup.exe` |

---

## 2. What Worked Well

1. **Pure Core Isolation (`crates/sonybridge-core`):**
   - Kept packet framing, byte-stuffing, checksum verification, and state machine transitions strictly in a pure Rust crate with zero Win32 or Tauri dependencies.
   - Allowed instantaneous unit testing (14 tests in < 0.01s) without spinning up OS sockets or COM runtimes.
2. **Autonomous UI/UX & Hardware QA Loop (`ui-qa-tester`):**
   - Running `ui-qa-cli` headlessly against `sonybridge-app.exe` automatically caught layout clipping on high-DPI viewports, missing accessibility names on sliders, and low contrast on secondary buttons.
   - Closing the loop (detect &rarr; issue &rarr; code fix &rarr; re-verify) eliminated 30 defects down to 0 findings.
3. **Smart Audio Routing (`AudioEndpointSanitizer`):**
   - Solved the notorious Windows Bluetooth hands-free audio bug by switching `eConsole`, `eMultimedia`, and `eCommunications` default render endpoints to Stereo A2DP via `IPolicyConfig`.
4. **Single-Binary & Progressive Setup Ergonomics:**
   - Unifying CLI diagnostic probe (`sonybridge-app probe`) and companion GUI into a single executable removed multi-binary confusion for end users.
   - 4-channel installation strategy (WinGet, Scoop, Standalone Setup Wizard, Portable ZIP) ensured accessible distribution.

---

## 3. What Went Wrong & Lessons Learned

| # | What Happened | Root Cause | Adopted Rule / Permanent Fix |
|---|---|---|---|
| 1 | **Tauri / WebView2 Child HWND Input Failure:** Mouse clicks sent to the outer frame did not activate DOM buttons in `ui-qa-tester`. | WebView2 renders inside a child `Chrome_RenderWidgetHostHWND` with its own coordinate space. | Walk child HWNDs up to depth 20, pause 300ms for UIA hydration, and translate coordinates using `ScreenToClient(render_hwnd)`. |
| 2 | **PowerShell Corrupted Non-ASCII UTF-8 Output:** Piping `gh api` or CLI output through standard PowerShell 5 stdout mangled symbols. | PowerShell 5 defaults to ANSI / UTF-16 console encoding. | Always force UTF-8 without BOM (`[System.IO.File]::WriteAllText(..., [System.Text.UTF8Encoding]::new($false))`) and set `[Console]::OutputEncoding = UTF8`. |
| 3 | **Multiple Executables Confused Users:** Distributing separate `sonybridge-app.exe` and `sonybridge-cli.exe` binaries raised questions about which binary to run. | Architecture split CLI diagnostic spike from GUI application. | Single-binary dispatch (`sonybridge-app probe` / `--enforce-a2dp` / `--setup`) with `AttachConsole` so the primary executable handles both CLI and GUI seamlessly. |
| 4 | **Static Tray Icon Tooltip:** System tray icon tooltip remained hardcoded to initial placeholder text. | State changes in Tauri commands did not broadcast back to `TrayIcon`. | Created tray with `with_id("main_tray")` and dynamically updated tooltips on ANC/battery updates. |
| 5 | **Missing In-App Maintenance & Uninstall:** Users had no way within the GUI to re-launch the setup wizard, re-create shortcuts, or trigger clean uninstallation. | Maintenance was only available via standalone PowerShell scripts. | Added dedicated maintenance controls in the "Windows & Power" tab and wired Tauri commands to launch the scripts. |
| 6 | **Tauri v2 Silent Window Unmapped on Launch:** Launching `sonybridge-app.exe` started a silent process with HWND `0` that never appeared on screen. | Tauri v2 defaults to hidden/unmapped windows when tray icons are configured unless explicitly surfaced. | Call `window.show()`, `window.unminimize()`, `window.center()`, and `window.set_focus()` in `app.setup()`, and set `"visible": true`, `"center": true`, `"focus": true` in `tauri.conf.json`. |
| 7 | **Setup Script Relative Path Failure & Mojibake:** `setup-wizard.ps1` displayed `â€¢` and `v0.2.0` when run from `dist/v0.3.0/`. | PowerShell 5.1 interpreted UTF-8 bullets without BOM as ANSI 1252, and relative check `..\Cargo.toml` failed in release subfolders. | Enforce clean ASCII bullets (`*` / `-`) or `[char]0x2022`, and resolve version strings dynamically across multiple parent directories and folder name patterns. |
| 8 | **Deceptive Static Frontend State:** App opened claiming `Connected` (85% battery) even when headphones were disconnected or off. | Frontend markup statically contained positive placeholder values instead of neutral/querying states. | Markup defaults to `Checking...` with `--%` battery; app queries live WinRT/audio status on startup and on an active interval (5s). |
| 9 | **Dual Bluetooth Profile Confusion (BLE vs Classic Audio):** Windows paired only `LE_WH-1000XM3` (Swift Pair beacon), leading to no sound, no audio sink, and failed RFCOMM. | Sony headphones expose both BLE beacon and Classic Bluetooth Audio under separate device IDs. | Diagnostic layer inspects both Classic Bluetooth and BLE; flags `BLE Only (Audio Disconnected)` and provides 1-click native pairing wizard triggers (`DevicePairingWizard.exe`, `ms-settings-connectabledevices:devicediscovery`). |
| 10 | **Post-Install Pairing Gap:** Completing installation left users stranded if headphones were not previously paired in Windows. | Setup wizard only offered launching the application without pairing onboarding. | Integrated "Launch Windows Pairing Wizard Now" button and pre-checked "Launch Guided Headphone Pairing Assistant on startup" (`--pair`) in setup wizard completion screen. |

---

## 4. User Feedback & Evolution

- *"i'm ready to test walk me through what you want me to do"* &rarr; Codified Step-by-Step Hardware QA walkthrough and added `.github/ISSUE_TEMPLATE/qa_verification.yml` for empirical test submissions.
- *"we made some improvements to the overall projects agents etc, can you check any change and implement all the improvements? take example to the magicbridge repo"* &rarr; Ported Single-Binary CLI dispatch, standalone installer staging (`SonyBridge-Setup.exe`), dynamic tray telemetry, and in-app maintenance actions.
- *"if I try to run the exe nothing is happening, can you double check? can you ensure any release has is own separate folder with all the dist files?"* &rarr; Fixed Tauri v2 window startup visibility (`show`, `unminimize`, `center`, `focus`), hardened setup mode script fallthrough, and isolated releases into dedicated per-version folders (`dist/v<Version>/`).
- *"showed connected even if the headphones at the time were not connected / the headphones are not working, not reproducing the audio and not showing off as an option / can't the application help me to pair the headphones correctly?"* &rarr; Diagnosed `LE_WH-1000XM3` vs Classic Audio disparity, eliminated static frontend mock states, added live BLE/Classic connection diagnostics, and built an interactive Guided Headphone Pairing Assistant integrated into both the application and the setup wizard completion screen.
