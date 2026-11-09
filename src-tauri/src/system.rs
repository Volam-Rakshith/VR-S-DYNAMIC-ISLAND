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
pub fn get_system_status() -> Result<SystemStatus, String> {
    #[cfg(windows)] {
        let mut p = PowerStatus { ac: 255, flags: 255, percent: 255, saver: 0, remaining: u32::MAX, full: u32::MAX };
        // GetSystemPowerStatus fills the supplied buffer; a zero result is failure.
        if unsafe { GetSystemPowerStatus(&mut p) } == 0 { return Err("Windows power status is unavailable".into()); }
        let present = if p.flags == 255 { None } else { Some(p.flags & 128 == 0) };
        let battery_percent = if present == Some(true) && p.percent <= 100 { Some(p.percent) } else { None };
        let charging = if present == Some(true) && p.flags != 255 { Some(p.flags & 8 != 0) } else { None };
        let remaining_seconds = if present == Some(true) && p.remaining != u32::MAX { Some(p.remaining) } else { None };
        let mut flags = 0u32;
        // This is a connection configuration indicator, not an Internet reachability test.
        let connection_configured = Some(unsafe { InternetGetConnectedState(&mut flags, 0) } != 0);
        Ok(SystemStatus { battery_present: present, battery_percent, charging, remaining_seconds, connection_configured })
    }
    #[cfg(not(windows))] { Err("Windows is required".into()) }
}
