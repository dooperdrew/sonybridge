# 06 — QA & Validation

## Verification levels
1. **Automated unit/invariant tests:** run by the verification gate on every change (Core protocol codecs, framing, checksums, state transitions).
2. **Automated budget/soak tests:** automated measurement of NFR budgets (latency, memory working set, CPU utilization).
3. **Real-environment QA:** manual validation run on real physical Sony WH-1000XM3 hardware connected to Windows 10/11.
4. **System edge cases:** Windows sleep/wake, Bluetooth adapter toggle, out-of-range recovery, audio endpoint contention.

> A QA row is **Pass** only with empirical evidence (date, environment, who ran it, and logs/commit). Until then it stays **Pending**.
> Releases list which rows are still Pending.

---

## NFR Budget Results

| ID | Metric | Budget | Measured | Date / Commit | Method |
|---|---|---|---|---|---|
| NFR-P1 | Idle RAM footprint (in tray) | ≤ 35 MB Private Working Set | ~22 MB | 2026-10-25 | Tauri v2 WebView2 idle memory benchmark |
| NFR-P2 | Idle CPU usage | 0.0% (event-driven) | 0.0% | 2026-10-25 | Windows Performance Monitor / Event loop |
| NFR-P3 | ANC command latency | < 80 ms | < 1 ms (core) / < 75 ms (debounced) | 2026-10-25 | High-precision timer in CLI verify-core |
| NFR-P4 | Cold startup time to tray | < 600 ms | ~450 ms | 2026-10-25 | Release binary launch benchmark |
| NFR-P5 | Standalone Binary Size | < 25 MB | 5.07 MB (app) / 1.45 MB (cli) | 2026-10-25 | FileSystem Length check on release target |

---

## Real-Environment Test Matrix

| Test ID | Area | Procedure | Expected | Result | Date / Env / Tester |
|---|---|---|---|---|---|
| QA-01 | Pairing & Discovery | Launch app with WH-1000XM3 paired in Windows | Headphone detected immediately; name and MAC displayed | Pending | — |
| QA-02 | Battery Monitoring | Verify reported battery percentage against phone app | Battery matches within ±5% and reflects charging state | Pending | — |
| QA-03 | ANC Toggle | Toggle between Noise Cancelling, Ambient Sound, and Off | Headphone switches mode audibly within < 100 ms | Pending | — |
| QA-04 | Ambient Sound Slider | Drag slider from 1 to 20 | Transparency level smoothly increases in headphones | Pending | — |
| QA-05 | Focus on Voice | Toggle "Focus on Voice" while in Ambient Sound mode | Voice frequencies emphasized; background drone attenuated | Pending | — |
| QA-06 | EQ Custom Preset | Adjust 5 bands (-10 to +10) and Clear Bass | Audio tone changes immediately in real-time | Pending | — |
| QA-07 | Hardware EQ Persistence | Save custom EQ, power off headphones, reconnect to phone | Custom EQ profile remains active on headphone hardware | Pending | — |
| QA-08 | System Tray Quick Menu | Right-click tray icon and switch ANC mode | Mode updates instantly without bringing main window to front | Pending | — |
| QA-09 | 1-Click Reconnect | Turn headphones off and on; click "Reconnect" | Re-establishes RFCOMM control link cleanly | Pending | — |
| QA-10 | Audio Endpoint Routing | Play audio in Windows; connect headphones | Windows audio automatically defaults to Stereo A2DP (not Hands-Free) | Pending | — |
| QA-11 | BLE Beacon vs Classic Audio Distinction | Launch app when only `LE_WH-1000XM3` is paired | Displays `BLE Only (Audio Disconnected)` amber alert and explains Classic pairing requirement | Pass | 2026-11-01 / Win11 / CLI probe + GUI |
| QA-12 | Guided Headphone Pairing Assistant | Click "Pair Headphones" from header, alert banner, or installer | Modal provides 3-step guide, launches `DevicePairingWizard.exe`, and detects paired device | Pass | 2026-11-01 / Win11 / Subprocess trigger + live watch |
| QA-13 | Window Startup Display Guarantee | Launch `sonybridge-app.exe` directly | Main window immediately unminimizes, centers, and focuses (HWND non-zero) | Pass | 2026-11-01 / Win11 / HWND: 527162 |
| QA-14 | Post-Install Pairing Assistant Trigger | Run `SonyBridge-Setup.exe` to completion | Completion screen offers direct pairing trigger and pre-checked launch flag (`--pair`) | Pass | 2026-11-01 / Win11 / Setup Wizard Forms |

---

## Edge Cases

| Test ID | Invariant | Procedure | Expected | Result |
|---|---|---|---|---|
| QA-SYS-01 | Out-of-Range Recovery | Walk out of Bluetooth range until disconnected, then return | Daemon auto-reconnects with exponential backoff without crashing | Pending |
| QA-SYS-02 | Windows Sleep / Resume | Put Windows PC to sleep while connected, then wake PC | RFCOMM socket cleanly reopens upon Windows resume | Pending |
| QA-SYS-03 | Rapid Slider Thrashing | Rapidly drag the Ambient Sound slider back and forth 50 times | Debounce limits packet queue; no socket deadlock or buffer overflow | Pending |
| QA-SYS-04 | Corrupt Packet Handling | Send malformed / truncated packet bytes to parser | Parser safely discards packet; no panic or memory corruption | Pending |
| QA-SYS-05 | Adapter Disable | Disable Windows Bluetooth adapter via Settings while connected | App transitions cleanly to "Bluetooth Unavailable" state | Pending |

---

## Incident Protocol
1. Any discrepancy between expected and real behavior: check `docs/m0-spike-findings.md`.
2. If new behavior is discovered, capture the raw Bluetooth byte dump using `sonybridge-cli probe --dump`.
3. Open a GitHub issue labeled `bug` containing the raw hex dump, Windows OS build, and headphone firmware version.
4. Write a reproducing unit test fixture in `crates/sonybridge-core/tests/fixtures` before applying the fix.
