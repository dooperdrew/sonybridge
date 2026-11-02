# XMCompanion User Guide 🎧 🌉
*(formerly SonyBridge)*

**Comprehensive User Manual, Installation Guide & Hardware Testing Reference for Sony Bluetooth Headphones (`WH-1000XM3`, `WH-1000XM4`, `WH-1000XM5`, and `WF` series) on Windows 10 & 11**

---

## Table of Contents
1. [Introduction & Overview](#1-introduction--overview)
2. [Quick Start & Installation (4 Channels)](#2-quick-start--installation-4-channels)
3. [Pairing & Connecting Your Sony Headphones](#3-pairing--connecting-your-sony-headphones)
4. [First-Run Onboarding Wizard & Bilingual UI (EN / IT)](#4-first-run-onboarding-wizard--bilingual-ui-en--it)
5. [System Tray & 3-Tab Adaptive Flyout Controls](#5-system-tray--3-tab-adaptive-flyout-controls)
6. [1-Click Sound Profiles](#6-1-click-sound-profiles)
7. [Active Noise Canceling (ANC), Wind Reduction & Ambient Sound](#7-active-noise-canceling-anc-wind-reduction--ambient-sound)
8. [Hardware 5-Band Equalizer & Clear Bass](#8-hardware-5-band-equalizer--clear-bass)
9. [Smart Windows Stereo (A2DP) Audio Routing & Power Controls](#9-smart-windows-stereo-a2dp-audio-routing--power-controls)
10. [Live Wire Frame Inspector & Headless CLI Diagnostics](#10-live-wire-frame-inspector--headless-cli-diagnostics)
11. [Hardware QA Testing Procedure (WH-1000XM3)](#11-hardware-qa-testing-procedure-wh-1000xm3)
12. [Troubleshooting](#12-troubleshooting)
13. [Uninstallation](#13-uninstallation)

---

## 1. Introduction & Overview

**XMCompanion** is a native, ultra-lightweight Windows 10 and 11 desktop companion and system tray utility built specifically for Sony wireless headphones (**WH-1000XM3**, **WH-1000XM4**, **WH-1000XM5**, and **WF-1000XM4/XM5**).

When connected to Windows, Sony Bluetooth headphones normally suffer from several frustrating limitations:
- Windows automatically switches audio into the muffled 8 kHz mono **`Hands-Free AG Audio` (`HFP`)** telephony profile whenever an application accesses the microphone.
- Sony does not offer a Windows version of the mobile *Sony | Headphones Connect* app, leaving you unable to adjust Ambient Sound (`1..20`), Wind Noise Reduction, Focus on Voice, Clear Bass, or the 5-band hardware DSP Equalizer from your PC.
- Windows Bluetooth settings provide coarse battery indicators and no visibility into active ANC/EQ states.

XMCompanion fixes all of these problems while maintaining strict system integrity:
- **0% Idle CPU & `<= 35 MB` RAM:** Purely event-driven Bluetooth RFCOMM socket I/O and UI IPC.
- **Zero Kernel Drivers & Zero Elevation:** Operates 100% in standard user space without `.sys` drivers or UAC prompts.
- **100% Offline & Private:** Zero network calls, zero telemetry, and no Sony cloud account required.

---

## 2. Quick Start & Installation (4 Channels)

### System Requirements
- **Operating System:** Windows 10 (version 1903 or later) or Windows 11 (64-bit x64).
- **Permissions:** Standard user account (no Administrator privileges required).
- **Hardware:** Sony WH-1000XM3 (or XM4, XM5, WF series) paired over Bluetooth.

### Method 1: Microsoft WinGet (Recommended)
Install directly using the official Windows Package Manager:
```powershell
# Install from official WinGet community repository:
winget install DooperDrew.SonyBridge

# Or install directly from the local repository manifest:
winget install --manifest packaging/winget/manifests/d/DooperDrew/SonyBridge/0.3.0/
```

### Method 2: Scoop Package Manager
Install via Scoop for clean user-space installation, `sonybridge` / `sonybridge-cli` command aliases, and Start Menu shortcuts:
```powershell
scoop install https://raw.githubusercontent.com/DooperDrew/sonybridge/main/packaging/scoop/sonybridge.json
```

### Method 3: Guided Windows Setup Wizard (Zero-Elevation GUI)
XMCompanion provides a single standalone installer (`XMCompanion-Setup.exe`) and a zero-elevation graphical Windows Setup Wizard:
1. Download **[`XMCompanion-Setup.exe`](https://github.com/DooperDrew/sonybridge/releases/download/v0.3.0/XMCompanion-Setup.exe)** or **[`XMCompanion-v0.3.0-windows-x64.zip`](https://github.com/DooperDrew/sonybridge/releases/download/v0.3.0/XMCompanion-v0.3.0-windows-x64.zip)** from [GitHub Releases](https://github.com/DooperDrew/sonybridge/releases/latest).
2. Run **`XMCompanion-Setup.exe`** (or extract the archive and run `setup.cmd` / `setup-wizard.ps1`).
3. The graphical setup wizard allows you to:
   - Select your installation directory (default: `%LOCALAPPDATA%\Programs\XMCompanion`, no Administrator privileges needed).
   - Toggle **Start Menu** (`XMCompanion` and `XMCompanion Diagnostic Probe`) and **Desktop** shortcuts.
   - Configure automatic launch at Windows sign-in (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run\XMCompanion`).
   - Launch XMCompanion immediately upon completion.
4. XMCompanion registers cleanly in Windows **Installed apps** (Settings &rarr; Apps / Control Panel) with 1-click uninstallation support.

For unattended, console, or scripted environments:
```powershell
# Silent zero-touch installation:
.\setup-wizard.ps1 -Silent

# Console-mode interactive setup wizard:
.\setup-wizard.ps1 -NoGui

# Lightweight user-space installer script:
.\install.ps1
```

### Method 4: Portable Run (No Installation Required)
1. Download and extract [`XMCompanion-v0.3.0-portable.zip`](https://github.com/DooperDrew/sonybridge/releases/download/v0.3.0/XMCompanion-v0.3.0-portable.zip) (or [`XMCompanion-v0.3.0-windows-x64.zip`](https://github.com/DooperDrew/sonybridge/releases/download/v0.3.0/XMCompanion-v0.3.0-windows-x64.zip)) into any folder (e.g., `C:\Tools\XMCompanion` or a USB drive).
2. Double-click **`XMCompanion.exe`** (or run `.\XMCompanion.exe probe` in terminal).
3. The XMCompanion headphone icon appears in your Windows System Tray (notification area next to the clock) and opens the control dashboard.

---

## 3. Pairing & Connecting Your Sony Headphones

1. Power on your **WH-1000XM3** (or XM4/XM5) headphones.
2. If not yet paired with your PC, hold the headphone **Power** button for **7 seconds** until the blue LED flashes twice repeatedly and you hear *"Bluetooth pairing"*.
3. Open Windows **Settings &rarr; Bluetooth & devices &rarr; Add device &rarr; Bluetooth** and select **WH-1000XM3**.
4. Once Windows displays *Connected voice, music*, click **Connect RFCOMM** in the top-right corner of XMCompanion (or right-click the tray icon and click **Quick Reconnect**).
5. XMCompanion opens the Sony MDR RFCOMM control channel (`96cc203e-5068-46ad-b32d-396ceebe3928`), synchronizes battery telemetry, and enforces Stereo (`A2DP`) audio if configured.

---

## 4. First-Run Onboarding Wizard & Bilingual UI (`EN` / `IT`)

- **First-Run Onboarding Modal:** On your first launch (when `onboarding_completed` is `false` in `%APPDATA%\XMCompanion\config.json`), XMCompanion presents a guided 3-step onboarding modal explaining Bluetooth pairing, Windows Stereo (`A2DP`) protection, and Sound Profiles. You can re-open the wizard anytime via the **`?`** button in the header.
- **Instant Language Switching (`EN` / `IT`):** Click **`EN`** or **`IT`** in the top bar to switch all UI labels, tooltips, and descriptions between English and Italian immediately.

---

## 5. System Tray & 3-Tab Adaptive Flyout Controls

### Windows System Tray
- **Left-Click Tray Icon:** Toggles the XMCompanion window (show/hide).
- **Right-Click Context Menu:**
  - `Sony WH-1000XM3` — Status header
  - `Open Dashboard` — Restores the window to the foreground
  - `Quick Reconnect` — Re-establishes the RFCOMM control channel
  - `Quit XMCompanion` — Closes the RFCOMM socket cleanly and exits
- **Close Button (`[X]`):** Minimizes XMCompanion to the system tray so battery monitoring and audio protection stay active with minimal footprint.

### 3-Tab Adaptive Flyout
The main interface is organized into three focused tabs:
1. **🎛️ Dashboard & EQ:** 1-click Sound Profiles, ANC / Wind Reduction / Ambient Sound (`1..20`) + Focus on Voice, Clear Bass (`-10..+10`), and the 5-Band Hardware Equalizer.
2. **🪟 Windows & Power:** 1-click **Enforce High-Res Stereo (A2DP)**, Auto-Switch A2DP on Connect toggle, Start with Windows (`HKCU\...\Run`) toggle, Auto-Power Off timeout, and Right-Ear Touch Sensor lock.
3. **🔬 Architecture & Wire Inspector:** Live hexadecimal MDR packet telemetry inspector and 1-click RFCOMM diagnostic probe.

---

## 6. 1-Click Sound Profiles

At the top of the **Dashboard & EQ** tab, click any Sound Profile chip to configure both the ANC controller (`0x68`) and the 5-band hardware Equalizer + Clear Bass (`0x58`) in a single operation:

| Profile | ANC Mode | Clear Bass | 5-Band EQ (`400`, `1k`, `2.5k`, `6.3k`, `16k`) | Best For |
|---|---|---|---|---|
| **Focus (Max ANC)** | Noise Canceling (Dual Sensor) | `+2` | `[0, +1, +2, +1, 0]` | Deep coding, flights, noisy offices |
| **Office / Calls** | Ambient Sound (Level 12 + Voice Focus) | `0` | `[-1, +2, +4, +2, 0]` | Video calls, hearing colleagues while working |
| **Cinema / Bass Boost** | Noise Canceling (Dual Sensor) | `+6` | `[+4, +2, 0, +2, +3]` | Movies, gaming, electronic & hip-hop music |
| **Studio Reference** | Noise Canceling (Dual Sensor) | `0` | `[0, 0, 0, 0, 0]` | Uncolored flat frequency response |

---

## 7. Active Noise Canceling (ANC), Wind Reduction & Ambient Sound

- **ANC On:** Activates Sony's Dual Noise Sensor technology for maximum attenuation.
- **Wind Noise Reduction:** Hardware MDR v1 sub-mode specifically available on the WH-1000XM3/XM4 that disables the outer feed-forward mic wind turbulence while keeping inner feedback ANC active.
- **Ambient Sound (`1..=20`):** Drag the 20-step slider to control how much environmental sound is mixed in.
- **Focus on Voice:** Check this box while in Ambient Sound mode to attenuate low-frequency hum while passing human speech frequencies clearly.
- **Off:** Disables active noise processing for passive isolation and maximum battery life.

---

## 8. Hardware 5-Band Equalizer & Clear Bass

- **5 Frequency Bands:** `400 Hz`, `1 kHz`, `2.5 kHz`, `6.3 kHz`, and `16 kHz` (adjustable from `-10 dB` to `+10 dB`).
- **Clear Bass (`-10..+10`):** Sony's dedicated DSP sub-bass enhancer that boosts low-end punch without distorting mid-range vocals.
- **Hardware EEPROM Persistence:** Every change sends MDR v1 opcode `0x58` (`EQEBB_SET_PARAM`) directly to the headphone DSP, meaning your tuning stays saved inside the headphones even when you switch to another device.

---

## 9. Smart Windows Stereo (A2DP) Audio Routing & Power Controls

Open the **🪟 Windows & Power** tab to manage Windows OS integration and hardware power settings:
- **Enforce High-Res Stereo (A2DP) Now:** Enumerates active Windows Core Audio render endpoints via `IMMDeviceEnumerator`, locates your Sony **Stereo (`A2DP`)** endpoint, and invokes `IPolicyConfig::SetDefaultEndpoint` across `eConsole`, `eMultimedia`, and `eCommunications` roles so Windows immediately stops routing audio through the muffled `Hands-Free AG Audio` (`HFP`) device.
- **Auto-Switch A2DP on Connect:** Automatically enforces Stereo A2DP whenever SonyBridge connects to your headphones.
- **Start with Windows:** Registers or removes `SonyBridge` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- **Auto-Power Off (`0x28`):** Configure headphone idle shutdown (`5 min`, `30 min`, `1 hour`, `3 hours`, or `Do not turn off`).
- **Right-Ear Touch Sensor Panel (`0xD8`):** Enable or lock the capacitive touch panel on the right earcup to prevent accidental pauses or track skips.

---

## 10. Live Wire Frame Inspector & Headless CLI Diagnostics

- **Live Hexadecimal Wire Frame Inspector:** Open the **🔬 Architecture & Wire Inspector** tab to view the exact byte-stuffed Sony MDR v1 packet (`3E 0C ... 3C`) transmitted on every button click or slider change.
- **Headless Diagnostic CLI (`sonybridge-cli.exe`):**
  ```powershell
  # Inspect paired Sony Bluetooth devices, RFCOMM service UUIDs, and Windows Core Audio endpoints:
  .\sonybridge-cli.exe probe

  # Enforce High-Res Stereo (A2DP) from the command line or scripts:
  .\sonybridge-cli.exe enforce-a2dp
  ```

---

## 11. Hardware QA Testing Procedure (WH-1000XM3)

Follow these steps when validating physical WH-1000XM3 hardware (tracked in [Milestone M9](https://github.com/DooperDrew/sonybridge/milestone/10), Issues [`#10`](https://github.com/DooperDrew/sonybridge/issues/10) and [`#11`](https://github.com/DooperDrew/sonybridge/issues/11)):

1. **Step 1 — Headless Diagnostic Probe:**
   With your WH-1000XM3 powered on and connected in Windows Bluetooth settings, run:
   ```powershell
   .\sonybridge-cli.exe probe
   ```
   Verify that the JSON output lists your `WH-1000XM3` RFCOMM service (`96cc203e-5068-46ad-b32d-396ceebe3928`) and classifies your Windows audio endpoints (`A2DP Stereo` vs `Hands-Free AG`).
2. **Step 2 — Connect & Verify Live Telemetry:**
   Launch `sonybridge-app.exe` and click **Connect RFCOMM**. Verify that the transport badge shows `WinRT RFCOMM Socket (96cc203e-...)` and the battery gauge reflects your headphone charge.
3. **Step 3 — Test ANC, Ambient Sound & 5-Band EQ:**
   - Switch between **ANC On**, **Wind Reduction**, **Ambient Sound (1–20)**, and **Off** and confirm the audible change in the headset.
   - Click the **Cinema / Bass Boost** and **Studio Reference** Sound Profile chips while playing music and confirm the DSP tone changes immediately.
   - Open the **Architecture & Wire Inspector** tab and confirm live hexadecimal frames (`3E 0C ... 3C`) update on each action.
4. **Step 4 — Submit Empirical QA Report:**
   Paste the output of `.\sonybridge-cli.exe probe` into [Issue #10](https://github.com/DooperDrew/sonybridge/issues/10) or open a [Hardware QA Report](https://github.com/DooperDrew/sonybridge/issues/new?template=hardware_qa_report.md).

---

## 12. Troubleshooting

- **Headphones Connected in Windows, but RFCOMM Connect Uses Mock Fallback:**
  Ensure the mobile *Sony | Headphones Connect* app on your phone is not actively holding the Bluetooth RFCOMM control socket open (the WH-1000XM3 only allows one active MDR control socket at a time). Disconnect the phone app or toggle Bluetooth off/on on the phone, then click **Connect RFCOMM** in SonyBridge.
- **Audio Still Sounds Muffled During a Voice Call:**
  Click **Enforce High-Res Stereo (A2DP)** in the **Windows & Power** tab, and ensure your communication app (Discord/Teams/Zoom) uses your desk/laptop microphone rather than `Headset Microphone (WH-1000XM3 Hands-Free AG Audio)`.

---

## 13. Uninstallation

### Option A: Via Windows Settings (Installed Apps)
1. Open Windows **Settings &rarr; Apps &rarr; Installed apps** (or Control Panel &rarr; *Programs and Features*).
2. Locate **XMCompanion (Sony Headphones Companion for Windows)** and click **Uninstall**.

### Option B: Via PowerShell Uninstaller Script
From the installation folder (`%LOCALAPPDATA%\Programs\XMCompanion`) or extracted release archive:
```powershell
# Interactive uninstaller (asks whether to keep or remove %APPDATA%\XMCompanion profiles):
.\uninstall.ps1

# Automated silent uninstaller (preserves user settings):
.\uninstall.ps1 -Silent

# Automated silent uninstaller with full user data purge:
.\uninstall.ps1 -Silent -PurgeData
```

### Option C: Portable Clean-Up
If you ran `XMCompanion.exe` portably without installing:
1. Right-click the XMCompanion tray icon and select **Quit XMCompanion**.
2. Delete the folder where you extracted the `.zip` (and optionally `%APPDATA%\XMCompanion`).

---

*SonyBridge is free and open-source software licensed under the [MIT License](../LICENSE).*
