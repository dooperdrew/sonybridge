# 05 — Roadmap & Milestone Tracker

> **Single Source of Truth**: Every milestone below maps 1:1 to a [GitHub Milestone](https://github.com/DooperDrew/sonybridge/milestones) and tracked [GitHub Issues](https://github.com/DooperDrew/sonybridge/issues).

---

## Milestone M0: Protocol & Windows Audio Spike (`v0.1.0` — Closed)
- [x] **M0.1**: Document Sony MDR v1/v2 RFCOMM framing (`0x3E`..`0x3C`, `0x3D` byte stuffing, checksums) and Windows Core Audio `IMMDeviceEnumerator` / `IPolicyConfig` routing (`2026-10-25`, `#1`)
- [x] **M0.2**: Build `sonybridge-cli probe` diagnostic tool to enumerate Bluetooth RFCOMM services and Windows audio endpoints (`2026-10-25`, `#1`)

## Milestone M1: Pure Core Protocol & State Machine (`v0.1.0` — Closed)
- [x] **M1.1**: Implement `crates/sonybridge-core` with `#![forbid(unsafe_code)]`, canonical + byte-stuffed wire codec, and unit tests (`2026-10-25`, `#2`)
- [x] **M1.2**: Implement deterministic `HeadphoneStateMachine` for Battery, ANC/Ambient, 5-Band EQ, and Clear Bass (`2026-10-25`, `#2`)

## Milestone M2: WinRT Bluetooth RFCOMM Transport (`v0.1.0` / `v0.2.0` — Closed)
- [x] **M2.1**: Implement `WinRtRfcommTransport` in `crates/sonybridge-windows/src/bluetooth.rs` using `RfcommDeviceService` and `StreamSocket` (`2026-10-25`, `#3`)

## Milestone M3: Windows Core Audio Endpoint Sanitizer (`v0.1.0` / `v0.2.0` — Closed)
- [x] **M3.1**: Enumerate active Windows render endpoints (`IMMDeviceEnumerator`) and classify Sony Stereo (`A2DP`) vs Hands-Free (`HFP`) (`2026-10-25`, `#4`)
- [x] **M3.2**: Implement `IPolicyConfig::SetDefaultEndpoint` in `AudioEndpointSanitizer::enforce_stereo_a2dp()` (`2026-10-25`, `#4`)

## Milestone M4: Tauri v2 System Tray & IPC Bridge (`v0.1.0` — Closed)
- [x] **M4.1**: Implement Tauri v2 system tray icon, context menu (`Open`, `Connect`, `Disconnect`, `Quit`), and single-instance mutex (`2026-10-25`, `#5`)

## Milestone M5: ANC, Ambient Sound & 5-Band Equalizer UI (`v0.1.0` / `v0.2.0` — Closed)
- [x] **M5.1**: Build glassmorphic dark UI for ANC/Wind/Ambient (1–20), Voice Passthrough, Clear Bass (`-10..+10`), and 5-band Equalizer (`2026-10-25`, `#6`, `#12`)
- [x] **M5.2**: Add 1-click Sound Profiles (*Focus*, *Office / Calls*, *Cinema / Bass Boost*, *Studio Reference*) and persistent `AppConfig` (`2026-10-25`, `#13`)

## Milestone M6: Bilingual Localization (`EN` / `IT`) & First-Run Onboarding (`v0.2.0` — Closed)
- [x] **M6.1**: Implement `crates/sonybridge-core/src/i18n.rs` and `locales/en.toml` + `locales/it.toml` with live UI language switching (`2026-10-25`, `#7`, `#14`)
- [x] **M6.2**: Build First-Run Onboarding Wizard modal and interactive `setup.cmd` / `setup-wizard.ps1` (`2026-10-25`, `#14`)

## Milestone M7: Registry Autostart & Power Controls (`v0.2.0` — Closed)
- [x] **M7.1**: Implement `AutostartManager` (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run\SonyBridge`), Auto-Power Off, and Touch Sensor lock (`2026-10-25`, `#8`)

## Milestone M8: Release Packaging, Portable ZIP & Package Manifests (`v0.1.0` / `v0.2.0` — Closed)
- [x] **M8.1**: Build `scripts/package-release.ps1` and `scripts/generate-manifests.ps1` to produce `SonyBridge-v0.1.0-windows-x64.zip` and `SonyBridge-v0.2.0-windows-x64.zip` plus Scoop/WinGet manifests (`2026-10-25`, `#9`)

## Milestone M9: Empirical Physical Hardware QA on Sony WH-1000XM3 (`Open` — Pending User Hardware Test)
- [ ] **M9.1**: Empirical hardware validation on physical Sony WH-1000XM3 headphones (`sonybridge-cli.exe probe`, live battery, ANC/Ambient transitions, 5-band EQ & Clear Bass, A2DP enforcement) (`Pending`, `#10`)
- [ ] **M9.2**: Power states, Windows sleep/resume, and Bluetooth out-of-range reconnection verification (`Pending`, `#11`)

## Milestone M10: Autonomous UI/UX & Accessibility Hardening (`v0.2.0` — Closed)
- [x] **M10.1**: Eliminate layout clipping on Architecture tab and viewport (`VIS-CLIP-005`) (`2026-10-27`, `#16`)
- [x] **M10.2**: Add explicit accessible names and ARIA labels to sliders, presets, and checkboxes (`A11Y-NAME-000`) (`2026-10-27`, `#17`)
- [x] **M10.3**: Fix First-Run Wizard button responsiveness, modal dismiss triggers, and MDR wire frame emission (`UX-DEAD-BTN-013`) (`2026-10-27`, `#18`)
- [x] **M10.4**: Elevate color contrast across inactive tabs, secondary buttons, and chips for WCAG AA conformance (`VIS-CONTRAST-007`) (`2026-10-27`, `#19`)

## Milestone M11: Multi-Model Expansion & Community Ecosystem (`v0.2.0` — Closed)
- [x] **M11.1**: Document multi-agent engineering methodology, UX evolution, and community packaging strategy (`2026-10-25`, `#15`)

## Milestone M12: Single Unified Executable, Dynamic Tray Telemetry & Setup Ergonomics (`v0.3.0` — Closed)
- [x] **M12.1**: Unified Single-Binary CLI Dispatch: embed diagnostic probe (`probe`), A2DP enforcement (`--enforce-a2dp`), and setup/uninstall flags into `sonybridge-app.exe` with console attachment (`2026-10-31`)
- [x] **M12.2**: Dynamic System Tray Telemetry: real-time `main_tray` tooltip updates on ANC/ambient mode transitions, sound profile switches, and battery telemetry (`2026-10-31`)
- [x] **M12.3**: In-App Maintenance & Guided Tools: added Re-run Wizard, Run Probe, and Clean Uninstall triggers inside the Windows & Power settings panel (`2026-10-31`)
- [x] **M12.4**: Single-Binary Distribution & Packaging Overhaul: updated `scripts/package-release.ps1` to produce standalone installer (`XMCompanion-Setup.exe`) and single-binary clean portable archive (`XMCompanion-v0.3.0-portable.zip`) (`2026-10-31`)
- [x] **M12.5**: GitHub Governance & Spec Synchronization: added `.github/ISSUE_TEMPLATE/qa_verification.yml`, aligned release workflows, and authored `docs/retrospective.md` (`2026-10-31`)
- [x] **M12.6**: Repository Rebranding & GitHub Discoverability: adapted narrative, UI, desktop window, tray, AppData paths, setup wizard, and packaging to `XMCompanion`; vendored scripts via `.gitattributes` to restore Rust as #1 language (63.2%) (`2026-11-02`)
