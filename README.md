# Process Manager

A lightweight, high-performance Windows desktop application for managing long-running processes with real-time log monitoring, resource usage tracking, and system tray integration.

## Features

### Core
- ✅ Run multiple long-running commands in background
- ✅ Named process management with save/load configuration
- ✅ Start / Stop / Restart controls per process
- ✅ Real-time stdout/stderr log capturing
- ✅ Auto-restart on crash with configurable settings
- ✅ Process configuration persistence (JSON)
- ✅ Silent background execution (no console windows)

### UI
- ✅ Minimal dark-themed interface
- ✅ Sidebar with process list
- ✅ Main panel with real-time logs
- ✅ CPU and RAM per-process monitoring
- ✅ Process status indicators (Running/Stopped/Crashed)
- ✅ System tray integration with minimize-to-tray
- ✅ Bulk Start All / Stop All buttons

### Bonus Features
- ✅ Log persistence (auto-saved to files)
- ✅ Log file rotation when size exceeds threshold
- ✅ Auto-start selected processes on Windows boot
- ✅ Crash counter per process
- ✅ Uptime tracking
- ✅ Process-specific settings (auto-restart, auto-start toggles)

## Tech Stack

- **Backend**: Tauri 2.x, Rust
- **Frontend**: Vue 3, TypeScript, Pinia
- **State Management**: Pinia store
- **Process Management**: `std::process::Command` with tokio async
- **Monitoring**: sysinfo crate
- **OS Integration**: winreg (Windows registry)
- **Bundling**: NSIS installer (.exe)

## Project Structure

```
process-manager/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── types.rs             # Type definitions
│   │   ├── process_manager.rs   # Core process management
│   │   ├── log_handler.rs       # Log capture & persistence
│   │   ├── config_handler.rs    # Config file I/O
│   │   ├── monitoring.rs        # CPU/RAM tracking
│   │   ├── commands.rs          # Tauri command endpoints
│   │   ├── lib.rs              # App initialization & system tray
│   │   └── main.rs             # Entry point
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri configuration
│
├── src/                         # Vue frontend
│   ├── components/
│   │   ├── ProcessList.vue      # Sidebar: process listing & controls
│   │   └── ProcessPanel.vue     # Main: logs & process details
│   ├── stores/
│   │   └── processStore.ts      # Pinia state management
│   ├── types/
│   │   └── process.ts          # TypeScript interfaces
│   ├── App.vue                # Root layout component
│   └── main.ts                # Vue app entry point
│
├── package.json               # Frontend dependencies
├── tsconfig.json             # TypeScript config
└── vite.config.ts            # Vite build config
```

## Prerequisites

