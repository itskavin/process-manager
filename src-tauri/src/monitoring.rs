use crate::types::ProcessMetrics;
use sysinfo::{Pid, System};

pub struct Monitoring;

impl Monitoring {
    /// Call with a persistent `System` that was created at startup.
    /// Each call refreshes it so the CPU delta is computed from the
    /// previous refresh (accurate after the first measurement).
    pub fn get_process_metrics(pid: u32, system: &mut System) -> Result<ProcessMetrics, String> {
        system.refresh_all();

        let total_memory = system.total_memory(); // bytes
        let pid_sysinfo = Pid::from_u32(pid);

        if let Some(process) = system.process(pid_sysinfo) {
            let cpu_percent = process.cpu_usage();
            // sysinfo 0.30 returns bytes
            let memory_bytes = process.memory();
            let memory_mb = (memory_bytes as f64 / (1024.0 * 1024.0)).ceil() as u64;
            let memory_percent = if total_memory > 0 {
                (memory_bytes as f64 / total_memory as f64 * 100.0) as f32
            } else {
                0.0
            };

            Ok(ProcessMetrics {
                cpu_percent,
                memory_mb,
                memory_percent,
            })
        } else {
            Err("Process not found".to_string())
        }
    }

    #[allow(dead_code)]
    pub fn get_all_processes_metrics(system: &mut System) -> Result<Vec<(u32, ProcessMetrics)>, String> {
        system.refresh_all();
        let total_memory = system.total_memory();
        let mut metrics = Vec::new();

        for (pid, process) in system.processes() {
            let cpu_percent = process.cpu_usage();
            let memory_bytes = process.memory();
            let memory_mb = (memory_bytes as f64 / (1024.0 * 1024.0)).ceil() as u64;
            let memory_percent = if total_memory > 0 {
                (memory_bytes as f64 / total_memory as f64 * 100.0) as f32
            } else {
                0.0
            };

            metrics.push((pid.as_u32(), ProcessMetrics {
                cpu_percent,
                memory_mb,
                memory_percent,
            }));
        }

        Ok(metrics)
    }
}
