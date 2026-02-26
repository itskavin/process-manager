import { defineStore } from "pinia";
import { reactive, ref } from "vue";
import type { Process, LogEntry, ProcessMetrics } from "@/types/process";
import { invoke } from "@tauri-apps/api/core";

export const useProcessStore = defineStore("process", () => {
  const processes = ref<Process[]>([]);
  const selectedProcessId = ref<string | null>(null);
  const logs = reactive<Record<string, LogEntry[]>>({});
  const metrics = reactive<Record<string, ProcessMetrics>>({});

  const selectedProcess = () => {
    if (!selectedProcessId.value) return null;
    return processes.value.find((p) => p.id === selectedProcessId.value) || null;
  };

  const getProcessLogs = (id: string): LogEntry[] => {
    return logs[id] || [];
  };

  const getProcessMetrics = (id: string): ProcessMetrics => {
    return (
      metrics[id] || {
        cpuPercent: 0,
        memoryMb: 0,
        memoryPercent: 0,
      }
    );
  };

  const addProcess = async (
    name: string,
    command: string,
    args: string[]
  ) => {
    try {
      const id: string = await invoke("add_process", {
        name,
        command,
        args,
      });
      const newProcess: Process = {
        id,
        name,
        command,
        args,
        status: "Stopped",
        autoRestart: false,
        autoStart: false,
        uptimeMs: 0,
        crashCount: 0,
      };
      processes.value.push(newProcess);
      logs[id] = [];
      return id;
    } catch (error) {
      console.error("Failed to add process:", error);
      throw error;
    }
  };

  const removeProcess = async (id: string) => {
    try {
      await invoke("remove_process", { processId: id });
      processes.value = processes.value.filter((p) => p.id !== id);
      delete logs[id];
      delete metrics[id];
      if (selectedProcessId.value === id) {
        selectedProcessId.value = null;
      }
    } catch (error) {
      console.error("Failed to remove process:", error);
      throw error;
    }
  };

  const startProcess = async (id: string) => {
    try {
      const pid: number = await invoke("start_process", { processId: id });
      const process = processes.value.find((p) => p.id === id);
      if (process) {
        process.status = "Running";
        process.pid = pid;
      }
    } catch (error) {
      console.error("Failed to start process:", error);
      throw error;
    }
  };

  const stopProcess = async (id: string) => {
    try {
      await invoke("stop_process", { processId: id });
      const process = processes.value.find((p) => p.id === id);
      if (process) {
        process.status = "Stopped";
        process.pid = undefined;
      }
    } catch (error) {
      console.error("Failed to stop process:", error);
      throw error;
    }
  };

  const restartProcess = async (id: string) => {
    try {
      const pid: number = await invoke("restart_process", { processId: id });
      const process = processes.value.find((p) => p.id === id);
      if (process) {
        process.status = "Running";
        process.pid = pid;
      }
    } catch (error) {
      console.error("Failed to restart process:", error);
      throw error;
    }
  };

  const updateProcess = async (
    id: string,
    autoRestart?: boolean,
    autoStart?: boolean
  ) => {
    try {
      await invoke("update_process", { processId: id, autoRestart, autoStart });
      const process = processes.value.find((p) => p.id === id);
      if (process) {
        if (autoRestart !== undefined) process.autoRestart = autoRestart;
        if (autoStart !== undefined) process.autoStart = autoStart;
      }
    } catch (error) {
      console.error("Failed to update process:", error);
      throw error;
    }
  };

  const loadProcesses = async () => {
    try {
      const result: Process[] = await invoke("get_processes");
      processes.value = result;
      result.forEach((p) => {
        if (!logs[p.id]) logs[p.id] = [];
        if (!metrics[p.id]) metrics[p.id] = { cpuPercent: 0, memoryMb: 0 };
      });
    } catch (error) {
      console.error("Failed to load processes:", error);
      throw error;
    }
  };

  const loadLogs = async (id: string) => {
    try {
      const result: Array<{
        timestamp: string;
        level: string;
        message: string;
      }> = await invoke("get_logs", { processId: id });
      logs[id] = result.map((log) => ({
        timestamp: log.timestamp,
        level: (log.level === "stdout" ? "stdout" : "stderr") as
          | "stdout"
          | "stderr",
        message: log.message,
      }));
    } catch (error) {
      console.error("Failed to load logs:", error);
    }
  };

  const addLog = (id: string, log: LogEntry) => {
    if (!logs[id]) logs[id] = [];
    logs[id].push(log);
    // Keep only last 1000 logs
    if (logs[id].length > 1000) {
      logs[id] = logs[id].slice(-1000);
    }
  };

  const clearLogs = async (id: string) => {
    try {
      await invoke("clear_logs", { processId: id });
      logs[id] = [];
    } catch (error) {
      console.error("Failed to clear logs:", error);
      throw error;
    }
  };

  const updateMetrics = async (id: string) => {
    try {
      const result: { cpu_percent: number; memory_mb: number; memory_percent: number } = await invoke(
        "get_metrics",
        { processId: id }
      );
      metrics[id] = {
        cpuPercent: result.cpu_percent,
        memoryMb: result.memory_mb,
        memoryPercent: result.memory_percent,
      };
    } catch (error) {
      console.error("Failed to get metrics:", error);
    }
  };

  const saveConfig = async () => {
    try {
      await invoke("save_config");
    } catch (error) {
      console.error("Failed to save config:", error);
      throw error;
    }
  };

  const loadConfig = async () => {
    try {
      const result: Process[] = await invoke("load_config");
      processes.value = result;
      result.forEach((p) => {
        if (!logs[p.id]) logs[p.id] = [];
        if (!metrics[p.id]) metrics[p.id] = { cpuPercent: 0, memoryMb: 0 };
      });
    } catch (error) {
      console.error("Failed to load config:", error);
      throw error;
    }
  };

  const setAutoStart = async (enable: boolean) => {
    try {
      await invoke("set_auto_start", { enable });
    } catch (error) {
      console.error("Failed to set auto-start:", error);
      throw error;
    }
  };

  const startAll = async () => {
    try {
      await invoke("start_all");
    } catch (error) {
      console.error("Failed to start all:", error);
      throw error;
    }
  };

  const stopAll = async () => {
    try {
      await invoke("stop_all");
    } catch (error) {
      console.error("Failed to stop all:", error);
      throw error;
    }
  };

  return {
    processes,
    selectedProcessId,
    selectedProcess,
    addProcess,
    removeProcess,
    startProcess,
    stopProcess,
    restartProcess,
    updateProcess,
    loadProcesses,
    loadLogs,
    getProcessLogs,
    addLog,
    clearLogs,
    getProcessMetrics,
    updateMetrics,
    saveConfig,
    loadConfig,
    setAutoStart,
    startAll,
    stopAll,
  };
});
