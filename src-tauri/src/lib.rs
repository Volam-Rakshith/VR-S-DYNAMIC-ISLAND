mod system;
mod audio;
mod media;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, menu::{Menu, MenuItem}, tray::TrayIconBuilder};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
struct Settings { theme: String, accent: String, show_seconds: bool, animation_ms: u16, animation: String, position: String, compact_width: u16, compact_height: u16, opacity: u8, show_media: bool, show_volume: bool, show_power: bool, show_network: bool, widget_order: Vec<String>, clock_format: String, corner_radius: u8 }
impl Default for Settings {
 fn default() -> Self { Self { theme: "dark".into(), accent: "#a4f5c8".into(), show_seconds: false, animation_ms: 240, animation: "island-spring".into(), position: "top-center".into(), compact_width: 260, compact_height: 52, opacity: 100, show_media: true, show_volume: true, show_power: true, show_network: true, widget_order: vec!["media".into(), "volume".into(), "system".into()], clock_format: "system".into(), corner_radius: 25 } }
}
impl Settings {
 fn valid(&self) -> bool { ["dark", "light", "black", "midnight", "ocean", "forest", "lavender", "rose", "sunset", "slate", "sand", "mint", "contrast", "glass", "neo"].contains(&self.theme.as_str()) && self.accent.len() == 7 && self.accent.starts_with('#') && self.accent[1..].bytes().all(|b| b.is_ascii_hexdigit()) && self.animation_ms <= 1000 && ["island-spring", "fade", "scale", "slide-down", "slide-up", "slide-left", "slide-right", "bounce", "soft-pop", "flip", "blur", "zoom-out", "rotate", "swing", "stretch", "none"].contains(&self.animation.as_str()) && ["top-center", "top-left", "top-right", "bottom-center", "bottom-left", "bottom-right"].contains(&self.position.as_str()) && (240..=400).contains(&self.compact_width) && (48..=80).contains(&self.compact_height) && (40..=100).contains(&self.opacity) && self.widget_order.len() == 3 && ["media", "volume", "system"].iter().all(|id| self.widget_order.iter().filter(|item| item.as_str() == *id).count() == 1) && ["system", "12", "24"].contains(&self.clock_format.as_str()) && (14..=40).contains(&self.corner_radius) }
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
#[tauri::command]
fn apply_layout(window: tauri::WebviewWindow, settings: Settings, expanded: bool) -> Result<(), String> {
 if !settings.valid() { return Err("Invalid layout settings".into()); }
 let monitor = window.current_monitor().map_err(|e| e.to_string())?.ok_or("No monitor available")?;
 let scale = monitor.scale_factor();
 let (width, height) = if expanded { (420_u32, 520_u32) } else { (settings.compact_width as u32, settings.compact_height as u32) };
 window.set_size(tauri::LogicalSize::new(width, height)).map_err(|e| e.to_string())?;
 let area = monitor.work_area();
 let w = (width as f64 * scale).round() as i32;
 let h = (height as f64 * scale).round() as i32;
 let margin = (12.0 * scale).round() as i32;
 let x = if settings.position.ends_with("left") { area.position.x + margin } else if settings.position.ends_with("right") { area.position.x + area.size.width as i32 - w - margin } else { area.position.x + (area.size.width as i32 - w) / 2 };
 let y = if settings.position.starts_with("bottom") { area.position.y + area.size.height as i32 - h - margin } else { area.position.y + margin };
 window.set_position(tauri::PhysicalPosition::new(x, y)).map_err(|e| e.to_string())
}
pub fn run() {
 tauri::Builder::default().invoke_handler(tauri::generate_handler![load_settings, save_settings, apply_layout, system::get_system_status, audio::get_volume, audio::set_volume, audio::set_mute, media::media_snapshot, media::media_action]).setup(|app| {
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
