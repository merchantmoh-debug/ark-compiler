/*
 * Copyright (c) 2026 Mohamad Al-Zawahreh (dba Sovereign Systems).
 *
 * This file is part of the Ark Sovereign Compiler.
 *
 * LICENSE: DUAL-LICENSED (AGPLv3 or COMMERCIAL).
 * PATENT NOTICE: Protected by US Patent App #63/935,467.
 *
 * Desktop FFI — OS-level desktop automation tools.
 *
 * Tier 1 (Implemented):
 *   clipboard_read/write, open_app/close_app, browser_open,
 *   system_health, system_storage, shutdown, restart, sleep
 *
 * Tier 2 (Stubbed — needs complex platform APIs):
 *   screenshot, webcam, audio_record, volume, brightness,
 *   media_play/pause/next, caffeine, focus_mode, recycle_bin, panic
 *
 * All functions are #[cfg(not(target_arch = "wasm32"))].
 * WASM builds get error stubs via the existing intrinsic_desktop_stub.
 */

use serde::{Deserialize, Serialize};

// ===========================================================================
// Data Types
// ===========================================================================

/// System health snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub cpu_usage_percent: f64,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub memory_free_mb: u64,
    pub uptime_secs: u64,
    pub os_name: String,
    pub hostname: String,
}

/// Disk usage info for a single mount point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub free_gb: f64,
    pub usage_percent: f64,
}

// ===========================================================================
// Tier 1 Implementations (non-WASM only)
// ===========================================================================

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use std::process::Command;

    // --- Clipboard ---

    /// Read text from the system clipboard.
    pub fn clipboard_read() -> Result<String, String> {
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| format!("Clipboard init failed: {}", e))?;
        clipboard
            .get_text()
            .map_err(|e| format!("Clipboard read failed: {}", e))
    }

    /// Write text to the system clipboard.
    pub fn clipboard_write(text: &str) -> Result<(), String> {
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| format!("Clipboard init failed: {}", e))?;
        clipboard
            .set_text(text.to_string())
            .map_err(|e| format!("Clipboard write failed: {}", e))
    }

    // --- Application Management ---

    /// Launch an application by name or path.
    pub fn open_app(name: &str) -> Result<String, String> {
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("cmd")
                .args(["/C", "start", "", name])
                .spawn()
                .map_err(|e| format!("Failed to launch '{}': {}", name, e))?;
            Ok(format!("Launched '{}' (PID: {})", name, output.id()))
        }

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("xdg-open")
                .arg(name)
                .spawn()
                .map_err(|e| format!("Failed to launch '{}': {}", name, e))?;
            Ok(format!("Launched '{}' (PID: {})", name, output.id()))
        }

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("open")
                .arg("-a")
                .arg(name)
                .spawn()
                .map_err(|e| format!("Failed to launch '{}': {}", name, e))?;
            Ok(format!("Launched '{}' (PID: {})", name, output.id()))
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Err(format!("Unsupported OS for open_app"))
        }
    }

    /// Close/kill an application by name.
    pub fn close_app(name: &str) -> Result<String, String> {
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("taskkill")
                .args(["/IM", name, "/F"])
                .output()
                .map_err(|e| format!("Failed to kill '{}': {}", name, e))?;
            if output.status.success() {
                Ok(format!("Killed '{}'", name))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("taskkill failed for '{}': {}", name, stderr.trim()))
            }
        }

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("pkill")
                .arg("-f")
                .arg(name)
                .output()
                .map_err(|e| format!("Failed to kill '{}': {}", name, e))?;
            if output.status.success() {
                Ok(format!("Killed '{}'", name))
            } else {
                Err(format!("pkill failed for '{}'", name))
            }
        }

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("pkill")
                .arg("-f")
                .arg(name)
                .output()
                .map_err(|e| format!("Failed to kill '{}': {}", name, e))?;
            if output.status.success() {
                Ok(format!("Killed '{}'", name))
            } else {
                Err(format!("pkill failed for '{}'", name))
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Err(format!("Unsupported OS for close_app"))
        }
    }

    // --- Browser ---

    /// Open a URL in the default browser.
    pub fn browser_open(url: &str) -> Result<(), String> {
        open::that(url).map_err(|e| format!("Failed to open '{}': {}", url, e))
    }

    // --- System Info ---

    /// Get system health snapshot (CPU, memory, uptime, OS).
    pub fn system_health() -> Result<SystemHealth, String> {
        use sysinfo::System;

        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_usage() as f64;
        let total_mem = sys.total_memory() / (1024 * 1024); // bytes → MB
        let used_mem = sys.used_memory() / (1024 * 1024);
        let free_mem = total_mem.saturating_sub(used_mem);

        Ok(SystemHealth {
            cpu_usage_percent: cpu_usage,
            memory_total_mb: total_mem,
            memory_used_mb: used_mem,
            memory_free_mb: free_mem,
            uptime_secs: System::uptime(),
            os_name: System::long_os_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        })
    }

    /// Get disk usage for all mounted volumes.
    pub fn system_storage() -> Result<Vec<DiskInfo>, String> {
        use sysinfo::Disks;

        let disks = Disks::new_with_refreshed_list();
        let mut result = Vec::new();

        for disk in disks.list() {
            let total = disk.total_space() as f64 / (1024.0 * 1024.0 * 1024.0);
            let free = disk.available_space() as f64 / (1024.0 * 1024.0 * 1024.0);
            let used = total - free;
            let usage_pct = if total > 0.0 {
                (used / total) * 100.0
            } else {
                0.0
            };

            result.push(DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_gb: total,
                used_gb: used,
                free_gb: free,
                usage_percent: usage_pct,
            });
        }

        Ok(result)
    }

    // --- Power Management ---

    /// Shut down the system.
    pub fn power_shutdown() -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            Command::new("shutdown")
                .args(["/s", "/t", "5", "/c", "Ark Sovereign shutdown"])
                .spawn()
                .map_err(|e| format!("Shutdown failed: {}", e))?;
            Ok(())
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("shutdown")
                .args(["-h", "now"])
                .spawn()
                .map_err(|e| format!("Shutdown failed: {}", e))?;
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("osascript")
                .args(["-e", "tell app \"System Events\" to shut down"])
                .spawn()
                .map_err(|e| format!("Shutdown failed: {}", e))?;
            Ok(())
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Err("Unsupported OS for shutdown".to_string())
        }
    }

    /// Restart the system.
    pub fn power_restart() -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            Command::new("shutdown")
                .args(["/r", "/t", "5", "/c", "Ark Sovereign restart"])
                .spawn()
                .map_err(|e| format!("Restart failed: {}", e))?;
            Ok(())
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("shutdown")
                .args(["-r", "now"])
                .spawn()
                .map_err(|e| format!("Restart failed: {}", e))?;
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("osascript")
                .args(["-e", "tell app \"System Events\" to restart"])
                .spawn()
                .map_err(|e| format!("Restart failed: {}", e))?;
            Ok(())
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Err("Unsupported OS for restart".to_string())
        }
    }

    /// Put the system to sleep.
    pub fn power_sleep() -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            Command::new("rundll32.exe")
                .args(["powrprof.dll,SetSuspendState", "0,1,0"])
                .spawn()
                .map_err(|e| format!("Sleep failed: {}", e))?;
            Ok(())
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("systemctl")
                .arg("suspend")
                .spawn()
                .map_err(|e| format!("Sleep failed: {}", e))?;
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("pmset")
                .arg("sleepnow")
                .spawn()
                .map_err(|e| format!("Sleep failed: {}", e))?;
            Ok(())
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Err("Unsupported OS for sleep".to_string())
        }
    }
}

