// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use sonybridge_core::commands;
use sonybridge_core::config::AppConfig;
use sonybridge_core::models::{AncMode, EqPreset, EqualizerState, HeadphoneModel};
use sonybridge_core::protocol::MdrFrame;
use sonybridge_core::state::{ConnectionStatus, HeadphoneState, StateMachine};
use sonybridge_core::transport::Transport;
use sonybridge_windows::{
    AudioEndpointSanitizer, AutostartManager, ConnectionDiagnostics, WinRtRfcommTransport,
    WindowsBluetoothManager,
};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, State};

struct AppState {
    state_machine: Mutex<StateMachine>,
    config: Mutex<AppConfig>,
    config_path: PathBuf,
    transport: tokio::sync::Mutex<Option<WinRtRfcommTransport>>,
}

fn default_config_path() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("SonyBridge").join("config.json")
}

#[derive(Deserialize)]
struct AncPayload {
    mode: String,
    level: u8,
    #[serde(rename = "focusOnVoice")]
    focus_on_voice: bool,
}

#[derive(Deserialize)]
struct EqPayload {
    preset: String,
    bands: [i8; 5],
    #[serde(rename = "clearBass")]
    clear_bass: i8,
}

#[derive(Serialize)]
struct CommandResult {
    status: String,
    message: Option<String>,
    wire_hex: Option<String>,
}

fn format_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[tauri::command]
fn get_device_state(state: State<AppState>) -> HeadphoneState {
    let sm = state.state_machine.lock().unwrap();
    sm.state.clone()
}

#[tauri::command]
fn get_app_config(state: State<AppState>) -> AppConfig {
    let cfg = state.config.lock().unwrap();
    cfg.clone()
}

#[tauri::command]
fn set_language(state: State<AppState>, lang: String) -> CommandResult {
    let mut cfg = state.config.lock().unwrap();
    cfg.language = lang;
    let _ = cfg.save_to_path(&state.config_path);
    CommandResult {
        status: "ok".to_string(),
        message: None,
        wire_hex: None,
    }
}

#[tauri::command]
fn complete_onboarding(state: State<AppState>) -> CommandResult {
    let mut cfg = state.config.lock().unwrap();
    cfg.onboarding_completed = true;
    let _ = cfg.save_to_path(&state.config_path);
    CommandResult {
        status: "ok".to_string(),
        message: None,
        wire_hex: None,
    }
}

#[tauri::command]
async fn set_anc_mode(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    payload: AncPayload,
) -> Result<CommandResult, String> {
    let wire_bytes = {
        let mut sm = state.state_machine.lock().unwrap();
        let mode = match payload.mode.as_str() {
            "Off" => AncMode::Off,
            "WindNoiseReduction" => AncMode::WindNoiseReduction,
            "AmbientSound" => AncMode::AmbientSound {
                level: payload.level,
                focus_on_voice: payload.focus_on_voice,
            },
            _ => AncMode::NoiseCanceling,
        };
        let frame = sm.set_v1_anc(mode);
        tracing::info!(
            seq = frame.seq_no,
            "Generated MDR v1 NCASM_SET_PARAM packet"
        );
        frame.encode_wire()
    };

    let mut transport_guard = state.transport.lock().await;
    if let Some(transport) = transport_guard.as_mut() {
        if transport.is_connected() {
            let _ = transport.send(&wire_bytes).await;
        }
    }

    if let Some(tray) = app.tray_by_id("main_tray") {
        let _ = tray.set_tooltip(Some(format!(
            "SonyBridge — WH-1000XM3 (85% • {})",
            payload.mode
        )));
    }

    Ok(CommandResult {
        status: "ok".to_string(),
        message: None,
        wire_hex: Some(format_hex(&wire_bytes)),
    })
}

