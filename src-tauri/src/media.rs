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
