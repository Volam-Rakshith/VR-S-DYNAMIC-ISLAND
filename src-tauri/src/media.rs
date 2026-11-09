//! Read and control only the current Windows media session.
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSnapshot {
    source: String, title: String, artist: String, album: String,
    playing: bool, can_toggle: bool, can_previous: bool, can_next: bool,
    position_seconds: Option<u64>, duration_seconds: Option<u64>,
}

#[tauri::command]
pub async fn media_snapshot() -> Result<Option<MediaSnapshot>, String> {
    #[cfg(windows)] {
        use windows::Media::Control::{GlobalSystemMediaTransportControlsSessionManager as Manager, GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status};
        let manager = Manager::RequestAsync().map_err(|e| e.to_string())?.await.map_err(|e| e.to_string())?;
        // Windows represents no current session as a failed null conversion.
        let session = match manager.GetCurrentSession() { Ok(s) => s, Err(_) => return Ok(None) };
        let source = session.SourceAppUserModelId().map_err(|e| e.to_string())?.to_string();
        let properties = session.TryGetMediaPropertiesAsync().map_err(|e| e.to_string())?.await.map_err(|e| e.to_string())?;
        let info = session.GetPlaybackInfo().map_err(|e| e.to_string())?;
        let controls = info.Controls().map_err(|e| e.to_string())?;
        let timeline = session.GetTimelineProperties().ok();
        let position_seconds = timeline.as_ref().and_then(|t| t.Position().ok()).and_then(|v| u64::try_from(v.Duration).ok()).map(|v| v / 10_000_000);
        let duration_seconds = timeline.as_ref().and_then(|t| t.EndTime().ok()).and_then(|v| u64::try_from(v.Duration).ok()).map(|v| v / 10_000_000).filter(|v| *v > 0);
        Ok(Some(MediaSnapshot {
            source, title: properties.Title().unwrap_or_default().to_string(), artist: properties.Artist().unwrap_or_default().to_string(), album: properties.AlbumTitle().unwrap_or_default().to_string(),
            playing: info.PlaybackStatus().map(|s| s == Status::Playing).unwrap_or(false),
            can_toggle: controls.IsPlayPauseToggleEnabled().unwrap_or(false),
            can_previous: controls.IsPreviousEnabled().unwrap_or(false), can_next: controls.IsNextEnabled().unwrap_or(false),
            position_seconds, duration_seconds,
        }))
    }
    #[cfg(not(windows))] { Err("Windows is required".into()) }
}

#[tauri::command]
pub async fn media_action(source: String, action: String) -> Result<(), String> {
    #[cfg(windows)] {
        use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as Manager;
        let manager = Manager::RequestAsync().map_err(|e| e.to_string())?.await.map_err(|e| e.to_string())?;
        let session = manager.GetCurrentSession().map_err(|_| "No active media session".to_string())?;
        if session.SourceAppUserModelId().map_err(|e| e.to_string())?.to_string() != source { return Err("The active player changed; refresh and try again".into()); }
        let controls = session.GetPlaybackInfo().and_then(|i| i.Controls()).map_err(|e| e.to_string())?;
        let accepted = match action.as_str() {
            "toggle" if controls.IsPlayPauseToggleEnabled().unwrap_or(false) => session.TryTogglePlayPauseAsync().map_err(|e| e.to_string())?.await.map_err(|e| e.to_string())?,
            "previous" if controls.IsPreviousEnabled().unwrap_or(false) => session.TrySkipPreviousAsync().map_err(|e| e.to_string())?.await.map_err(|e| e.to_string())?,
            "next" if controls.IsNextEnabled().unwrap_or(false) => session.TrySkipNextAsync().map_err(|e| e.to_string())?.await.map_err(|e| e.to_string())?,
            _ => return Err("This player does not support that action".into()),
        };
        if accepted { Ok(()) } else { Err("The player declined the request".into()) }
    }
    #[cfg(not(windows))] { let _ = (source, action); Err("Windows is required".into()) }
}

// Only one alarm sound sequence runs at a time. Other completed timers remain
// in the frontend's alert queue; they must not race to resume playback.
static ALARM_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
struct AlarmGuard;
impl Drop for AlarmGuard {
    fn drop(&mut self) { ALARM_ACTIVE.store(false, std::sync::atomic::Ordering::Release); }
}

/// Briefly pauses the current playing session, sounds the local alarm, and
/// attempts to resume only the same still-paused track. A player can refuse a
/// command, disappear, or change tracks; in these cases sound still plays.
#[tauri::command]
pub async fn play_timer_alarm(sound: String) -> Result<(), String> {
    if sound == "silent" { return Ok(()); }
    if ALARM_ACTIVE.compare_exchange(false, true, std::sync::atomic::Ordering::Acquire, std::sync::atomic::Ordering::Relaxed).is_err() { return Ok(()); }
    let _guard = AlarmGuard;
    #[cfg(windows)] {
        use windows::Media::Control::{GlobalSystemMediaTransportControlsSessionManager as Manager, GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status};
        // Failure to access media never prevents the alarm from sounding.
        let mut paused: Option<(windows::Media::Control::GlobalSystemMediaTransportControlsSession, String, String)> = None;
        if let Ok(op) = Manager::RequestAsync() {
            if let Ok(manager) = op.await {
                if let Ok(session) = manager.GetCurrentSession() {
                    let info = session.GetPlaybackInfo();
                    let eligible = info.as_ref().ok().and_then(|i| i.PlaybackStatus().ok()).map(|s| s == Status::Playing).unwrap_or(false)
                        && info.as_ref().ok().and_then(|i| i.Controls().ok()).map(|c| c.IsPauseEnabled().unwrap_or(false) && c.IsPlayEnabled().unwrap_or(false)).unwrap_or(false);
                    if eligible {
                        if let (Ok(source), Ok(properties_op)) = (session.SourceAppUserModelId(), session.TryGetMediaPropertiesAsync()) {
                            if let Ok(properties) = properties_op.await {
                                if let Ok(title) = properties.Title() {
                                    if let Ok(pause_op) = session.TryPauseAsync() {
                                        if pause_op.await.unwrap_or(false) { paused = Some((session, source.to_string(), title.to_string())); }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let sound_result = crate::system::play_timer_sound(sound);
        // Embedded tones last 380 ms. Give the Windows mixer time to finish.
        if sound_result.is_ok() {
            let _ = tauri::async_runtime::spawn_blocking(|| std::thread::sleep(std::time::Duration::from_millis(550))).await;
        }
        if let Some((session, original_source, original_title)) = paused {
            // Confirm that Windows still considers the same player/track active,
            // and do not override a player that has already resumed or stopped.
            let mut still_current = false;
            if let Ok(op) = Manager::RequestAsync() {
                if let Ok(manager) = op.await {
                    if let Ok(current) = manager.GetCurrentSession() {
                        if current.SourceAppUserModelId().map(|v| v.to_string() == original_source).unwrap_or(false) {
                            if let Ok(props_op) = current.TryGetMediaPropertiesAsync() {
                                if let Ok(props) = props_op.await {
                                    still_current = props.Title().map(|v| v.to_string() == original_title).unwrap_or(false);
                                }
                            }
                        }
                    }
                }
            }
            let still_paused = session.GetPlaybackInfo().and_then(|v| v.PlaybackStatus()).map(|v| v == Status::Paused).unwrap_or(false);
            if still_current && still_paused {
                if let Ok(op) = session.TryPlayAsync() { let _ = op.await; }
            }
        }
        sound_result
    }
    #[cfg(not(windows))] { let _ = sound; Err("Windows is required".into()) }
}
