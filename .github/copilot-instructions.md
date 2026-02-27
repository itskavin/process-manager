# Copilot Instructions for Process Manager

## Project Overview

**Process Manager** is a Tauri 2 desktop app (Windows-only) that manages, monitors, and controls long-running CLI commands via a native GUI. Think PM2 with a UI. It combines a **Vue 3 + TypeScript frontend** with a **Rust backend** for process control, monitoring, and terminal integration.

Core features:
- Start/stop/restart processes with state persistence
- Real-time ANSI-colored log streaming per process
- CPU/memory monitoring via `sysinfo`
- Integrated PowerShell terminal with command history
- System tray with quick actions
- Auto-restart/auto-start toggles

## Architecture

```
┌─────────────────────┐  Tauri IPC  ┌──────────────────┐
│   Frontend (Vue 3)  │◄──(invoke/emit)──┤  Backend (Rust)  │
│  - Pinia stores     │                  │ - ProcessManager │
│  - Components       │       Events    │ - Commands (IPC) │
│  - Composables      │      (real-time) │ - logHandler     │
└─────────────────────┘                  │ - Monitoring     │
                                         └──────────────────┘
```

**Data Flow:**
1. Frontend action (click "Start Process")
2. Pinia store invokes Tauri command (`invoke("start_process", ...)`)
3. Rust handler spawns process, streams logs via events
4. Frontend receives events and updates UI reactively

## Frontend Patterns

### State Management (Pinia)

All UI state lives in [src/stores/processStore.ts](src/stores/processStore.ts):
- `processes`: array of Process objects (id, name, command, status, pid, etc.)
- `selectedProcessId`: currently viewed process
- `logs`: Record<processId, LogEntry[]> (keyed by process ID)
- `metrics`: Record<processId, ProcessMetrics> (CPU, memory per process)

**Always use the store for state**, not local component state:
```typescript
import { useProcessStore } from '@/stores/processStore';
const store = useProcessStore();
store.startProcess(processId); // Pinia action
```

### Composables

[src/composables/useDialog.ts](src/composables/useDialog.ts) provides a global dialog pattern:
```typescript
const dialog = useDialog();
const confirmed = await dialog.openConfirm('Title', 'Are you sure?');
```
This shares state via reactive objects—do NOT create separate dialog instances.

### Component Structure

- **[ProcessList.vue](src/components/ProcessList.vue)**: Sidebar list of all processes
- **[ProcessPanel.vue](src/components/ProcessPanel.vue)**: Main view (logs + metrics for selected)
- **[TerminalPane.vue](src/components/TerminalPane.vue)**: Integrated PowerShell terminal
- **[AddProcessModal.vue](src/components/AddProcessModal.vue)**: Form to add new processes
- **[AppDialog.vue](src/components/AppDialog.vue)**: Global confirmation/alert dialogs

Components are primarily **presentational**; business logic lives in the Pinia store.

## Backend Patterns (Rust)

### Key Modules