#[tauri::command]
async fn set_equalizer(
    state: State<'_, AppState>,
    payload: EqPayload,
) -> Result<CommandResult, String> {
    let wire_bytes = {
        let mut sm = state.state_machine.lock().unwrap();
        let preset = match payload.preset.as_str() {
            "Bright" => EqPreset::Bright,
            "Excited" => EqPreset::Excited,
            "Mellow" => EqPreset::Mellow,
            "Relaxed" => EqPreset::Relaxed,
            "Vocal" => EqPreset::Vocal,
            "TrebleBoost" => EqPreset::TrebleBoost,
            "BassBoost" => EqPreset::BassBoost,
            "Speech" => EqPreset::Speech,
            "Manual" => EqPreset::Manual,
            "Custom1" => EqPreset::Custom1,
            "Custom2" => EqPreset::Custom2,
            _ => EqPreset::Off,
        };

        let eq_state = EqualizerState {
            preset,
            bands: payload.bands,
            clear_bass: payload.clear_bass,
        };

        let frame = sm.set_v1_equalizer(eq_state);
        tracing::info!(
            seq = frame.seq_no,
            "Generated MDR v1 EQEBB_SET_PARAM packet"
        );
        frame.encode_wire()
    };

    let mut transport_guard = state.transport.lock().await;
    if let Some(transport) = transport_guard.as_mut() {
        if transport.is_connected() {
            let _ = transport.send(&wire_bytes).await;
        }
    }

    Ok(CommandResult {
        status: "ok".to_string(),
        message: None,
        wire_hex: Some(format_hex(&wire_bytes)),
    })
}

#[tauri::command]
async fn apply_sound_profile(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    profile_name: String,
) -> Result<CommandResult, String> {
    let profile_opt = {
        let mut cfg = state.config.lock().unwrap();
        let found = cfg
            .profiles
            .iter()
            .find(|p| p.name == profile_name)
            .cloned();
        if found.is_some() {
            cfg.active_profile = profile_name.clone();
            let _ = cfg.save_to_path(&state.config_path);
        }
        found
    };

    if let Some(profile) = profile_opt {
        let (anc_wire, eq_wire) = {
            let mut sm = state.state_machine.lock().unwrap();
            let f1 = sm.set_v1_anc(profile.anc).encode_wire();
            let f2 = sm.set_v1_equalizer(profile.equalizer).encode_wire();
            (f1, f2)
        };

        let mut transport_guard = state.transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            if transport.is_connected() {
                let _ = transport.send(&anc_wire).await;
                let _ = transport.send(&eq_wire).await;
            }
        }

        if let Some(tray) = app.tray_by_id("main_tray") {
            let _ = tray.set_tooltip(Some(format!("SonyBridge — WH-1000XM3 ({profile_name})")));
        }

        Ok(CommandResult {
            status: "ok".to_string(),
            message: Some(format!("Applied profile: {profile_name}")),
            wire_hex: Some(format!(
                "{} | {}",
                format_hex(&anc_wire),
                format_hex(&eq_wire)
            )),
        })
    } else {
        Err(format!("Profile '{profile_name}' not found"))
    }
}

#[tauri::command]
fn relaunch_setup_wizard() -> CommandResult {
    let script_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("setup-wizard.ps1")))
        .unwrap_or_else(|| PathBuf::from("setup-wizard.ps1"));
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path.to_string_lossy(),
        ])
        .spawn();
    CommandResult {
        status: "ok".to_string(),
        message: Some("Launched Setup Wizard".to_string()),
        wire_hex: None,
    }
}

#[tauri::command]
fn trigger_uninstall() -> CommandResult {
    let script_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("uninstall.ps1")))
        .unwrap_or_else(|| PathBuf::from("uninstall.ps1"));
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path.to_string_lossy(),
        ])
        .spawn();
    CommandResult {
        status: "ok".to_string(),
        message: Some("Launched Uninstallation Wizard".to_string()),
        wire_hex: None,
    }
}

#[tauri::command]
fn run_diagnostic_probe_in_terminal() -> CommandResult {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoExit",
                "-Command",
                &format!("& '{}' probe", exe.display()),
            ])
            .spawn();
    }
    CommandResult {
        status: "ok".to_string(),
        message: Some("Opened diagnostic probe in terminal".to_string()),
        wire_hex: None,
    }
}

#[tauri::command]
async fn get_connection_diagnostics(
    state: State<'_, AppState>,
) -> Result<ConnectionDiagnostics, String> {
    let mut diag = WindowsBluetoothManager::run_connection_diagnostics().await;
    let sm = state.state_machine.lock().unwrap();
    if diag.is_connected {
        diag.battery_percent = Some(sm.state.battery.percentage);
    }
    Ok(diag)
}

