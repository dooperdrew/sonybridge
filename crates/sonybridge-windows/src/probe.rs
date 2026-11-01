//! Diagnostic probe module for environment and hardware inspection.

use sonybridge_core::commands;
use sonybridge_core::protocol::MdrFrame;
use sonybridge_core::transport::Transport;

use crate::audio::AudioEndpointSanitizer;
use crate::bluetooth::{WinRtRfcommTransport, WindowsBluetoothManager};

/// Runs the full diagnostic probe against local Bluetooth devices and Core Audio endpoints.
pub async fn run_diagnostic_probe() {
    println!("=== SonyBridge M0/M9 Diagnostic Probe ===");
    println!("\n[1/3] Scanning for paired Sony Bluetooth devices...");
    match WindowsBluetoothManager::discover_paired_sony_devices().await {
        Ok(devices) => {
            if devices.is_empty() {
                println!("  -> No paired Classic Bluetooth Sony audio headphones detected.");
                // Check if BLE beacon was paired instead
                if let Ok(ble_devices) =
                    WindowsBluetoothManager::discover_paired_sony_ble_devices().await
                {
                    if let Some(ble_name) = ble_devices.first() {
                        println!("  -> [!] Detected paired BLE beacon: '{ble_name}'");
                        println!("  -> [!] IMPORTANT: '{ble_name}' is Bluetooth Low Energy only (no audio / no RFCOMM).");
                        println!("  -> [!] Remedy: Put headphones in pairing mode (hold Power for 7s until 'Bluetooth pairing')");
                        println!("         and select 'WH-1000XM3' (headphone icon, NOT LE_) in Windows Settings.");
                    } else {
                        println!("  -> Tip: Ensure your WH-1000XM3 is paired in Windows Settings.");
                    }
                } else {
                    println!("  -> Tip: Ensure your WH-1000XM3 is paired in Windows Settings.");
                }
            } else {
                println!("  -> Found {} paired device(s):", devices.len());
                for d in &devices {
                    println!(
                        "     * Name: {}\n       MAC: {:012X}\n       Connected: {}\n       RFCOMM Services: {:?}",
                        d.name, d.mac_address, d.is_connected, d.rfcomm_services
                    );
                }

                if let Some(active) = devices.iter().find(|d| d.is_connected) {
                    println!(
                        "\n[2/3] Probing live RFCOMM MDR channel on {}...",
                        active.name
                    );
                    let mut transport =
                        WinRtRfcommTransport::with_mac(&active.name, active.mac_address);
                    match transport.connect().await {
                        Ok(()) => {
                            println!("  -> RFCOMM StreamSocket connected!");
                            let query = MdrFrame::new(0, commands::build_v1_battery_query());
                            if transport.send(&query.encode_wire()).await.is_ok() {
                                println!(
                                    "  -> Sent MDR v1 Battery Query: {:02X?}",
                                    query.encode_wire()
                                );
                            }
                            let _ = transport.disconnect().await;
                        }
                        Err(e) => {
                            println!("  -> RFCOMM connect note: {e}");
                        }
                    }
                } else {
                    println!("\n[2/3] Skipping live RFCOMM handshake (headphones currently powered off / disconnected).");
                }
            }
        }
        Err(e) => {
            println!("  -> Bluetooth scan error: {e}");
        }
    }

    println!("\n[3/3] Inspecting Windows Core Audio endpoints...");
    match AudioEndpointSanitizer::enumerate_render_endpoints() {
        Ok(endpoints) => {
            println!("  -> Found {} active render endpoint(s):", endpoints.len());
            for ep in &endpoints {
                let tag = if ep.is_hands_free {
                    "[Hands-Free AG - Degraded Quality]"
                } else {
                    "[Stereo / Normal]"
                };
                let def = if ep.is_default { " (DEFAULT)" } else { "" };
                println!("     * {tag}{def} {}", ep.id);
            }
        }
        Err(e) => {
            println!("  -> Audio endpoint enumeration error: {e}");
        }
    }
    println!("\n=== Diagnostic Probe Completed ===");
}
