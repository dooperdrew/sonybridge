# Changelog

All notable changes to **SonyBridge** are documented in this file.

## [0.3.0] - 2026-11-02

### Added & Rebranded (Milestone M12 & Rebranding: XMCompanion)
- **Complete Rebranding to XMCompanion**:
  - Rebranded application from `SonyBridge` to `XMCompanion` across frontend UI, desktop window title, system tray items, autostart registry entries, AppData config paths, installer wizard (`setup-wizard.ps1`), packaging scripts (`package-release.ps1`), locales (`en.toml`, `it.toml`), documentation, and README.
  - Added `.gitattributes` vendoring rule for `scripts/**` and `packaging/**` ensuring GitHub Linguist recognizes Rust as the primary language (63.2%).
  - Expanded GitHub topics and tags for maximum discoverability.
  - Provided backwards-compatible migration for configuration files (`%APPDATA%\XMCompanion` with fallback to `SonyBridge`) and autostart registry entries.
- **Guided Headphone Pairing Assistant (App & Setup Wizard)**:
  - Added an interactive 3-step pairing modal in the companion app with 1-click native triggers for `DevicePairingWizard.exe` and `ms-settings-connectabledevices:devicediscovery`, real-time connection watching, and automatic link confirmation.
  - Integrated a direct "Launch Windows Pairing Wizard Now" action and pre-checked "Launch Guided Headphone Pairing Assistant on startup" (`--pair`) into the setup wizard completion screen.
- **Dual-Bluetooth Profile Diagnostics (BLE Swift Pair vs Classic Audio)**:
  - Implemented dual-profile detection in `WindowsBluetoothManager` differentiating `LE_WH-1000XM3` (low-energy beacon without audio/RFCOMM) from `WH-1000XM3` (Classic Audio A2DP/HFP + MDR).
  - Added real-time diagnostics telemetry, amber `BLE Only (Audio Disconnected)` badge, in-app troubleshooting banner, and 1-click Windows Bluetooth/Sound settings triggers.
- **Connection Truth & Live Telemetry Polling**:
  - Eliminated static mocked connection states (`Connected`, `85%`, dummy packet logs); UI defaults to neutral querying state and polls live WinRT/audio status on load and on an active interval.
- **Setup Script Resilience & Encoding Hardening**:
  - Replaced UTF-8 bullet characters with clean ASCII to eliminate PowerShell ANSI 1252 mojibake (`â€¢`), and enhanced version detection to dynamically inspect directory hierarchies and folder naming patterns.
- **Startup Window Guarantee & Setup Fallback**:
  - Ensured `sonybridge-app.exe` immediately surfaces, centers, and focuses the main desktop window on startup in Tauri v2 rather than remaining hidden in the system tray.
  - Hardened `SonyBridge-Setup.exe` with automatic fallback to the companion application GUI and onboarding wizard if external PowerShell scripts are absent.
- **Isolated Per-Version Distribution Staging**:
  - Enhanced `scripts/package-release.ps1` to produce a clean, isolated `dist/v<Version>/` directory containing all standalone executables, scripts, locales, documentation, and archive bundles for instant testing and distribution.
- **Single-Binary CLI & Probe Unification (`sonybridge-app`)**:
  - Embedded diagnostic hardware probe (`probe`), A2DP audio routing (`--enforce-a2dp`), and setup/uninstallation switches directly into `sonybridge-app.exe` with parent console attachment (`AttachConsole`).
  - Running without arguments launches the glassmorphic desktop companion as usual.
- **Dynamic System Tray Telemetry**:
  - Registered unique tray ID `main_tray` and added real-time live tooltip updates on ANC/ambient mode transitions, sound profile switches, and battery telemetry.
- **In-App Maintenance & Guided Tools**:
  - Added dedicated controls in the "Windows & Power" tab to re-run the guided onboarding wizard (`#btn-reopen-wizard`), run the terminal diagnostic probe (`#btn-run-probe`), and cleanly uninstall SonyBridge (`#btn-uninstall-app`).
- **Release Packaging Overhaul (`scripts/package-release.ps1`)**:
  - Produces a single standalone installer (`SonyBridge-Setup.exe`) and single-binary clean portable archive (`SonyBridge-v0.3.0-portable.zip`) alongside WinGet and Scoop manifests.