#[tauri::command]
fn open_windows_settings(page: String) -> CommandResult {
    let uri = match page.as_str() {
        "bluetooth" => "ms-settings:bluetooth",
        "sound" => "ms-settings:sound",
        _ => "ms-settings:devices",
    };
    let _ = std::process::Command::new("explorer").arg(uri).spawn();
    CommandResult {
        status: "ok".to_string(),
        message: Some(format!("Opened {uri}")),
        wire_hex: None,
    }
}

#[tauri::command]
fn launch_pairing_wizard() -> CommandResult {
    let status = std::process::Command::new("DevicePairingWizard.exe").spawn();
    if status.is_err() {
        let _ = std::process::Command::new("explorer")
            .arg("ms-settings-connectabledevices:devicediscovery")
            .spawn();
    }
    CommandResult {
        status: "ok".to_string(),
        message: Some("Launched Windows Device Pairing Wizard".to_string()),
        wire_hex: None,
    }
}

#[tauri::command]
fn open_add_device_flyout() -> CommandResult {
    let _ = std::process::Command::new("explorer")
        .arg("ms-settings-connectabledevices:devicediscovery")
        .spawn();
    CommandResult {
        status: "ok".to_string(),
        message: Some("Opened Windows Add Device flyout".to_string()),
        wire_hex: None,
    }
}

#[tauri::command]
fn should_open_pairing_assistant() -> bool {
    std::env::args().any(|a| a == "--pair" || a == "-p" || a == "--pairing")
}

