# VR Island

**Your Windows. Your Island.**  
Made by **VR DEVELOPMENTS**.

VR Island is an open-source Windows 10/11 x64 desktop island project built with Tauri 2, Rust, React, TypeScript, Vite, Tailwind CSS, and Lucide. No account, subscription, paid API, analytics, or cloud service is required.

> **Status: early development, not release-ready.** This repository is a partial foundation, not the complete product described in the project vision. No Windows build, installer, or integration has been verified. Do not use it as a finished application.

## Implemented foundation

- Frameless, transparent, always-on-top, taskbar-free window configuration with initial top-centre placement on the current monitor.
- Compact clock and expandable interface, local focus timer (one timer, with in-app completion notice), and tray commands for show, hide, settings, and exit.
- Local appearance settings with validation, import, export and defaults. Themes: dark, light, black.
- Windows CI workflow configured to attempt NSIS and MSI builds and upload artifacts.

These desktop features are **implemented but unverified on Windows**. Timer state is lost when the application exits; the completion notice is shown inside the island, not as a native notification. Window resizing is not animated; only the panel's appearance is animated. The window's top-left remains fixed on expansion, so expanded mode is not centred. Settings writes are not atomic. The browser development view is only for frontend development and is not a supported deployment target; it does not persist settings or use the desktop features.

## Not yet implemented

Windows media sessions and playback controls, volume, battery, notification centre, clipboard history, downloads, microphone activity, networking, brightness, startup, fullscreen handling, multiple-monitor selection, positioning persistence, configurable dimensions and auto-hide, widget/plugin architecture, multiple timers and reminders, promotion playback/management, automatic Windows theme, additional themes, native update checking, release automation, portable builds, and the remaining requested tray/settings controls. No data for these features is simulated. The `promotions/` directory is only a reserved placeholder; **no videos are detected or played**. See [project status](docs/STATUS.md).

## Requirements

Windows 10 or 11 64-bit, WebView2 runtime, Node.js 20+, Rust stable and Microsoft's C++ build tools. See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for setup. These are development requirements; installer prerequisites must be verified during Windows testing.

## Build and run

In PowerShell, from this directory:

```powershell
npm ci
npm run tauri dev
npm run check
npm test
npm run tauri build
```

The Tauri build is configured to request NSIS and MSI bundles under `src-tauri/target/release/bundle/`. Successful generation has not been verified. `npm run build` alone builds the frontend, not a Windows desktop application. The Windows Actions workflow publishes build artifacts, not GitHub Releases. It consumes GitHub Actions minutes and artifact storage.

## Usage and settings

Click the pill to expand; use the close button to collapse. Choose a focus timer preset and start or pause it. In settings, adjust theme, accent, clock seconds, and appearance duration; export or import a validated JSON configuration, or restore defaults. The Windows tray menu is intended to let you hide, show, open settings or exit. All settings are stored in Tauri's local application configuration directory. Existing invalid settings are reported rather than silently overwritten.

## Promotion setup

The `promotions/` folder is reserved for a future opt-in local promotion feature. Adding files currently does nothing. Do not include copyrighted media without permission. No third-party ad network, tracking, or bundled videos are included.

## Troubleshooting

- If the build fails, confirm the Windows Tauri prerequisites and current WebView2 installation.
- If settings cannot be read or saved, inspect the in-app error and permissions for the local app configuration directory.
- Window layering, transparency and system tray behavior require testing on the target Windows version and hardware.

## Screenshots

No verified Windows screenshots are available yet.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Windows testing and narrowly scoped pull requests are welcome. Review third-party dependency licenses before distributing a release.

## License

MIT; see [LICENSE](LICENSE). Dependencies have their own licenses. VR Island is made by VR DEVELOPMENTS. This project is not affiliated with Apple or Microsoft.
