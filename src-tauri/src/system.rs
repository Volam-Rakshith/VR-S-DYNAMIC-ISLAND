//! Read-only Windows status. No identifiers are returned to the UI.
use serde::Serialize;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub battery_present: Option<bool>,
    pub battery_percent: Option<u8>,
    pub charging: Option<bool>,
    pub remaining_seconds: Option<u32>,
    pub connection_configured: Option<bool>,
}
#[cfg(windows)]
#[repr(C)]
struct PowerStatus { ac: u8, flags: u8, percent: u8, saver: u8, remaining: u32, full: u32 }
#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" { fn GetSystemPowerStatus(status: *mut PowerStatus) -> i32; }
#[cfg(windows)]
#[link(name = "wininet")]
extern "system" { fn InternetGetConnectedState(flags: *mut u32, reserved: u32) -> i32; }

#[tauri::command]
pub fn get_system_status(include_power: bool, include_network: bool) -> Result<SystemStatus, String> {
    #[cfg(windows)] {
        let (mut present, mut battery_percent, mut charging, mut remaining_seconds) = (None, None, None, None);
        if include_power {
            let mut p = PowerStatus { ac: 255, flags: 255, percent: 255, saver: 0, remaining: u32::MAX, full: u32::MAX };
            if unsafe { GetSystemPowerStatus(&mut p) } == 0 { return Err("Windows power status is unavailable".into()); }
            present = if p.flags == 255 { None } else { Some(p.flags & 128 == 0) };
            battery_percent = if present == Some(true) && p.percent <= 100 { Some(p.percent) } else { None };
            charging = if present == Some(true) && p.flags != 255 { Some(p.flags & 8 != 0) } else { None };
            remaining_seconds = if present == Some(true) && p.remaining != u32::MAX { Some(p.remaining) } else { None };
        }
        let connection_configured = if include_network {
            let mut flags = 0u32;
            // Configuration indicator only: this does not verify Internet reachability.
            Some(unsafe { InternetGetConnectedState(&mut flags, 0) } != 0)
        } else { None };
        Ok(SystemStatus { battery_present: present, battery_percent, charging, remaining_seconds, connection_configured })
    }
    #[cfg(not(windows))] { let _ = (include_power, include_network); Err("Windows is required".into()) }
}
