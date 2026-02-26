<template>
  <div class="panel">
    <!-- No process selected -->
    <div v-if="!store.selectedProcess()" class="no-selection">
      <div class="no-selection-icon">⚡</div>
      <div class="no-selection-text">Select a process to view details</div>
    </div>

    <!-- Process selected -->
    <div v-else class="panel-body">
      <!-- Panel Header -->
      <div class="panel-header">
        <div class="header-left">
          <span :class="['status-dot', `dot-${store.selectedProcess()!.status.toLowerCase()}`]"></span>
          <h2 class="proc-title">{{ store.selectedProcess()!.name }}</h2>
          <span :class="['status-label', `sl-${store.selectedProcess()!.status.toLowerCase()}`]">
            {{ store.selectedProcess()!.status }}
          </span>
        </div>
        <div class="header-actions">
          <button
            v-if="store.selectedProcess()!.status !== 'Running'"
            class="hdr-btn start"
            @click="startProcess"
          >▶ Start</button>
          <button
            v-if="store.selectedProcess()!.status === 'Running'"
            class="hdr-btn stop"
            @click="stopProcess"
          >⏹ Stop</button>
          <button class="hdr-btn restart" @click="restartProcess">↻ Restart</button>
          <button class="hdr-btn settings" @click="showSettings = !showSettings">
            {{ showSettings ? '✕ Close' : '⚙ Settings' }}
          </button>
        </div>
      </div>

      <!-- Settings Panel -->
      <div v-if="showSettings" class="settings-bar">
        <label class="setting-item">
          <input type="checkbox" :checked="store.selectedProcess()!.autoRestart" @change="toggleAutoRestart" />
          <span>Auto-restart on crash</span>
        </label>
        <label class="setting-item">
          <input type="checkbox" :checked="store.selectedProcess()!.autoStart" @change="toggleAutoStart" />
          <span>Auto-start on launch</span>
        </label>
        <button class="save-btn" @click="saveConfig">✓ Save Config</button>
      </div>

      <!-- Metrics Row -->
      <div class="metrics-row">
        <div class="metric">
          <span class="metric-label">Command</span>
          <span class="metric-value mono">{{ store.selectedProcess()!.command }}{{ store.selectedProcess()!.args?.length ? ' ' + store.selectedProcess()!.args.join(' ') : '' }}</span>
        </div>
        <div class="metric" v-if="store.selectedProcess()!.pid">
          <span class="metric-label">PID</span>
          <span class="metric-value">{{ store.selectedProcess()!.pid }}</span>
        </div>
        <div class="metric" v-if="store.selectedProcess()!.status === 'Running'">
          <span class="metric-label">CPU</span>
          <span class="metric-value">{{ metrics.cpuPercent.toFixed(1) }}%</span>
        </div>
        <div class="metric" v-if="store.selectedProcess()!.status === 'Running'">
          <span class="metric-label">Memory</span>
          <span class="metric-value">{{ metrics.memoryMb }} MB</span>
        </div>
        <div class="metric" v-if="store.selectedProcess()!.uptimeMs">
          <span class="metric-label">Uptime</span>
          <span class="metric-value">{{ formatUptime(store.selectedProcess()!.uptimeMs) }}</span>
        </div>
        <div class="metric" v-if="store.selectedProcess()!.crashCount">
          <span class="metric-label">Crashes</span>
          <span class="metric-value crash">{{ store.selectedProcess()!.crashCount }}</span>
        </div>
      </div>

      <!-- Logs -->
      <div class="logs-section">
        <div class="logs-header">
          <span class="logs-title">OUTPUT</span>
          <div class="logs-actions">
            <label class="autoscroll-toggle">
              <input type="checkbox" v-model="autoScroll" />
              <span>Auto-scroll</span>
            </label>
            <button class="log-btn" @click="refreshLogs">↻ Refresh</button>
            <button class="log-btn danger" @click="clearLogs">Clear</button>
          </div>
        </div>
        <div class="logs-body" ref="logsEl">
          <div
            v-for="(log, i) in store.getProcessLogs(store.selectedProcessId!)"
            :key="i"
            :class="['log-line', `log-${log.level}`]"
          >
            <span class="ts">{{ formatTs(log.timestamp) }}</span>
            <span class="lvl">{{ log.level }}</span>
            <span class="msg">{{ log.message }}</span>
          </div>
          <div v-if="store.getProcessLogs(store.selectedProcessId!).length === 0" class="no-logs">
            No output yet…
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch, nextTick, onBeforeUnmount } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useProcessStore } from '@/stores/processStore'