### Windows
- [Node.js 16+](https://nodejs.org/)
- [Rust 1.60+](https://www.rust-lang.org/tools/install)
- bun package manager
- Windows 10/11 (for WebView2 support)

### Build Tools
The first build will download:
- WebView2 runtime (~1.8 MB with embedBootstrapper)
- Tauri CLI dependencies

## Setup & Installation

### 1. Clone/Navigate to Project

```bash
cd process manager
```

### 2. Install Dependencies

```bash
# using bun (faster)
bun install
```

### 3. Verify Rust Setup

```bash
rustc --version  # Should be 1.60+
cargo --version
```

## Development

### Run in Dev Mode

```bash
# Frontend dev server + Tauri dev app open side-by-side
bun run tauri-dev

# Or individually:
bun run dev              # Vite frontend only (http://localhost:1420)
bun run tauri dev        # Backend only
```

After running `npm run tauri-dev`:
- Tauri app window opens on localhost:1420
- Vue components hot-reload
- Rust changes require full restart

### Frontend-Only Development

```bash
npm run dev
# Opens http://localhost:1420 in browser
# Backend still required for functionality
```

## Building

### Build for Windows Release

```bash
bun run tauri build
```

**Output**: `src-tauri/target/release/bundle/nsis/ProcessManager_X.X.X_x64-setup.exe`

**Size**: ~60-80 MB (includes embedded WebView2)

**Build takes**: 5-10 minutes (first time), 2-3 minutes (subsequent)

### Build Debug Binary

```bash
bun run tauri build -- --debug
```

## Usage

### Adding Processes

1. Enter process name and command in sidebar input
2. Click "+ Add"
3. Process appears in list with "Stopped" status

### Common Commands

- **Node.js**: `node script.js`
- **Python**: `python script.py`
- **FFmpeg**: `ffmpeg -i input.mp4 output.mp4`
- **PowerShell**: `pwsh -Command "Get-Process"`

### Starting & Monitoring

1. Click process in sidebar to select
2. Click ▶ button or use Start All
3. View real-time logs in main panel
4. Monitor CPU/RAM metrics
5. Click ⏹ to stop or ↻ to restart

### Configuration

**Save Configuration**:
- Modify process settings (auto-restart, name)
- Click ⚙ Settings
- Toggle options and "Save Config"
- Saves to: `%APPDATA%\ProcessManager\processes.json`

**Load Configuration**:
- Auto-loaded on app start
- Manually load with `load_config` command

**Auto-Start on Boot**:
- Toggle via Settings panel
- Writes to Windows registry: `HKEY_CURRENT_USER\...\Run`

### Logs

- **Real-time**: Streamed to UI as processes output
- **File Storage**: `%APPDATA%\ProcessManager\logs\{processId}.log`
- **Format**: `[YYYY-MM-DD HH:MM:SS.mmm] [stdout|stderr] message`
- **Limit**: 1000 lines displayed in UI
- **Persistence**: All logs automatically saved
- **Rotated**: When file exceeds 1 MB

## Configuration Files

### `%APPDATA%\ProcessManager\processes.json`

```json
[
  {
    "id": "uuid",
    "name": "My Node App",
    "command": "node",
    "args": ["server.js"],
    "auto_restart": true,
    "auto_start": false,
    "env": null
  }
]
```

### `%APPDATA%\ProcessManager\logs\{processId}.log`

Contains combined stdout + stderr for each process.

## API Reference

### Tauri Commands (Frontend → Backend)

**Process Control**
- `get_processes()` → `Process[]`
- `add_process(name, command, args)` → `processId`
- `remove_process(processId)` → `()`
- `start_process(processId)` → `pid`
- `stop_process(processId)` → `()`
- `restart_process(processId)` → `pid`
- `update_process(processId, autoRestart?, autoStart?)` → `()`

**Monitoring**
- `get_metrics(processId)` → `{ cpu_percent, memory_mb }`

**Logs**
- `get_logs(processId)` → `LogEntry[]`
- `clear_logs(processId)` → `()`

**Config**
- `save_config()` → `()`
- `load_config()` → `Process[]`
- `set_auto_start(enable)` → `()`

**Bulk Actions**
- `start_all()` → `()`
- `stop_all()` → `()`

### Events (Backend → Frontend)

- `process:status_changed` - Process status updated
- `log:{processId}` - New log entry

## Troubleshooting

### "WebView2 not found"
1. Download WebView2 runtime: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
2. Install and restart

### Processes not saving
1. Check: `%APPDATA%\ProcessManager\processes.json` exists
2. Verify write permissions to AppData folder
3. Try manual save via Settings > Save Config

### No logs appearing
1. Check process is actually running (status = "Running")
2. Verify process has stdout/stderr output
3. Check log files: `%APPDATA%\ProcessManager\logs\*.log`
4. Try refreshing logs in UI

### High CPU usage
1. Log streaming is real-time - normal if process outputs heavily
2. Limiting to 1000 logs in UI memory
3. Reduce polling (modify `main.ts` setInterval from 2000ms)

### App won't start
1. Verify Node.js installed: `node --version`
2. Verify Rust installed: `rustc --version`
3. Check Cargo.toml dependencies
4. Try: `npm install && bun run tauri-dev`

## Performance

- **Memory**: ~100-150 MB base + log buffer
- **CPU**: < 1% idle, scales with process I/O
- **Responsiveness**: 60 FPS UI (Vue reactive)
- **Log throughput**: Handles 10K+ lines/second

## Platform Support

- ✅ Windows 10/11 (primary target)
- ⚠️ Linux (untested, minor tweaks needed)
- ⚠️ macOS (untested, minor tweaks needed)

## Building for Other Platforms

Tauri supports cross-platform builds with minor code changes:

1. Remove Windows-specific imports in `commands.rs` (winreg)
2. Update `tauri.conf.json` bundler config
3. Adjust paths in `config_handler.rs`

## Contributing

To extend functionality:

1. Add Rust commands in `src-tauri/src/commands.rs`
2. Create Vue components in `src/components/`
3. Update Pinia store in `src/stores/processStore.ts`
4. Test in dev: `npm run tauri-dev`
5. Build and test release: `npm run tauri build`

## License

MIT - Feel free to modify and distribute

## Roadmap

- [ ] Process templates (Quick Add)
- [ ] Log filtering & search
- [ ] Process environment variables UI
- [ ] Process health checks
- [ ] Multi-process restart strategies
- [ ] Remote process monitoring (SSH)
- [ ] Discord/Webhook notifications on crash
- [ ] Performance graphs (CPU/RAM over time)

---

**Happy process managing!** 🚀