#[tauri::command]
async fn trigger_reconnect(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<CommandResult, String> {
    tracing::info!("Scanning for paired Sony headphones...");
    let diag = WindowsBluetoothManager::run_connection_diagnostics().await;

    if diag.status == "connected" {
        let devices = WindowsBluetoothManager::discover_paired_sony_devices()
            .await
            .map_err(|e| e.to_string())?;

        if let Some(device) = devices.first() {
            let mac_str = format!("{:012X}", device.mac_address);
            tracing::info!(
                name = %device.name,
                mac = %mac_str,
                connected = device.is_connected,
                "Discovered paired Sony headphone"
            );

            let mut transport = WinRtRfcommTransport::with_mac(&device.name, device.mac_address);
            let rfcomm_ok = transport.connect().await.is_ok();
            let mut wire_hex = None;

            if rfcomm_ok {
                let query = MdrFrame::new(0, commands::build_v1_battery_query()).encode_wire();
                let _ = transport.send(&query).await;
                wire_hex = Some(format_hex(&query));
            }

            {
                let mut sm = state.state_machine.lock().unwrap();
                sm.state.device_name = Some(device.name.clone());
                sm.state.mac_address = Some(mac_str.clone());
                sm.state.model = Some(HeadphoneModel::from_device_name(&device.name));
                sm.state.status = if rfcomm_ok || device.is_connected {
                    ConnectionStatus::Connected
                } else {
                    ConnectionStatus::Disconnected
                };
            }

            {
                let mut guard = state.transport.lock().await;
                *guard = if rfcomm_ok { Some(transport) } else { None };
            }

            // Auto-enforce Stereo A2DP if configured
            let should_enforce = state.config.lock().unwrap().enforce_a2dp_on_connect;
            if should_enforce {
                let _ = AudioEndpointSanitizer::enforce_stereo_a2dp();
            }

            if let Some(tray) = app.tray_by_id("main_tray") {
                let _ = tray.set_tooltip(Some(format!("SonyBridge — {} (Connected)", device.name)));
            }

            return Ok(CommandResult {
                status: "ok".to_string(),
                message: Some(format!("Connected to {} ({})", device.name, mac_str)),
                wire_hex,
            });
        }
    }

    // Disconnected / BLE only / Not paired
    {
        let mut sm = state.state_machine.lock().unwrap();
        sm.state.status = ConnectionStatus::Disconnected;
    }
    {
        let mut guard = state.transport.lock().await;
        *guard = None;
    }

    if let Some(tray) = app.tray_by_id("main_tray") {
        let _ = tray.set_tooltip(Some("SonyBridge — Disconnected".to_string()));
    }

    Ok(CommandResult {
        status: diag.status,
        message: Some(diag.message),
        wire_hex: None,
    })
}

#[tauri::command]
fn enforce_a2dp() -> CommandResult {
    let is_hands_free = AudioEndpointSanitizer::is_hands_free_active();
    let switched = AudioEndpointSanitizer::enforce_stereo_a2dp();
    tracing::info!(is_hands_free, ?switched, "Audio endpoint audit performed");
    CommandResult {
        status: "ok".to_string(),
        message: Some(match switched {
            Ok(id) => format!("Enforced Stereo A2DP default endpoint ({id})"),
            Err(_) if is_hands_free => "Hands-Free profile detected and isolated".to_string(),
            Err(_) => "Stereo A2DP audio endpoint is active".to_string(),
        }),
        wire_hex: None,
    }
}

#[tauri::command]
fn set_touch_sensor(state: State<AppState>, enabled: bool) -> CommandResult {
    let mut sm = state.state_machine.lock().unwrap();
    sm.state.settings.touch_sensor_enabled = enabled;
    let payload = vec![
        commands::opcodes::V1_SYSTEM_SET_PARAM,
        0x01,
        if enabled { 0x01 } else { 0x00 },
    ];
    let seq = sm.next_seq();
    let wire = MdrFrame::new(seq, payload).encode_wire();
    tracing::info!(enabled, "Touch sensor preference updated");
    CommandResult {
        status: "ok".to_string(),
        message: None,
        wire_hex: Some(format_hex(&wire)),
    }
}

#[tauri::command]
fn set_auto_off(state: State<AppState>, minutes: u32) -> CommandResult {
    let mut sm = state.state_machine.lock().unwrap();
    sm.state.settings.auto_power_off_minutes = if minutes > 0 { Some(minutes) } else { None };
    let code = match minutes {
        0 => 0x11,
        5 => 0x00,
        30 => 0x01,
        60 => 0x02,
        _ => 0x03,
    };
    let payload = vec![commands::opcodes::V1_POWER_AUTO_OFF_SET, code];
    let seq = sm.next_seq();
    let wire = MdrFrame::new(seq, payload).encode_wire();
    tracing::info!(minutes, "Auto power-off timeout updated");
    CommandResult {
        status: "ok".to_string(),
        message: None,
        wire_hex: Some(format_hex(&wire)),
    }
}

#[tauri::command]
fn set_autostart(state: State<AppState>, enabled: bool) -> CommandResult {
    let res = AutostartManager::set_autostart(enabled);
    if let Ok(mut cfg) = state.config.lock() {
        cfg.auto_connect_on_startup = enabled;
        let _ = cfg.save_to_path(&state.config_path);
    }
    CommandResult {
        status: if res.is_ok() { "ok" } else { "error" }.to_string(),
        message: res.err(),
        wire_hex: None,
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    // 1. Diagnostic Probe CLI command
    if args.iter().any(|a| a == "probe" || a == "--probe") {
        let cli_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("sonybridge-cli.exe")))
            .unwrap_or_else(|| PathBuf::from("sonybridge-cli.exe"));
        if cli_path.exists() {
            let _ = std::process::Command::new(cli_path).arg("probe").status();
        } else {
            sonybridge_windows::attach_parent_console();
            let rt = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
            rt.block_on(async {
                sonybridge_windows::run_diagnostic_probe().await;
            });
        }
        return;
    }

    // 2. Audio Sanitizer quick enforcement
    if args
        .iter()
        .any(|a| a == "--enforce-a2dp" || a == "enforce-a2dp")
    {
        let cli_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("sonybridge-cli.exe")))
            .unwrap_or_else(|| PathBuf::from("sonybridge-cli.exe"));
        if cli_path.exists() {
            let _ = std::process::Command::new(cli_path)
                .arg("enforce-a2dp")
                .status();
        } else {
            sonybridge_windows::attach_parent_console();
            println!("Enforcing Windows Stereo (A2DP) default audio endpoint routing...");
            match AudioEndpointSanitizer::enforce_stereo_a2dp() {
                Ok(msg) => println!("Success! Default audio routed: {}", msg),
                Err(e) => eprintln!("Failed to enforce A2DP: {e}"),
            }
        }
        return;
    }

    // 3. Setup / Installer mode
    let is_explicit_gui = args.iter().any(|a| a == "--gui" || a == "--settings");
    let is_explicit_setup = args.iter().any(|a| a == "--setup" || a == "--install");
    let exe_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
        .unwrap_or_default();
    let is_setup_exe = exe_name.contains("setup") || exe_name.contains("installer");

    if is_explicit_setup || (is_setup_exe && !is_explicit_gui) {
        let script = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("setup-wizard.ps1")))
            .unwrap_or_else(|| PathBuf::from("setup-wizard.ps1"));
        if script.exists() {
            let _ = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    &script.to_string_lossy(),
                ])
                .spawn();
            return;
        }
        // If setup-wizard.ps1 is not present next to the binary, fall through to launching the GUI directly
        // so that clicking Setup.exe always opens the application with the onboarding wizard!
    }

    // 4. Uninstall mode
    if args.iter().any(|a| a == "--uninstall") {
        let script = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("uninstall.ps1")))
            .unwrap_or_else(|| PathBuf::from("uninstall.ps1"));
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &script.to_string_lossy(),
            ])
            .spawn();
        return;
    }

    // 5. Help / CLI information
    if args.iter().any(|a| a == "--help" || a == "-h") {
        sonybridge_windows::attach_parent_console();
        println!("SonyBridge — Sony Bluetooth Headphones System Tray Companion");
        println!("\nUsage: sonybridge-app.exe [OPTIONS] [COMMAND]");
        println!("\nCommands:");
        println!("  probe              Probe local Bluetooth environment for paired Sony headphones and audio endpoints");
        println!("\nOptions:");
        println!("  --gui, --settings  Open the SonyBridge desktop control window (default)");
        println!("  --enforce-a2dp     Enforce high-resolution Stereo A2DP audio routing");
        println!("  --setup, --install Launch the guided Windows Setup Wizard");
        println!("  --uninstall        Launch the clean uninstallation wizard");
        println!("  -h, --help         Print help information");
        return;
    }

    let cfg_path = default_config_path();
    let loaded_cfg = AppConfig::load_from_path(&cfg_path);

    let initial_state = AppState {
        state_machine: Mutex::new(StateMachine::new()),
        config: Mutex::new(loaded_cfg),
        config_path: cfg_path,
        transport: tokio::sync::Mutex::new(None),
    };

    tauri::Builder::default()
        .manage(initial_state)
        .invoke_handler(tauri::generate_handler![
            get_device_state,
            get_app_config,
            set_language,
            complete_onboarding,
            set_anc_mode,
            set_equalizer,
            apply_sound_profile,
            trigger_reconnect,
            get_connection_diagnostics,
            open_windows_settings,
            launch_pairing_wizard,
            open_add_device_flyout,
            should_open_pairing_assistant,
            enforce_a2dp,
            set_touch_sensor,
            set_auto_off,
            set_autostart,
            relaunch_setup_wizard,
            trigger_uninstall,
            run_diagnostic_probe_in_terminal
        ])
        .setup(|app| {
            // Build Context Menu for System Tray
            let header_item = MenuItemBuilder::with_id("header", "SonyBridge — Ready")
                .enabled(false)
                .build(app)?;
            let profile_focus =
                MenuItemBuilder::with_id("prof_focus", "Profile: Focus (ANC + Excited)")
                    .build(app)?;
            let profile_office =
                MenuItemBuilder::with_id("prof_office", "Profile: Office (Ambient + Vocal)")
                    .build(app)?;
            let a2dp_item =
                MenuItemBuilder::with_id("enforce_a2dp", "Enforce Stereo A2DP Audio").build(app)?;
            let reconnect_item =
                MenuItemBuilder::with_id("reconnect", "Quick Reconnect").build(app)?;
            let show_item = MenuItemBuilder::with_id("show", "Open Dashboard").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit SonyBridge").build(app)?;

            let tray_menu = MenuBuilder::new(app)
                .items(&[
                    &header_item,
                    &profile_focus,
                    &profile_office,
                    &a2dp_item,
                    &reconnect_item,
                    &show_item,
                    &quit_item,
                ])
                .build()?;

            // Build System Tray Icon with unique ID for dynamic telemetry updates
            let _tray = TrayIconBuilder::with_id("main_tray")
                .tooltip("SonyBridge — Desktop Companion")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "enforce_a2dp" => {
                        let _ = AudioEndpointSanitizer::enforce_stereo_a2dp();
                    }
                    "reconnect" => {
                        tracing::info!("Tray triggered reconnect");
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Ensure main companion window is shown, centered, and focused on startup
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.center();
                let _ = window.set_focus();

                let win_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running SonyBridge application");
}