const store = useProcessStore()
const showSettings = ref(false)
const autoScroll = ref(true)
const logsEl = ref<HTMLElement>()
const metrics = reactive({ cpuPercent: 0, memoryMb: 0 })

let metricsInterval: ReturnType<typeof setInterval> | null = null
let unlistenLog: UnlistenFn | null = null

const scrollToBottom = async () => {
  if (!autoScroll.value) return
  await nextTick()
  if (logsEl.value) logsEl.value.scrollTop = logsEl.value.scrollHeight
}

const startMetricsPolling = (id: string) => {
  stopMetricsPolling()
  metricsInterval = setInterval(async () => {
    if (store.selectedProcessId !== id) { stopMetricsPolling(); return }
    try {
      await store.updateMetrics(id)
      const m = store.getProcessMetrics(id)
      metrics.cpuPercent = m.cpuPercent
      metrics.memoryMb = m.memoryMb
    } catch { /* ignore */ }
  }, 1500)
}

const stopMetricsPolling = () => {
  if (metricsInterval) { clearInterval(metricsInterval); metricsInterval = null }
}

const stopLogListener = () => {
  if (unlistenLog) { unlistenLog(); unlistenLog = null }
}

onBeforeUnmount(() => {
  stopMetricsPolling()
  stopLogListener()
})

watch(() => store.selectedProcessId, async (id) => {
  metrics.cpuPercent = 0
  metrics.memoryMb = 0
  showSettings.value = false
  stopLogListener()
  if (!id) { stopMetricsPolling(); return }
  await loadLogs()
  startMetricsPolling(id)
  // Subscribe to real-time log events from this process
  unlistenLog = await listen(`process:log:${id}`, (event) => {
    const log = event.payload as { timestamp: string; level: string; message: string }
    store.addLog(id, {
      timestamp: log.timestamp,
      level: (log.level === 'stdout' ? 'stdout' : 'stderr') as 'stdout' | 'stderr',
      message: log.message,
    })
  })
  // Guard: if selection changed while awaiting listen(), discard immediately
  if (store.selectedProcessId !== id) { stopLogListener() }
})

// Auto-scroll on new logs
watch(() => store.selectedProcessId && store.getProcessLogs(store.selectedProcessId!).length,
  () => scrollToBottom()
)

const loadLogs = async () => {
  if (!store.selectedProcessId) return
  try { await store.loadLogs(store.selectedProcessId) }
  catch (e) { console.error('Failed to load logs:', e) }
  await scrollToBottom()
}

const startProcess = async () => {
  if (!store.selectedProcessId) return
  try { await store.startProcess(store.selectedProcessId) }
  catch (e) { alert(`Start failed: ${e}`) }
}

const stopProcess = async () => {
  if (!store.selectedProcessId) return
  try { await store.stopProcess(store.selectedProcessId) }
  catch (e) { alert(`Stop failed: ${e}`) }
}

const restartProcess = async () => {
  if (!store.selectedProcessId) return
  try { await store.restartProcess(store.selectedProcessId) }
  catch (e) { alert(`Restart failed: ${e}`) }
}

const toggleAutoRestart = async () => {
  if (!store.selectedProcessId) return
  const p = store.selectedProcess()
  if (!p) return
  try { await store.updateProcess(store.selectedProcessId, !p.autoRestart) }
  catch (e) { alert(`Failed: ${e}`) }
}

const toggleAutoStart = async () => {
  if (!store.selectedProcessId) return
  const p = store.selectedProcess()
  if (!p) return
  try { await store.updateProcess(store.selectedProcessId, undefined, !p.autoStart) }
  catch (e) { alert(`Failed: ${e}`) }
}

const saveConfig = async () => {
  try { await store.saveConfig(); showSettings.value = false }
  catch (e) { alert(`Failed to save: ${e}`) }
}

const refreshLogs = async () => { await loadLogs() }

const clearLogs = async () => {
  if (!store.selectedProcessId || !confirm('Clear all logs?')) return
  try { await store.clearLogs(store.selectedProcessId) }
  catch (e) { alert(`Failed to clear: ${e}`) }
}