- **Empirical QA & Retrospective Documentation**:
  - Added `.github/ISSUE_TEMPLATE/qa_verification.yml` for structured physical hardware test submissions.
  - Authored comprehensive milestone retrospective in `docs/retrospective.md`.

### Hardened & Fixed (Autonomous UI/UX & Accessibility QA via `ui-qa-tester`)
- **Accessibility & Contrast Conformance**:
  - Added explicit `aria-label` and `title` attributes to all sliders, preset selectors, equalizer bands, and toggles (`#17`).
  - Elevated contrast ratios across inactive tabs, secondary buttons, chips, and segmented controls for WCAG AA compliance (`#19`).
- **Layout & Interaction Polish**:
  - Resolved layout clipping on Architecture tab cards and footer by introducing custom scrollable viewports and adjusting window dimensions (`#16`).
  - Fixed First-Run Wizard button responsiveness, modal dismiss handling (close button, backdrop click, Escape key), and live MDR sequence string emission (`#18`).
- **Autonomous QA Harness**:
  - Integrated `qa-session.toml` and verified 0 defects across 15 crawl steps and 75 UIA elements using `ui-qa-tester`.

## [0.2.0] - 2026-10-25

### Added
- **Real Sony WH-1000XM3 MDR v1 Wire Protocol (`crates/sonybridge-core`)**:
  - Added authentic Sony MDR v1 byte-stuffed framing (`0x3E` start marker, `0x3C` end marker, `0x3D` escape byte with `0x10` mask, `0x0C` `DATA_MDR`, `0x01` `ACK`, and full header+payload modulo-256 checksum).
  - Added physical WH-1000XM3 MDR v1 command encoders (`0x68` `NCASM_SET_PARAM` for Dual NC, Wind Noise Reduction, and 20-step Ambient Sound + Voice Passthrough; `0x58` `EQEBB_SET_PARAM` for Clear Bass `-10..+10` and 5-band Equalizer; `0x12`/`0x13` Battery inquiry/reply; `0x28` Auto-Power Off; `0xD8` Touch Panel control).
- **Live WinRT RFCOMM Socket & Core Audio `IPolicyConfig` Routing (`crates/sonybridge-windows`)**:
  - Connected `WinRtRfcommTransport` over `Windows::Devices::Bluetooth::Rfcomm::RfcommDeviceService` and `StreamSocket` (`DataWriter`/`DataReader`) targeting Sony MDR v1 (`96cc203e-5068-46ad-b32d-396ceebe3928`), MDR v2 (`956c7b26-d49a-4ba8-b03f-b17d393cb6e2`), and SPP (`00001101-0000-1000-8000-00805f9b34fb`).
  - Implemented `AudioEndpointSanitizer::enforce_stereo_a2dp()` using the Windows Core Audio `IPolicyConfig` COM vtable to switch default playback across `eConsole`, `eMultimedia`, and `eCommunications` away from muffled `Hands-Free AG Audio` to high-res Stereo (`A2DP`).
  - Added `AutostartManager` for per-user `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\SonyBridge` registration.
- **3-Tab Adaptive UI, 1-Click Sound Profiles & First-Run Onboarding (`crates/sonybridge-app`)**:
  - Redesigned the tray flyout into 3 tabs (`Dashboard & EQ`, `Windows & Power`, `Architecture & Wire Inspector`).
  - Added 4 built-in 1-click Sound Profiles (*Focus (Max ANC)*, *Office / Calls*, *Cinema / Bass Boost*, *Studio Reference*).
  - Added interactive First-Run Onboarding Wizard modal and bilingual English/Italian (`EN`/`IT`) localization switcher.
  - Added live hexadecimal wire packet telemetry inspector.
- **Zero-Build Portable Release Packaging & Setup Wizard (`scripts/`, `packaging/`)**:
  - Added `scripts/package-release.ps1`, `scripts/setup.cmd`, `scripts/setup-wizard.ps1`, `scripts/install.ps1`, `scripts/uninstall.ps1`, and `scripts/generate-manifests.ps1` (Scoop & WinGet manifests).

## [0.1.0] - 2026-10-25

### Added
- Initial Rust Cargo workspace (`sonybridge-core`, `sonybridge-windows`, `sonybridge-cli`, `sonybridge-app`).
- Canonical MDR packet framing, state machine, mock transport, Core Audio endpoint enumeration, and Tauri v2 system tray application.
- Full specification suite (`docs/01-vision.md` through `docs/06-qa-and-validation.md`, `AGENTS.md`).