- **[src-tauri/src/process_manager.rs](src-tauri/src/process_manager.rs)**: Core `ProcessManager` struct handling process lifecycle (spawn, monitor, stop, auto-restart)
- **[src-tauri/src/commands.rs](src-tauri/src/commands.rs)**: Tauri `#[command]` handlers that expose Rust methods to frontend via IPC
- **[src-tauri/src/monitoring.rs](src-tauri/src/monitoring.rs)**: CPU/memory tracking using `sysinfo`
- **[src-tauri/src/log_handler.rs](src-tauri/src/log_handler.rs)**: Writes stdout/stderr to files + emits events in real-time
- **[src-tauri/src/terminal.rs](src-tauri/src/terminal.rs)**: PowerShell session management with working directory tracking
- **[src-tauri/src/config_handler.rs](src-tauri/src/config_handler.rs)**: Config persistence (JSON files in user's appdata)

### Process Lifecycle

`ProcessManager` stores a HashMap of `ProcessInstance` objects:
1. **Create**: `ProcessInstance::new()` generates UUID, marks as Stopped
2. **Start**: Spawns child process, captures stdout/stderr, starts monitor thread
3. **Monitor**: Auto-restart checks if `should_restart` flag is set
4. **Stop**: Terminates child process gracefully
5. **Emit**: Each state change and log line triggers a Tauri event to frontend

### Adding a New Tauri Command

1. Create `#[tauri::command]` function in [src-tauri/src/commands.rs](src-tauri/src/commands.rs)
2. Access `State<AppState>` to get `manager`, `log_handler`, `system`, `terminal`
3. Return serializable data (JSON); errors become Tauri errors
4. Frontend calls `invoke("command_name", args)`

Example:
```rust
#[tauri::command]
fn start_process(process_id: String, state: State<AppState>) -> Result<u32, String> {
    let mut manager = state.manager.lock().unwrap();
    manager.start_process(&process_id)
        .map(|pid| pid as u32)
        .map_err(|e| e.to_string())
}
```

### Real-Time Events

Logs and metrics flow to frontend via **Tauri events**:
- `process:log:{processId}` – log entry (stdout/stderr)
- `process:metrics:{processId}` – CPU/memory update
- `process:status:{processId}` – process status changed

Frontend uses `listen()` to subscribe via composables or store.

## IPC Communication (Frontend ↔ Rust)

### Invoking Commands

```typescript
import { invoke } from "@tauri-apps/api/core";

const processId = await invoke("add_process", {
  name: "dev-server",
  command: "npm",
  args: ["run", "dev"],
  workingDir: "/path/to/project",
});
```

### Listening to Events

```typescript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen("process:log:process-123", (event) => {
  console.log(event.payload); // { timestamp, level, message }
});
```

## Build & Development

### Development

```bash
# Terminal 1: Rust backend in watch mode
cd src-tauri
cargo tauri dev

# Terminal 2: Vite frontend (dev server)
bun run dev
```

Vite runs on `localhost:1420`, Rust debugs on port 9222 in VS Code.

### Building Release

```bash
bun run build          # Compile frontend
cargo build --release # Compile Rust
bun run tauri build   # Bundle as MSI + NSIS installer
```

Output: `src-tauri/target/x86_64-pc-windows-gnu/release/bundle/`

### Windows Cross-Compile on Linux (CI)

**Recent fix**: Moved `tray-icon` feature to Windows-target-specific dependency in [src-tauri/Cargo.toml](src-tauri/Cargo.toml) to avoid appindicator library detection on Linux.

## Key Conventions

### Process ID Generation
Uses UUID v4 (see [src-tauri/src/process_manager.rs](src-tauri/src/process_manager.rs)):
```rust
let id = Uuid::new_v4().to_string();
```

### Type Definitions
Shared types in [src/types/process.ts](src/types/process.ts) (frontend) and [src-tauri/src/types.rs](src-tauri/src/types.rs) (backend) must stay in sync.

### Error Handling
- Rust: Use `Result<T, String>` for `#[command]` functions; errors serialize as Tauri errors
- Frontend: Wrap `invoke()` in try/catch; check store.error state

### Logging
- Process stdout/stderr → `~/.pm/logs/{processId}.log`
- Application logs → `~/.pm/logs/app.log` (via `log_handler`)

## Typical Workflows

### Adding a New Process Attribute

1. Update `Process` interface in [src/types/process.ts](src/types/process.ts)
2. Update `ProcessState` in [src-tauri/src/types.rs](src-tauri/src/types.rs)
3. Update `ProcessConfig` serialization in [src-tauri/src/config_handler.rs](src-tauri/src/config_handler.rs)
4. Update Pinia store in [src/stores/processStore.ts](src/stores/processStore.ts) if exposing as action
5. Update UI component (ProcessPanel, ProcessList) to display/edit

### Adding a New Feature (e.g., "Restart Process")

1. Add `#[tauri::command]` in [src-tauri/src/commands.rs](src-tauri/src/commands.rs)
2. Add Pinia action in [src/stores/processStore.ts](src/stores/processStore.ts) that calls `invoke()`
3. Add UI button in relevant Vue component
4. Listen to events if needed for real-time feedback

## Project Structure

```
src/
  components/          # Vue SFC files (ProcessList, ProcessPanel, etc.)
  composables/         # Reusable Vue composables (useDialog)
  stores/              # Pinia stores (processStore)
  types/              # TypeScript interfaces (Process, LogEntry, etc.)
  App.vue             # Root component (frameless titlebar + layout)
  main.ts             # Vue app entrypoint

src-tauri/
  src/
    commands.rs       # IPC handlers (#[tauri::command])
    config_handler.rs # Config persistence
    log_handler.rs    # Log file I/O
    monitoring.rs     # CPU/memory tracking
    process_manager.rs # Process lifecycle
    terminal.rs       # PowerShell terminal
    types.rs          # Rust types
    lib.rs            # Module tree + Tauri setup
    main.rs           # Binary entrypoint (minimal)
  Cargo.toml          # Rust dependencies
  tauri.conf.json     # Tauri config (window, tray, bundle)

.github/workflows/
  release.yml         # CI/CD for cross-compiling Windows on Ubuntu
```

## Debugging Tips

- **Frontend errors**: Check browser console (Tauri dev tools: F12)
- **Rust panics**: Enable `RUST_BACKTRACE=1` in Tauri dev
- **Event flow**: Use `listen("*")` in console to log all events
- **Process escape**: Processes spawn in headless mode; if blocked, check firewall
- **Config not persisting**: Check `~/.pm/config.json` permissions and format

## Dependencies

- **Tauri 2**: IPC + window management
- **Vue 3 + Pinia**: UI state
- **sysinfo**: CPU/memory metrics
- **winreg**: Windows registry (auto-start config)
- **tokio**: Async runtime
- **uuid**: Process ID generation
