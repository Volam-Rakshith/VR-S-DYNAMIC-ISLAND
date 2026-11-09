//! Windows default output endpoint volume. No audio recording or transmission.
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeStatus { pub percent: u8, pub muted: bool }

#[cfg(windows)]
fn with_endpoint<T>(f: impl FnOnce(&windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume) -> windows::core::Result<T>) -> Result<T, String> {
    use windows::Win32::{Media::Audio::{Endpoints::IAudioEndpointVolume, eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator}, System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED}};
    // Every successful CoInitializeEx, including S_FALSE, must be balanced on this thread.
    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    // UI threads may already be initialized as STA. RPC_E_CHANGED_MODE means
    // COM is usable in the existing apartment; do not uninitialize that setup.
    let owns_init = initialized.is_ok();
    if !owns_init && initialized.0 as u32 != 0x80010106 { return Err(initialized.to_string()); }
    struct ComGuard(bool);
    impl Drop for ComGuard { fn drop(&mut self) { if self.0 { unsafe { CoUninitialize() } } } }
    let _guard = ComGuard(owns_init);
    unsafe {
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| e.to_string())?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).map_err(|e| e.to_string())?;
        let endpoint: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).map_err(|e| e.to_string())?;
        f(&endpoint).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn get_volume() -> Result<VolumeStatus, String> {
    #[cfg(windows)] { with_endpoint(|endpoint| unsafe {
        Ok(VolumeStatus { percent: (endpoint.GetMasterVolumeLevelScalar()?.clamp(0.0, 1.0) * 100.0).round() as u8, muted: endpoint.GetMute()?.as_bool() })
    }) }
    #[cfg(not(windows))] { Err("Windows is required".into()) }
}

#[tauri::command]
pub fn set_volume(percent: u8) -> Result<VolumeStatus, String> {
    if percent > 100 { return Err("Volume must be between 0 and 100".into()); }
    #[cfg(windows)] { with_endpoint(|endpoint| unsafe {
        endpoint.SetMasterVolumeLevelScalar(percent as f32 / 100.0, std::ptr::null())?;
        Ok(VolumeStatus { percent: (endpoint.GetMasterVolumeLevelScalar()?.clamp(0.0, 1.0) * 100.0).round() as u8, muted: endpoint.GetMute()?.as_bool() })
    }) }
    #[cfg(not(windows))] { Err("Windows is required".into()) }
}

#[tauri::command]
pub fn set_mute(muted: bool) -> Result<VolumeStatus, String> {
    #[cfg(windows)] { with_endpoint(|endpoint| unsafe {
        endpoint.SetMute(muted, std::ptr::null())?;
        Ok(VolumeStatus { percent: (endpoint.GetMasterVolumeLevelScalar()?.clamp(0.0, 1.0) * 100.0).round() as u8, muted: endpoint.GetMute()?.as_bool() })
    }) }
    #[cfg(not(windows))] { let _ = muted; Err("Windows is required".into()) }
}
