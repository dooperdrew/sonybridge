use std::process::Command;

const RUN_REG_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const APP_VALUE_NAME: &str = "XMCompanion";
const LEGACY_APP_VALUE_NAME: &str = "SonyBridge";

pub struct AutostartManager;

impl AutostartManager {
    /// Enable or disable per-user Windows login autostart for XMCompanion.
    pub fn set_autostart(enabled: bool) -> Result<(), String> {
        if enabled {
            // Remove legacy key if it existed
            let _ = Command::new("reg")
                .args(["delete", RUN_REG_KEY, "/v", LEGACY_APP_VALUE_NAME, "/f"])
                .status();

            let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let quoted = format!("\"{}\"", exe_path.display());
            let status = Command::new("reg")
                .args([
                    "add",
                    RUN_REG_KEY,
                    "/v",
                    APP_VALUE_NAME,
                    "/t",
                    "REG_SZ",
                    "/d",
                    &quoted,
                    "/f",
                ])
                .status()
                .map_err(|e| e.to_string())?;
            if status.success() {
                Ok(())
            } else {
                Err("reg add exited with non-zero status".to_string())
            }
        } else {
            let _ = Command::new("reg")
                .args(["delete", RUN_REG_KEY, "/v", APP_VALUE_NAME, "/f"])
                .status();
            let _ = Command::new("reg")
                .args(["delete", RUN_REG_KEY, "/v", LEGACY_APP_VALUE_NAME, "/f"])
                .status();
            Ok(())
        }
    }

    /// Check whether XMCompanion is currently registered in HKCU Run.
    pub fn is_enabled() -> bool {
        let current_ok = Command::new("reg")
            .args(["query", RUN_REG_KEY, "/v", APP_VALUE_NAME])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if current_ok {
            return true;
        }
        Command::new("reg")
            .args(["query", RUN_REG_KEY, "/v", LEGACY_APP_VALUE_NAME])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
