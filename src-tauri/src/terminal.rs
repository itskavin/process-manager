use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::{Arc, Mutex};

/// One running terminal job (a single command execution).
pub struct TerminalJob {
    pub child: Arc<Mutex<Option<Child>>>,
}

/// Per-session state: tracks working directory and active jobs.
pub struct TerminalSession {
    pub cwd: String,
}

/// Top-level terminal state stored in AppState.
pub struct TerminalManager {
    pub sessions: HashMap<String, TerminalSession>,
    pub jobs: HashMap<String, TerminalJob>,
}

impl TerminalManager {
    pub fn new() -> Self {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "C:\\".to_string());

        let mut sessions = HashMap::new();
        sessions.insert(
            "default".to_string(),
            TerminalSession { cwd },
        );

        Self {
            sessions,
            jobs: HashMap::new(),
        }
    }

    /// Return the current working directory for a session, creating it lazily.
    pub fn get_cwd(&mut self, session_id: &str) -> String {
        if !self.sessions.contains_key(session_id) {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "C:\\".to_string());
            self.sessions
                .insert(session_id.to_string(), TerminalSession { cwd });
        }
        self.sessions[session_id].cwd.clone()
    }

    /// Set a new CWD for a session. Resolves relative paths.
    /// Returns the resolved absolute path, or an error.
    pub fn set_cwd(&mut self, session_id: &str, path: &str) -> Result<String, String> {
        let current = if let Some(s) = self.sessions.get(session_id) {
            PathBuf::from(&s.cwd)
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("C:\\"))
        };

        let target = if PathBuf::from(path).is_absolute() {
            PathBuf::from(path)
        } else {
            current.join(path)
        };

        // Canonicalize to resolve .. and symlinks
        let resolved = target
            .canonicalize()
            .map_err(|_| format!("Directory not found: {}", path))?;

        if !resolved.is_dir() {
            return Err(format!("Not a directory: {}", resolved.display()));
        }

        let cwd_str = resolved.to_string_lossy().to_string();

        if let Some(s) = self.sessions.get_mut(session_id) {
            s.cwd = cwd_str.clone();
        } else {
            self.sessions
                .insert(session_id.to_string(), TerminalSession { cwd: cwd_str.clone() });
        }

        Ok(cwd_str)
    }

    /// Kill a running job by job_id. No-op if already done.
    pub fn kill_job(&mut self, job_id: &str) -> Result<(), String> {
        if let Some(job) = self.jobs.get(job_id) {
            let mut guard = job.child.lock().map_err(|e| e.to_string())?;
            if let Some(ref mut child) = *guard {
                child.kill().ok();
            }
        }
        Ok(())
    }

    /// Register a new job.
    pub fn add_job(&mut self, job_id: String, child: Arc<Mutex<Option<Child>>>) {
        self.jobs.insert(job_id, TerminalJob { child });
    }

    /// Remove a completed job.
    pub fn remove_job(&mut self, job_id: &str) {
        self.jobs.remove(job_id);
    }
}
