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

// State is shared across start and dismiss commands. A generation number lets
// dismissal cancel a start while a slow media app is still responding.
#[cfg(windows)]
struct PausedSession { session: windows::Media::Control::GlobalSystemMediaTransportControlsSession, source: String, title: String }
#[derive(Default)]
pub struct AlarmController { inner: std::sync::Mutex<AlarmState> }
#[derive(Default)]
struct AlarmState { generation: u64, active: bool, #[cfg(windows)] paused: Option<PausedSession> }

#[cfg(windows)]
async fn resume_if_unchanged(paused: PausedSession) {
    use windows::Media::Control::{GlobalSystemMediaTransportControlsSessionManager as Manager, GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status};
    let mut same = false;
    if let Ok(op) = Manager::RequestAsync() {
        if let Ok(manager) = op.await {
            if let Ok(current) = manager.GetCurrentSession() {
                if current.SourceAppUserModelId().map(|v| v.to_string() == paused.source).unwrap_or(false) {
                    if let Ok(op) = current.TryGetMediaPropertiesAsync() {
                        if let Ok(props) = op.await { same = props.Title().map(|v| v.to_string() == paused.title).unwrap_or(false); }
                    }
                }
            }
        }
    }
    let is_paused = paused.session.GetPlaybackInfo().and_then(|p| p.PlaybackStatus()).map(|v| v == Status::Paused).unwrap_or(false);
    if same && is_paused { if let Ok(op) = paused.session.TryPlayAsync() { let _ = op.await; } }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmOutcome { warning: Option<String> }

#[tauri::command]
pub async fn play_timer_alarm(app: tauri::AppHandle, controller: tauri::State<'_, AlarmController>, sound: String) -> Result<AlarmOutcome, String> {
    if sound == "silent" { return Ok(AlarmOutcome { warning: None }); }
    let generation = {
        let mut state = controller.inner.lock().map_err(|_| "Alarm state unavailable")?;
        if state.active { return Ok(AlarmOutcome { warning: None }); }
        state.generation = state.generation.wrapping_add(1); state.active = true; state.generation
    };
    #[cfg(windows)] {
        use windows::Media::Control::{GlobalSystemMediaTransportControlsSessionManager as Manager, GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status};
        let mut paused = None;
        let mut warning = None;
        if let Ok(op) = Manager::RequestAsync() {
            if let Ok(manager) = op.await {
                if let Ok(session) = manager.GetCurrentSession() {
                    let is_playing = session.GetPlaybackInfo().and_then(|i| i.PlaybackStatus()).map(|s| s == Status::Playing).unwrap_or(false);
                    if is_playing {
                        // Do not require IsPlayEnabled while already playing: many players
                        // disable Play until after they have paused.
                        if let (Ok(source), Ok(op)) = (session.SourceAppUserModelId(), session.TryGetMediaPropertiesAsync()) {
                            if let Ok(props) = op.await {
                                if let Ok(title) = props.Title() {
                                    let mut accepted = match session.TryPauseAsync() { Ok(op) => op.await.unwrap_or(false), Err(_) => false };
                                    if !accepted {
                                        let can_toggle = session.GetPlaybackInfo().and_then(|i| i.Controls()).and_then(|c| c.IsPlayPauseToggleEnabled()).unwrap_or(false);
                                        if can_toggle { accepted = match session.TryTogglePlayPauseAsync() { Ok(op) => op.await.unwrap_or(false), Err(_) => false }; }
                                    }
                                    if accepted {
                                        // The return value means the request was accepted, not
                                        // that the player has already stopped emitting audio.
                                        let mut confirmed = false;
                                        for _ in 0..10 {
                                            if session.GetPlaybackInfo().and_then(|i| i.PlaybackStatus()).map(|v| v == Status::Paused).unwrap_or(false) { confirmed = true; break; }
                                            let _ = tauri::async_runtime::spawn_blocking(|| std::thread::sleep(std::time::Duration::from_millis(100))).await;
                                        }
                                        if !confirmed { warning = Some("The player accepted pause but did not confirm it; music may overlap the alarm.".to_string()); }
                                        paused = Some(PausedSession { session, source: source.to_string(), title: title.to_string() });
                                    }
                                }
                            }
                        }
                        if paused.is_none() { warning = Some("Windows could not pause the active player; music may overlap the alarm.".to_string()); }
                    }
                }
            }
        }
        // No await while holding the lock. If dismissed during a slow pause,
        // immediately restore the player instead of starting a late alarm.
        let (cancelled, failed) = {
            let mut state = controller.inner.lock().map_err(|_| "Alarm state unavailable")?;
            if !state.active || state.generation != generation { (true, None) }
            else {
                state.paused = paused.take();
                match crate::system::start_sound(&app, &sound, true) {
                    Ok(()) => (false, None),
                    Err(e) => { state.active = false; paused = state.paused.take(); (false, Some(e)) }
                }
            }
        };
        if let Some(session) = paused { resume_if_unchanged(session).await; }
        if cancelled { return Ok(AlarmOutcome { warning: None }); }
        if let Some(error) = failed { return Err(error); }
        Ok(AlarmOutcome { warning })
    }
    #[cfg(not(windows))] { let _ = (app, sound, generation); Err("Windows is required".into()) }
}

#[tauri::command]
pub async fn stop_timer_alarm(controller: tauri::State<'_, AlarmController>) -> Result<(), String> {
    #[cfg(windows)] {
        let paused = {
            let mut state = controller.inner.lock().map_err(|_| "Alarm state unavailable")?;
            state.active = false; state.generation = state.generation.wrapping_add(1);
            crate::system::stop_sound(); state.paused.take()
        };
        if let Some(session) = paused { resume_if_unchanged(session).await; }
        Ok(())
    }
    #[cfg(not(windows))] { let _ = controller; Err("Windows is required".into()) }
}
