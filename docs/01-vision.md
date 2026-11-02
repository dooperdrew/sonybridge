# 01 — Vision

> Status: **APPROVED** | App name: **XMCompanion** (formerly SonyBridge)

## Problem
Sony produces some of the most popular Active Noise Cancelling headphones on the market (WH-1000XM3, WH-1000XM4, WH-1000XM5, WF-1000XM4/XM5). However, when paired with Windows 10/11 PCs, user experience is fundamentally degraded:
1. **Absence of Official Software:** Sony provides the "Sony | Headphones Connect" companion app exclusively for iOS and Android. Windows users have no native GUI to view accurate battery percentage, toggle Ambient Sound levels, customize the hardware 5-band Equalizer / Clear Bass, or manage device settings.
2. **Windows Bluetooth Connection Quirks:** Windows 10 Bluetooth pairing and reconnect frequently fail, stall, or require removing and re-pairing the device.
3. **Audio Routing & Hands-Free Agony:** Windows defaults to switching between "Headphones (Stereo A2DP)" and "Headset (Hands-Free AG Audio)", leading to muffled, distorted audio quality when applications access communications devices or microphones.
4. **Hardware Controls Inaccessible:** Features like Ambient Sound Control (0–20 slider, Focus on Voice), Touch Sensor Lock, and Auto-Power Off idle timeouts cannot be configured from the PC.

## Goal
**XMCompanion** is a lightweight, responsive Windows 10/11 desktop companion and system tray application that bridges Sony Bluetooth headphones to Windows. It brings reliable one-click connectivity, automatic audio endpoint sanity, real-time system tray battery monitoring, and full control over Noise Cancelling, Ambient Sound, and hardware EQ.

## Target users
- Owners of Sony WH-1000XM3 (and extended family: WH-1000XM4, WH-1000XM5, WF-1000XM3/XM4/XM5) using Windows 10 or 11 for daily work, music, video editing, calls, or gaming.
- Users who need instant access to ANC and battery state from the Windows Taskbar without needing to pull out their smartphone.
- Users frustrated by Windows Bluetooth reconnect failures and audio endpoint confusion.

## Guiding principles (in priority order)
1. **Rock-solid Bluetooth Reliability:** Connect and reconnect smoothly using native Windows Bluetooth APIs; fail gracefully with clear diagnostic messages.
2. **Invisible Resource Consumption:** Run silently in the background tray with negligible memory (≤ 35 MB) and 0% idle CPU.
3. **Pure Core Protocol Separation:** Reverse-engineered Sony MDR protocol logic must be isolated in a platform-independent, 100% unit-tested Rust crate with `#![forbid(unsafe_code)]`.
4. **Immediate Hardware Feedback:** Changes to ANC, Ambient Sound level, or EQ apply instantly in hardware without stuttering audio playback.
5. **No Telemetry, No Cloud, 100% Offline:** Operates entirely locally on the user's machine without external network requests or proprietary account logins.

## Non-goals (v1)
- Firmware flashing / over-the-air firmware update installation (risk of bricking physical hardware).
- 360 Reality Audio HRTF ear-photo analysis (requires Sony cloud servers).
- Kernel-mode drivers or custom Bluetooth virtual adapter drivers (user-space WinRT / Winsock only).
- Non-Sony headphone support (Bose, Sennheiser, Apple are out of scope).
