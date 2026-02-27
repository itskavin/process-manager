use crate::types::*;
use std::collections::HashMap;
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct ProcessInstance {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: ProcessStatus,
    pub child: Option<Child>,
    pub pid: Option<u32>,
    pub auto_restart: bool,
    pub auto_start: bool,
    pub working_dir: Option<String>,
    pub start_time: Option<u64>,
    pub crash_count: u32,
    pub should_restart: Arc<AtomicBool>,
}

impl ProcessInstance {
    pub fn new(name: String, command: String, args: Vec<String>, auto_restart: bool, auto_start: bool) -> Self {
        ProcessInstance {
            id: Uuid::new_v4().to_string(),
            name,
            command,
            args,
            status: ProcessStatus::Stopped,
            child: None,
            pid: None,
            auto_restart,
            auto_start,
            working_dir: None,
            start_time: None,
            crash_count: 0,
            should_restart: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn from_config(config: &ProcessConfig) -> Self {
        ProcessInstance {
            id: config.id.clone(),
            name: config.name.clone(),
            command: config.command.clone(),
            args: config.args.clone(),
            status: ProcessStatus::Stopped,
            child: None,
            pid: None,
            auto_restart: config.auto_restart,
            auto_start: config.auto_start,
            working_dir: config.working_dir.clone(),
            start_time: None,
            crash_count: 0,
            should_restart: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_uptime_ms(&self) -> u64 {
        match self.start_time {
            Some(start) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                now.saturating_sub(start)
            }
            None => 0,
        }
    }

    pub fn to_state(&self) -> ProcessState {
        ProcessState {
            id: self.id.clone(),
            name: self.name.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            status: self.status.to_string(),
            pid: self.pid,
            auto_restart: self.auto_restart,
            auto_start: self.auto_start,
            working_dir: self.working_dir.clone(),
            uptime_ms: self.get_uptime_ms(),
            crash_count: self.crash_count,
        }
    }

    pub fn to_config(&self) -> ProcessConfig {
        ProcessConfig {
            id: self.id.clone(),
            name: self.name.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            auto_restart: self.auto_restart,
            auto_start: self.auto_start,
            working_dir: self.working_dir.clone(),
            env: None,
        }
    }
}

pub struct ProcessManager {
    pub processes: HashMap<String, ProcessInstance>,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            processes: HashMap::new(),
        }
    }

    pub fn add_process(&mut self, name: String, command: String, args: Vec<String>, working_dir: Option<String>, auto_restart: bool) -> String {
        let mut process = ProcessInstance::new(name, command, args, auto_restart, false);
        process.working_dir = working_dir;
        let id = process.id.clone();
        self.processes.insert(id.clone(), process);
        id
    }

    pub fn get_process(&self, id: &str) -> Option<&ProcessInstance> {
        self.processes.get(id)
    }

    pub fn get_process_mut(&mut self, id: &str) -> Option<&mut ProcessInstance> {
        self.processes.get_mut(id)
    }

    pub fn remove_process(&mut self, id: &str) -> Option<ProcessInstance> {
        self.processes.remove(id)
    }

    pub fn get_all_processes(&self) -> Vec<ProcessState> {
        self.processes.values().map(|p| p.to_state()).collect()
    }

    pub fn spawn_process(&mut self, id: &str) -> Result<(u32, ChildStdout, ChildStderr), String> {
        let process = self
            .get_process_mut(id)
            .ok_or_else(|| "Process not found".to_string())?;

        // Don't spawn if already running
        if matches!(process.status, ProcessStatus::Running) {
            return Err("Process is already running".to_string());
        }

        let mut cmd = Command::new(&process.command);
        cmd.args(&process.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(ref dir) = process.working_dir {
            cmd.current_dir(dir);
        }

        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                let stdout = child.stdout.take()
                    .ok_or_else(|| "Failed to capture stdout".to_string())?;
                let stderr = child.stderr.take()
                    .ok_or_else(|| "Failed to capture stderr".to_string())?;
                process.pid = Some(pid);
                process.status = ProcessStatus::Running;
                process.child = Some(child);
                process.start_time = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                );
                process.crash_count = 0;
                process.should_restart.store(true, Ordering::SeqCst);
                Ok((pid, stdout, stderr))
            }
            Err(e) => {
                process.status = ProcessStatus::Crashed;
                process.crash_count += 1;
                Err(format!("Failed to spawn process: {}", e))
            }
        }
    }

    pub fn stop_process(&mut self, id: &str) -> Result<(), String> {
        let process = self
            .get_process_mut(id)
            .ok_or_else(|| "Process not found".to_string())?;

        process.should_restart.store(false, Ordering::SeqCst);

        if let Some(mut child) = process.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        process.status = ProcessStatus::Stopped;
        process.pid = None;
        Ok(())
    }

    pub fn restart_process(&mut self, id: &str) -> Result<(u32, ChildStdout, ChildStderr), String> {
        self.stop_process(id)?;
        self.spawn_process(id)
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
