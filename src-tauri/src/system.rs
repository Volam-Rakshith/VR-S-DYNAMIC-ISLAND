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

#[cfg(windows)]
#[link(name = "winmm")]
extern "system" { fn PlaySoundW(sound: *const u16, module: *mut std::ffi::c_void, flags: u32) -> i32; }

#[tauri::command]
pub fn play_timer_sound(sound: String) -> Result<(), String> {
    let wave: &'static [u8] = match sound.as_str() {
        "silent" => return Ok(()),
        "default" => include_bytes!("../assets/timer-default.wav"),
        "information" => include_bytes!("../assets/timer-information.wav"),
        "warning" => include_bytes!("../assets/timer-warning.wav"),
        _ => return Err("Unsupported alarm sound".into()),
    };
    #[cfg(windows)] {
        // Static embedded WAV remains valid while asynchronous playback runs.
        const SND_ASYNC: u32 = 1; const SND_NODEFAULT: u32 = 2; const SND_MEMORY: u32 = 4;
        if unsafe { PlaySoundW(wave.as_ptr().cast(), std::ptr::null_mut(), SND_ASYNC | SND_NODEFAULT | SND_MEMORY) } == 0 {
            return Err("Windows could not play the timer sound; check the output device and system volume".into());
        }
        Ok(())
    }
    #[cfg(not(windows))] { let _ = wave; Err("Windows is required".into()) }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDetails { kind: String, connectivity: String, wifi_name: Option<String> }

#[tauri::command]
pub fn get_network_details(include_wifi_name: bool) -> Result<NetworkDetails, String> {
    #[cfg(windows)] {
        use windows::Networking::Connectivity::{NetworkInformation, NetworkConnectivityLevel};
        let profile = NetworkInformation::GetInternetConnectionProfile().map_err(|_| "No preferred connection available".to_string())?;
        let wifi = profile.IsWlanConnectionProfile().map_err(|e| e.to_string())?;
        let level = profile.GetNetworkConnectivityLevel().map_err(|e| e.to_string())?;
        let connectivity = if level == NetworkConnectivityLevel::InternetAccess { "Internet access reported" }
            else if level == NetworkConnectivityLevel::LocalAccess { "Local network only" }
            else if level == NetworkConnectivityLevel::ConstrainedInternetAccess { "Limited Internet access" }
            else { "No connection reported" };
        // The user must opt in to display the SSID. We never store or transmit it.
        let wifi_name = if wifi && include_wifi_name { profile.WlanConnectionProfileDetails().ok().and_then(|w| w.GetConnectedSsid().ok()).map(|s| s.to_string()).filter(|s| !s.is_empty()) } else { None };
        Ok(NetworkDetails { kind: if wifi { "Wi-Fi" } else { "Network" }.into(), connectivity: connectivity.into(), wifi_name })
    }
    #[cfg(not(windows))] { let _ = include_wifi_name; Err("Windows is required".into()) }
}