const formatUptime = (ms: number) => {
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}s`
  if (s < 3600) return `${Math.floor(s / 60)}m ${s % 60}s`
  return `${Math.floor(s / 3600)}h ${Math.floor((s % 3600) / 60)}m`
}

const formatTs = (ts: string) => ts.length > 12 ? ts.slice(11, 19) : ts
</script>

<style scoped>
/* ── Outer shell ── */
.panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #171717;
  overflow: hidden;
}

/* ── No-selection placeholder ── */
.no-selection {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: #374151;
}
.no-selection-icon { font-size: 2.5rem; }
.no-selection-text { font-size: 0.9rem; }

/* ── Panel body (flex column, fills height) ── */
.panel-body {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* ── Header ── */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #2e2e2e;
  flex-shrink: 0;
  gap: 12px;
  flex-wrap: wrap;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.status-dot {
  width: 10px; height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot-running { background: #4ade80; box-shadow: 0 0 6px #4ade8077; }
.dot-stopped { background: #52525b; }
.dot-crashed { background: #f87171; box-shadow: 0 0 6px #f8717177; }
.proc-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: #f1f5f9;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.status-label {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 2px 7px;
  border-radius: 20px;
  flex-shrink: 0;
}
.sl-running { background: #14532d; color: #4ade80; }
.sl-stopped { background: #27272a; color: #a1a1aa; }
.sl-crashed { background: #7f1d1d; color: #f87171; }

.header-actions { display: flex; gap: 6px; flex-wrap: wrap; }
.hdr-btn {
  padding: 5px 12px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  font-size: 0.78rem;
  font-weight: 500;
  transition: opacity .15s;
  white-space: nowrap;
}
.hdr-btn:hover { opacity: 0.8; }
.hdr-btn.start    { background: #14532d; color: #4ade80; }
.hdr-btn.stop     { background: #7f1d1d; color: #f87171; }
.hdr-btn.restart  { background: #713f12; color: #fbbf24; }
.hdr-btn.settings { background: #1e293b; color: #94a3b8; border: 1px solid #334155; }

/* ── Settings ── */
.settings-bar {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 10px 16px;
  background: #1e293b;
  border-bottom: 1px solid #2e2e2e;
  flex-wrap: wrap;
  flex-shrink: 0;
}
.setting-item {
  display: flex;
  align-items: center;
  gap: 7px;
  color: #cbd5e1;
  font-size: 0.83rem;
  cursor: pointer;
}
.setting-item input[type="checkbox"] { cursor: pointer; }
.save-btn {
  padding: 5px 14px;
  background: #1e40af;
  border: none;
  border-radius: 5px;
  color: #93c5fd;
  font-size: 0.8rem;
  cursor: pointer;
  transition: background .15s;
  margin-left: auto;
}
.save-btn:hover { background: #1d4ed8; }

/* ── Metrics row ── */
.metrics-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0;
  border-bottom: 1px solid #2e2e2e;
  flex-shrink: 0;
}
.metric {
  flex: 1;
  min-width: 100px;
  padding: 10px 16px;
  border-right: 1px solid #2e2e2e;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.metric:last-child { border-right: none; }
.metric-label {
  font-size: 0.68rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #4b5563;
}
.metric-value {
  font-size: 0.85rem;
  color: #e2e8f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.metric-value.mono { font-family: 'Courier New', monospace; font-size: 0.8rem; }
.metric-value.crash { color: #f87171; }

/* ── Logs section ── */
.logs-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.logs-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid #2e2e2e;
  flex-shrink: 0;
}
.logs-title {
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #4b5563;
}
.logs-actions { display: flex; align-items: center; gap: 8px; }
.autoscroll-toggle {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 0.75rem;
  color: #6b7280;
  cursor: pointer;
}
.log-btn {
  padding: 3px 10px;
  background: #1e293b;
  border: 1px solid #334155;
  border-radius: 4px;
  color: #94a3b8;
  font-size: 0.74rem;
  cursor: pointer;
  transition: background .15s;
}
.log-btn:hover { background: #263144; }
.log-btn.danger { border-color: #7f1d1d; color: #fca5a5; }
.log-btn.danger:hover { background: #450a0a; }

.logs-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
  font-family: 'Cascadia Code', 'Courier New', monospace;
  font-size: 0.78rem;
  line-height: 1.5;
}
.log-line {
  display: flex;
  gap: 8px;
  padding: 1px 14px;
  transition: background .1s;
}
.log-line:hover { background: #1e2431; }
.log-stdout { color: #e2e8f0; }
.log-stderr .msg { color: #fca5a5; }
.ts { color: #374151; font-size: 0.72rem; min-width: 62px; flex-shrink: 0; }
.lvl {
  font-size: 0.68rem;
  font-weight: 700;
  min-width: 42px;
  flex-shrink: 0;
  text-transform: uppercase;
}
.log-stdout .lvl { color: #3b82f6; }
.log-stderr .lvl { color: #ef4444; }
.msg { flex: 1; word-break: break-all; white-space: pre-wrap; }

.no-logs {
  padding: 2rem;
  text-align: center;
  color: #374151;
  font-size: 0.85rem;
  font-family: sans-serif;
}
</style>