// ===========================================================================
// Public API (delegates to native:: or returns stub error)
// ===========================================================================

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

// WASM stubs
#[cfg(target_arch = "wasm32")]
pub fn clipboard_read() -> Result<String, String> {
    Err("Desktop FFI not available in WASM".to_string())
}
#[cfg(target_arch = "wasm32")]
pub fn clipboard_write(_text: &str) -> Result<(), String> {
    Err("Desktop FFI not available in WASM".to_string())
}
#[cfg(target_arch = "wasm32")]
pub fn open_app(_name: &str) -> Result<String, String> {
    Err("Desktop FFI not available in WASM".to_string())
}
#[cfg(target_arch = "wasm32")]
pub fn close_app(_name: &str) -> Result<String, String> {
    Err("Desktop FFI not available in WASM".to_string())
}
#[cfg(target_arch = "wasm32")]
pub fn browser_open(_url: &str) -> Result<(), String> {
    Err("Desktop FFI not available in WASM".to_string())
}
#[cfg(target_arch = "wasm32")]
pub fn system_health() -> Result<SystemHealth, String> {
    Err("Desktop FFI not available in WASM".to_string())
}
#[cfg(target_arch = "wasm32")]
pub fn system_storage() -> Result<Vec<DiskInfo>, String> {
    Err("Desktop FFI not available in WASM".to_string())
}
#[cfg(target_arch = "wasm32")]
pub fn power_shutdown() -> Result<(), String> {
    Err("Desktop FFI not available in WASM".to_string())
}
#[cfg(target_arch = "wasm32")]
pub fn power_restart() -> Result<(), String> {
    Err("Desktop FFI not available in WASM".to_string())
}
#[cfg(target_arch = "wasm32")]
pub fn power_sleep() -> Result<(), String> {
    Err("Desktop FFI not available in WASM".to_string())
}

/// Tier 2 stub: returns a descriptive error for unimplemented tools.
pub fn tier2_stub(tool_name: &str) -> Result<String, String> {
    Err(format!(
        "Desktop tool '{}' requires platform-specific FFI (not yet implemented). \
         Needs: win32 GDI / COM / DirectShow APIs.",
        tool_name
    ))
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn test_system_health_returns_valid() {
        let health = system_health().expect("system_health should succeed");
        assert!(health.memory_total_mb > 0);
        assert!(health.uptime_secs > 0);
        assert!(!health.os_name.is_empty());
        assert!(!health.hostname.is_empty());
    }

    #[test]
    fn test_system_storage_returns_disks() {
        let disks = system_storage().expect("system_storage should succeed");
        assert!(!disks.is_empty(), "At least one disk should exist");
        for disk in &disks {
            assert!(disk.total_gb > 0.0);
            assert!(disk.usage_percent >= 0.0 && disk.usage_percent <= 100.0);
        }
    }

    #[test]
    #[ignore] // Interacts with OS clipboard
    fn test_clipboard_round_trip() {
        let test_text = "Ark Sovereign Clipboard Test 🦅";
        clipboard_write(test_text).expect("clipboard_write should succeed");
        let read = clipboard_read().expect("clipboard_read should succeed");
        assert_eq!(read, test_text);
    }

    #[test]
    fn test_tier2_stub_returns_error() {
        let result = tier2_stub("screenshot");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("screenshot"));
    }

    #[test]
    #[ignore] // Would open a browser
    fn test_browser_open() {
        let result = browser_open("https://example.com");
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Would shutdown the system!
    fn test_power_shutdown() {
        let _ = power_shutdown();
    }
}
