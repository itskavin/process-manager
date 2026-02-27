export interface Process {
  id: string;
  name: string;
  command: string;
  args: string[];
  status: "Running" | "Stopped" | "Crashed";
  pid?: number;
  autoRestart: boolean;
  autoStart: boolean;
  workingDir?: string;
  uptimeMs: number;
  crashCount: number;
}

export interface LogEntry {
  timestamp: string;
  level: "stdout" | "stderr";
  message: string;
}

export interface ProcessMetrics {
  cpuPercent: number;
  memoryMb: number;
  memoryPercent: number;
}

export interface ProcessConfig {
  id: string;
  name: string;
  command: string;
  args: string[];
  autoRestart: boolean;
  autoStart: boolean;
  env?: Record<string, string>;
}
