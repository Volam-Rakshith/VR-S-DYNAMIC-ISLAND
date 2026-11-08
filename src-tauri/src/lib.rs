use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, menu::{Menu, MenuItem}, tray::TrayIconBuilder};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Settings { theme: String, accent: String, show_seconds: bool, animation_ms: u16 }
impl Default for Settings {
 fn default() -> Self { Self { theme: "dark".into(), accent: "#a4f5c8".into(), show_seconds: false, animation_ms: 240 } }
}
impl Settings {
 fn valid(&self) -> bool { ["dark", "light", "black"].contains(&self.theme.as_str()) && self.accent.len() == 7 && self.accent.starts_with('#') && self.accent[1..].bytes().all(|b| b.is_ascii_hexdigit()) && self.animation_ms <= 1000 }
}
#[tauri::command]
fn load_settings(app: tauri::AppHandle) -> Result<Settings, String> {
 let path = app.path().app_config_dir().map_err(|e| e.to_string())?.join("settings.json");
 match std::fs::read_to_string(path) { Ok(s) => { let settings: Settings = serde_json::from_str(&s).map_err(|e| e.to_string())?; if !settings.valid() { return Err("Invalid saved settings".into()); } Ok(settings) }, Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()), Err(e) => Err(e.to_string()) }
}
#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
 if !settings.valid() { return Err("Invalid settings".into()); }
 let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
 std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
 std::fs::write(dir.join("settings.json"), serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
pub fn run() {
 tauri::Builder::default().invoke_handler(tauri::generate_handler![load_settings, save_settings]).setup(|app| {
  let show = MenuItem::with_id(app, "show", "Show Island", true, None::<&str>)?;
  let hide = MenuItem::with_id(app, "hide", "Hide Island", true, None::<&str>)?;
  let settings = MenuItem::with_id(app, "settings", "Open Settings", true, None::<&str>)?;
  let exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
  let menu = Menu::with_items(app, &[&show, &hide, &settings, &exit])?;
  let mut tray = TrayIconBuilder::new().menu(&menu).show_menu_on_left_click(false).tooltip("VR Island").on_menu_event(|app, event| {
   if let Some(window) = app.get_webview_window("main") { match event.id.as_ref() { "show" => { let _ = window.show(); let _ = window.set_focus(); }, "hide" => { let _ = window.hide(); }, "settings" => { let _ = window.show(); let _ = window.set_focus(); let _ = window.emit("open-settings", ()); }, "exit" => app.exit(0), _ => {} } }
  });
  if let Some(icon) = app.default_window_icon() { tray = tray.icon(icon.clone()); }
  tray.build(app)?;
  if let Some(window) = app.get_webview_window("main") { let _ = window.center(); if let Ok(Some(monitor)) = window.current_monitor() { let size = monitor.size(); let origin = monitor.position(); let scale = monitor.scale_factor(); let width = (260.0 * scale) as i32; let _ = window.set_position(tauri::PhysicalPosition::new(origin.x + (size.width as i32 - width)/2, origin.y + (12.0 * scale) as i32)); } }
  Ok(())
 }).run(tauri::generate_context!()).expect("error while running VR Island");
}
