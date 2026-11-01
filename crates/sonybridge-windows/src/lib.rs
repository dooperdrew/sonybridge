//! Windows platform-specific integrations for SonyBridge.
//!
//! Covers:
//! - Bluetooth RFCOMM / WinRT StreamSocket transport
//! - Audio endpoint detection and default routing (A2DP vs Hands-Free via `IPolicyConfig`)
//! - Per-user Windows startup registration (`HKCU\...\Run`)

pub mod audio;
pub mod autostart;
pub mod bluetooth;
pub mod probe;

pub use audio::{AudioEndpoint, AudioEndpointSanitizer};
pub use autostart::AutostartManager;
pub use bluetooth::{
    ConnectionDiagnostics, DiscoveredDevice, WinRtRfcommTransport, WindowsBluetoothManager,
};
pub use probe::run_diagnostic_probe;

/// Attaches to the parent process console (e.g. PowerShell or cmd) so CLI output is displayed
/// when invoked from a terminal even if built as a GUI subsystem application.
pub fn attach_parent_console() {
    #[cfg(windows)]
    unsafe {
        use windows::core::w;
        use windows::Win32::Storage::FileSystem::{
            CreateFileW, FILE_GENERIC_WRITE, FILE_SHARE_WRITE, OPEN_EXISTING,
        };
        use windows::Win32::System::Console::{
            AttachConsole, GetStdHandle, SetStdHandle, ATTACH_PARENT_PROCESS, STD_ERROR_HANDLE,
            STD_OUTPUT_HANDLE,
        };

        let _ = AttachConsole(ATTACH_PARENT_PROCESS);

        let out = GetStdHandle(STD_OUTPUT_HANDLE);
        if out.is_err() || out.unwrap().is_invalid() {
            if let Ok(handle) = CreateFileW(
                w!("CONOUT$"),
                FILE_GENERIC_WRITE.0,
                FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
                None,
            ) {
                let _ = SetStdHandle(STD_OUTPUT_HANDLE, handle);
            }
        }

        let err = GetStdHandle(STD_ERROR_HANDLE);
        if err.is_err() || err.unwrap().is_invalid() {
            if let Ok(handle) = CreateFileW(
                w!("CONOUT$"),
                FILE_GENERIC_WRITE.0,
                FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
                None,
            ) {
                let _ = SetStdHandle(STD_ERROR_HANDLE, handle);
            }
        }
    }
}
