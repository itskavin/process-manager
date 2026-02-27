use crate::config_handler::ConfigHandler;
use crate::log_handler::LogHandler;
use crate::monitoring::Monitoring;
use crate::process_manager::ProcessManager;
use crate::terminal::TerminalManager;
use crate::types::{ProcessConfig, ProcessState};
use chrono::Local;
use serde_json::json;
use std::io::BufRead;
use std::process::{ChildStderr, ChildStdout};
use std::sync::{Arc, Mutex};
use sysinfo::System;
use tauri::{Emitter, State, WebviewWindow};

pub struct AppState {
    pub manager: Arc<Mutex<ProcessManager>>,
    pub log_handler: Arc<LogHandler>,
    /// Persistent sysinfo System — keeps prior CPU snapshot so delta is accurate.
    pub system: Arc<Mutex<System>>,
    /// Integrated terminal sessions.
    pub terminal: Arc<Mutex<TerminalManager>>,
}

/// Spawn background threads that read stdout/stderr from a child process,
/// write each line to the log file, and emit a real-time Tauri event.
fn start_log_readers(
    process_id: String,
    stdout: ChildStdout,
    stderr: ChildStderr,
    log_handler: Arc<LogHandler>,
    window: WebviewWindow,
) {
    // --- stdout ---
    {
        let lh = Arc::clone(&log_handler);
        let id = process_id.clone();
        let win = window.clone();
        std::thread::spawn(move || {
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(msg) => {
                        let ts = Local::now().format("%H:%M:%S%.3f").to_string();
                        let _ = lh.write_log(&id, "stdout", &msg);
                        let _ = win.emit(
                            &format!("process:log:{}", id),
                            json!({ "timestamp": ts, "level": "stdout", "message": msg }),
                        );
                    }
                    Err(_) => break,
                }
            }
        });
    }
    // --- stderr ---
    {
        let lh = Arc::clone(&log_handler);
        let id = process_id.clone();
        let win = window.clone();
        std::thread::spawn(move || {
            let reader = std::io::BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(msg) => {
                        let ts = Local::now().format("%H:%M:%S%.3f").to_string();
                        let _ = lh.write_log(&id, "stderr", &msg);
                        let _ = win.emit(
                            &format!("process:log:{}", id),
                            json!({ "timestamp": ts, "level": "stderr", "message": msg }),
                        );
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

#[tauri::command]
pub async fn get_processes(state: State<'_, AppState>) -> Result<Vec<ProcessState>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_all_processes())
}

#[tauri::command]
pub async fn start_process(
    process_id: String,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<u32, String> {
    let (pid, stdout, stderr) = {
        let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
        let (pid, stdout, stderr) = manager.spawn_process(&process_id)?;
        let _ = window.emit(
            "process:status_changed",
            json!({ "id": &process_id, "status": "Running", "pid": pid }),
        );
        (pid, stdout, stderr)
    }; // lock released here

    start_log_readers(
        process_id,
        stdout,
        stderr,
        Arc::clone(&state.log_handler),
        window,
    );

    Ok(pid)
}

#[tauri::command]
pub async fn stop_process(
    process_id: String,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.stop_process(&process_id)?;

    let _ = window.emit(
        "process:status_changed",
        json!({
            "id": process_id,
            "status": "Stopped"
        }),
    );

    Ok(())
}

#[tauri::command]
pub async fn restart_process(
    process_id: String,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<u32, String> {
    let (pid, stdout, stderr) = {
        let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
        let (pid, stdout, stderr) = manager.restart_process(&process_id)?;
        let _ = window.emit(
            "process:status_changed",
            json!({ "id": &process_id, "status": "Running", "pid": pid }),
        );
        (pid, stdout, stderr)
    }; // lock released here

    start_log_readers(
        process_id,
        stdout,
        stderr,
        Arc::clone(&state.log_handler),
        window,
    );

    Ok(pid)
}

#[tauri::command]
pub async fn add_process(
    name: String,
    command: String,
    args: Vec<String>,
    working_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let id = manager.add_process(name, command, args, working_dir, false);
    Ok(id)
}

#[tauri::command]
pub async fn remove_process(
    process_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    manager.stop_process(&process_id).ok();
    manager.remove_process(&process_id);

    Ok(())
}

#[tauri::command]
pub async fn update_process(
    process_id: String,
    auto_restart: Option<bool>,
    auto_start: Option<bool>,
    working_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    if let Some(process) = manager.get_process_mut(&process_id) {
        if let Some(ar) = auto_restart {
            process.auto_restart = ar;
        }
        if let Some(as_val) = auto_start {
            process.auto_start = as_val;
        }
        // empty string means "clear working dir"
        match working_dir {
            Some(ref dir) if dir.is_empty() => process.working_dir = None,
            Some(dir) => process.working_dir = Some(dir),
            None => {}
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_metrics(process_id: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let pid = {
        let manager = state.manager.lock().map_err(|e| e.to_string())?;
        manager.get_process(&process_id).and_then(|p| p.pid)
    };

    if let Some(pid) = pid {
        let mut system = state.system.lock().map_err(|e| e.to_string())?;
        if let Ok(metrics) = Monitoring::get_process_metrics(pid, &mut system) {
            return Ok(json!({
                "cpu_percent": metrics.cpu_percent,
                "memory_mb": metrics.memory_mb,
                "memory_percent": metrics.memory_percent,
            }));
        }
    }

    Ok(json!({
        "cpu_percent": 0.0,
        "memory_mb": 0,
        "memory_percent": 0.0,
    }))
}

#[tauri::command]
pub async fn get_logs(
    process_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let logs = state
        .log_handler
        .read_logs(&process_id, Some(1000))
        .map_err(|e| e.to_string())?;

    let vec = logs
        .iter()
        .map(|log| {
            json!({
                "timestamp": log.timestamp,
                "level": log.level,
                "message": log.message
            })
        })
        .collect();

    Ok(vec)
}

#[tauri::command]
pub async fn clear_logs(
    process_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .log_handler
        .clear_logs(&process_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_config(state: State<'_, AppState>) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;

    let configs: Vec<ProcessConfig> = manager
        .processes
        .values()
        .map(|p| p.to_config())
        .collect();
    drop(manager);

    ConfigHandler::save_configs(&configs).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_config(state: State<'_, AppState>) -> Result<Vec<ProcessState>, String> {
    let configs = ConfigHandler::load_configs().map_err(|e| e.to_string())?;
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    for config in configs {
        let instance = crate::process_manager::ProcessInstance::from_config(&config);
        manager.processes.insert(instance.id.clone(), instance);
    }

    Ok(manager.get_all_processes())
}

#[tauri::command]
pub async fn set_auto_start(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::env;
        use winreg::enums::*;
        use winreg::RegKey;

        let exe_path = env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .map_err(|e| format!("Failed to open registry: {}", e))?;

        if enable {
            key.set_value("ProcessManager", &exe_path.to_string_lossy().to_string())
                .map_err(|e| format!("Failed to set registry: {}", e))?;
        } else {
            key.delete_value("ProcessManager").ok();
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Auto-start only supported on Windows".to_string())
    }
}

#[tauri::command]
pub async fn start_all(
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<(), String> {
    let spawned = {
        let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
        let process_ids: Vec<String> = manager.processes.keys().cloned().collect();
        let mut spawned = Vec::new();
        for process_id in process_ids {
            if let Ok((pid, stdout, stderr)) = manager.spawn_process(&process_id) {
                let _ = window.emit(
                    "process:status_changed",
                    json!({ "id": &process_id, "status": "Running", "pid": pid }),
                );
                spawned.push((process_id, stdout, stderr));
            }
        }
        spawned
    }; // lock released here

    for (process_id, stdout, stderr) in spawned {
        start_log_readers(
            process_id,
            stdout,
            stderr,
            Arc::clone(&state.log_handler),
            window.clone(),
        );
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_all(state: State<'_, AppState>, window: WebviewWindow) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;

    let process_ids: Vec<String> = manager.processes.keys().cloned().collect();
    for process_id in process_ids {
        manager.stop_process(&process_id).ok();
        let _ = window.emit(
            "process:status_changed",
            json!({
                "id": process_id,
                "status": "Stopped"
            }),
        );
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════
// Terminal commands
// ═══════════════════════════════════════════════════════════════

/// Run a shell command in the given terminal session.
/// Streams stdout/stderr via `terminal:output:{session_id}` events.
/// Emits `terminal:done:{session_id}` when the process exits.
#[tauri::command]
pub async fn terminal_run(
    session_id: String,
    command: String,
    job_id: String,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<(), String> {
    // Grab current CWD for this session
    let cwd = {
        let mut terminal = state.terminal.lock().map_err(|e| e.to_string())?;
        terminal.get_cwd(&session_id)
    };

    // Spawn via PowerShell on Windows
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;
    
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &command])
        .current_dir(&cwd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    
    // Hide console window on Windows
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to spawn command: {}", e))?;

    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    // Wrap child so both streaming threads + kill command can access it
    let child_arc: std::sync::Arc<Mutex<Option<std::process::Child>>> =
        std::sync::Arc::new(Mutex::new(Some(child)));

    // Register job
    {
        let mut terminal = state.terminal.lock().map_err(|e| e.to_string())?;
        terminal.add_job(job_id.clone(), std::sync::Arc::clone(&child_arc));
    }

    // ── stdout reader thread ──
    {
        let win = window.clone();
        let sid = session_id.clone();
        let jid = job_id.clone();
        std::thread::spawn(move || {
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(msg) = line {
                    let ts = Local::now().format("%H:%M:%S%.3f").to_string();
                    let _ = win.emit(
                        &format!("terminal:output:{}", sid),
                        json!({ "jobId": jid, "line": msg, "isError": false, "timestamp": ts }),
                    );
                }
            }
        });
    }

    // ── stderr reader + exit-waiter thread ──
    {
        let win = window.clone();
        let sid = session_id.clone();
        let jid = job_id.clone();
        let child_ref = std::sync::Arc::clone(&child_arc);
        let terminal_arc = std::sync::Arc::clone(&state.terminal);
        std::thread::spawn(move || {
            // Read stderr
            let reader = std::io::BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(msg) = line {
                    let ts = Local::now().format("%H:%M:%S%.3f").to_string();
                    let _ = win.emit(
                        &format!("terminal:output:{}", sid),
                        json!({ "jobId": jid, "line": msg, "isError": true, "timestamp": ts }),
                    );
                }
            }

            // Wait for child to exit
            let exit_code = {
                let mut guard = child_ref.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(ref mut c) = *guard {
                    c.wait().map(|s| s.code()).ok().flatten().unwrap_or(-1)
                } else {
                    -1
                }
            };

            // Remove completed job from manager
            if let Ok(mut mgr) = terminal_arc.lock() {
                mgr.remove_job(&jid);
            }

            let _ = win.emit(
                &format!("terminal:done:{}", sid),
                json!({ "jobId": jid, "exitCode": exit_code }),
            );
        });
    }

    Ok(())
}

/// Kill a running terminal job.
#[tauri::command]
pub async fn terminal_kill(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut terminal = state.terminal.lock().map_err(|e| e.to_string())?;
    terminal.kill_job(&job_id)
}

/// Update the working directory for a terminal session.
/// Returns the resolved absolute CWD or an error.
#[tauri::command]
pub async fn terminal_set_cwd(
    session_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut terminal = state.terminal.lock().map_err(|e| e.to_string())?;
    terminal.set_cwd(&session_id, &path)
}

/// Return the current working directory for a terminal session.
#[tauri::command]
pub async fn terminal_get_cwd(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut terminal = state.terminal.lock().map_err(|e| e.to_string())?;
    Ok(terminal.get_cwd(&session_id))
}

/// Promote a terminal command to a managed process.
/// The running job (if any) is left untouched; a new process entry is created.
#[tauri::command]
pub async fn terminal_add_process(
    name: String,
    command: String,
    working_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let parts: Vec<&str> = command.trim().splitn(2, ' ').collect();
    let exe = parts.first().copied().unwrap_or("").to_string();
    let args: Vec<String> = if parts.len() > 1 {
        parts[1].split_whitespace().map(|s| s.to_string()).collect()
    } else {
        vec![]
    };

    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let id = manager.add_process(name, exe, args, working_dir, false);
    Ok(id)
}
