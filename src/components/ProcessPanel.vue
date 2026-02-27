<template>
  <div class="panel">
    <!-- Empty state -->
    <div v-if="!proc" class="empty-state">
      <svg viewBox="0 0 48 48" width="48" height="48" fill="none">
        <rect x="4" y="8" width="40" height="28" rx="3" stroke="#1e293b" stroke-width="2"/>
        <path d="M16 44h16M24 36v8" stroke="#1e293b" stroke-width="2" stroke-linecap="round"/>
        <path d="M14 20l4 4-4 4M22 28h8" stroke="#334155" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      <p>Select a process</p>
      <span>Choose one from the sidebar to inspect it</span>
    </div>

    <!-- Process detail -->
    <div v-else class="panel-body">

      <!-- ── Header ── -->
      <div class="panel-header">
        <div class="hdr-left">
          <span :class="['status-dot', `dot-${proc.status.toLowerCase()}`]"></span>
          <h2 class="proc-title">{{ proc.name }}</h2>
          <span :class="['status-badge', `sb-${proc.status.toLowerCase()}`]">{{ proc.status }}</span>
        </div>
        <div class="hdr-right">
          <button v-if="proc.status !== 'Running'" class="hdr-btn start" @click="startProcess">
            <svg viewBox="0 0 10 10" width="9" height="9"><path d="M2 1.5l7 3.5-7 3.5V1.5z" fill="currentColor"/></svg>
            Start
          </button>
          <button v-if="proc.status === 'Running'" class="hdr-btn stop" @click="stopProcess">
            <svg viewBox="0 0 10 10" width="9" height="9"><rect x="2" y="2" width="6" height="6" rx="1" fill="currentColor"/></svg>
            Stop
          </button>
          <button class="hdr-btn restart" @click="restartProcess">
            <svg viewBox="0 0 12 12" width="10" height="10" fill="none">
              <path d="M10 6A4 4 0 112 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              <path d="M10 3v3h-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            Restart
          </button>
          <button :class="['hdr-btn', 'settings-btn', showSettings ? 'active' : '']" @click="showSettings = !showSettings">
            <svg viewBox="0 0 12 12" width="10" height="10" fill="none">
              <circle cx="6" cy="6" r="1.5" stroke="currentColor" stroke-width="1.3"/>
              <path d="M6 1v1.5M6 9.5V11M11 6H9.5M2.5 6H1M9.2 2.8l-1.1 1.1M3.9 8.1L2.8 9.2M9.2 9.2L8.1 8.1M3.9 3.9L2.8 2.8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            </svg>
          </button>
          <button class="hdr-btn close-btn" @click="closePanel" title="Close">
            <svg viewBox="0 0 12 12" width="10" height="10" fill="none">
              <path d="M2 2l8 8M10 2L2 10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- ── Metrics ── -->
      <div class="metrics-row">
        <div class="metric">
          <span class="m-label">CPU</span>
          <span class="m-val" :class="{ 'val-dim': proc.status !== 'Running' }">
            {{ proc.status === 'Running' ? metrics.cpuPercent.toFixed(1) + '%' : '—' }}
          </span>
        </div>
        <div class="metric">
          <span class="m-label">Memory</span>
          <span class="m-val" :class="{ 'val-dim': proc.status !== 'Running' }">
            <span v-if="proc.status === 'Running'">
              {{ metrics.memoryMb }} MB
              <span class="m-sub">&thinsp;{{ metrics.memoryPercent.toFixed(1) }}%</span>
            </span>
            <span v-else>—</span>
          </span>
        </div>
        <div class="metric">
          <span class="m-label">PID</span>
          <span class="m-val" :class="{ 'val-dim': !proc.pid }">{{ proc.pid ?? '—' }}</span>
        </div>
        <div class="metric">
          <span class="m-label">Uptime</span>
          <span class="m-val" :class="{ 'val-dim': !proc.uptimeMs }">{{ proc.uptimeMs ? formatUptime(proc.uptimeMs) : '—' }}</span>
        </div>
        <div class="metric">
          <span class="m-label">Crashes</span>
          <span class="m-val" :class="proc.crashCount ? 'val-crash' : 'val-dim'">{{ proc.crashCount }}</span>
        </div>
        <div class="metric">
          <span class="m-label">Command</span>
          <span class="m-val m-mono" style="font-size:0.72rem">{{ proc.command }}{{ proc.args?.length ? ' ' + proc.args.join(' ') : '' }}</span>
        </div>
      </div>

      <!-- ── Settings panel ── -->
      <transition name="settings-slide">
        <div v-if="showSettings" class="settings-panel">
          <div class="settings-grid">
            <div class="settings-col">
              <div class="settings-section-title">Behavior</div>
              <label class="toggle-row">
                <div class="toggle-info">
                  <span class="toggle-label">Auto-restart on crash</span>
                  <span class="toggle-hint">Restart automatically if the process exits unexpectedly</span>
                </div>
                <div :class="['toggle', { on: draft.autoRestart }]" @click="draft.autoRestart = !draft.autoRestart">
                  <div class="toggle-thumb"></div>
                </div>
              </label>
              <label class="toggle-row">
                <div class="toggle-info">
                  <span class="toggle-label">Auto-start on app launch</span>
                  <span class="toggle-hint">Start this process when Process Manager opens</span>
                </div>
                <div :class="['toggle', { on: draft.autoStart }]" @click="draft.autoStart = !draft.autoStart">
                  <div class="toggle-thumb"></div>
                </div>
              </label>
              <label class="toggle-row">
                <div class="toggle-info">
                  <span class="toggle-label">Launch on Windows startup</span>
                  <span class="toggle-hint">Start Process Manager automatically when Windows boots</span>
                </div>
                <div :class="['toggle', { on: draft.bootAutoStart }]" @click="draft.bootAutoStart = !draft.bootAutoStart">
                  <div class="toggle-thumb"></div>
                </div>
              </label>
            </div>
            <div class="settings-col">
              <div class="settings-section-title">Process</div>
              <div class="settings-field">
                <label class="sf-label">Working Directory</label>
                <input v-model="draft.workingDir" type="text" class="sf-input"
                  placeholder="Default: inherited from app" />
              </div>
            </div>
          </div>
          <div class="settings-footer">
            <button class="settings-save" @click="saveSettings" :disabled="saving">
              {{ saving ? 'Saving...' : 'Save Settings' }}
            </button>
            <span v-if="savedMsg" class="saved-msg">✓ Saved</span>
          </div>
        </div>
      </transition>

      <!-- ── Log viewer ── -->
      <div class="logs-section">
        <div class="logs-toolbar">
          <span class="logs-label">OUTPUT</span>
          <div class="logs-actions">
            <label class="autoscroll">
              <span class="as-track" :class="{ on: autoScroll }" @click="autoScroll = !autoScroll">
                <span class="as-thumb"></span>
              </span>
              <span class="as-text">Auto-scroll</span>
            </label>
            <button class="log-btn" @click="refreshLogs">
              <svg viewBox="0 0 12 12" width="10" height="10" fill="none">
                <path d="M10 6A4 4 0 112 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <path d="M10 3v3h-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </button>
            <button class="log-btn danger" @click="clearLogs">Clear</button>
          </div>
        </div>
        <div class="logs-body" ref="logsEl">
          <div
            v-for="(log, i) in currentLogs"
            :key="i"
            :class="['log-line', `ll-${log.level}`]"
          >
            <span class="ll-ts">{{ formatTs(log.timestamp) }}</span>
            <span :class="['ll-lvl', `lvl-${log.level}`]">{{ log.level }}</span>
            <!-- eslint-disable-next-line vue/no-v-html -->
            <span class="ll-msg" v-html="parseAnsi(log.message)"></span>
          </div>
          <div v-if="currentLogs.length === 0" class="no-output">No output yet…</div>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, nextTick, onBeforeUnmount } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useProcessStore } from '@/stores/processStore'
import { useDialog } from '@/composables/useDialog'

const store = useProcessStore()
const { openConfirm, openAlert } = useDialog()
const showSettings = ref(false)
const autoScroll = ref(true)
const saving = ref(false)
const savedMsg = ref(false)
const logsEl = ref<HTMLElement>()
const metrics = reactive({ cpuPercent: 0, memoryMb: 0, memoryPercent: 0 })
const draft = reactive({ autoRestart: false, autoStart: false, bootAutoStart: false, workingDir: '' })

const proc = computed(() => store.selectedProcess())
const currentLogs = computed(() =>
  store.selectedProcessId ? store.getProcessLogs(store.selectedProcessId) : []
)

let metricsInterval: ReturnType<typeof setInterval> | null = null
let unlistenLog: UnlistenFn | null = null

const scrollToBottom = async () => {
  if (!autoScroll.value) return
  await nextTick()
  if (logsEl.value) logsEl.value.scrollTop = logsEl.value.scrollHeight
}

const syncDraft = () => {
  const p = proc.value
  if (!p) return
  draft.autoRestart = p.autoRestart
  draft.autoStart = p.autoStart
  draft.bootAutoStart = false // can't read from registry, default off
  draft.workingDir = p.workingDir ?? ''
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
      metrics.memoryPercent = m.memoryPercent
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
  metrics.memoryPercent = 0
  showSettings.value = false
  stopLogListener()
  if (!id) { stopMetricsPolling(); return }
  syncDraft()
  await loadLogs()
  startMetricsPolling(id)
  unlistenLog = await listen(`process:log:${id}`, (event) => {
    const log = event.payload as { timestamp: string; level: string; message: string }
    store.addLog(id, {
      timestamp: log.timestamp,
      level: (log.level === 'stdout' ? 'stdout' : 'stderr') as 'stdout' | 'stderr',
      message: log.message,
    })
  })
  if (store.selectedProcessId !== id) { stopLogListener() }
})

watch(showSettings, (val) => { if (val) syncDraft() })

// Auto-scroll on new logs
watch(() => currentLogs.value.length, () => scrollToBottom())

const loadLogs = async () => {
  if (!store.selectedProcessId) return
  try { await store.loadLogs(store.selectedProcessId) }
  catch (e) { console.error('Failed to load logs:', e) }
  await scrollToBottom()
}

const startProcess = async () => {
  if (!store.selectedProcessId) return
  try { await store.startProcess(store.selectedProcessId) }
  catch (e) { await openAlert('Start Failed', String(e)) }
}

const stopProcess = async () => {
  if (!store.selectedProcessId) return
  try { await store.stopProcess(store.selectedProcessId) }
  catch (e) { await openAlert('Stop Failed', String(e)) }
}

const restartProcess = async () => {
  if (!store.selectedProcessId) return
  try { await store.restartProcess(store.selectedProcessId) }
  catch (e) { await openAlert('Restart Failed', String(e)) }
}

const closePanel = () => {
  store.selectedProcessId = null
}

const saveSettings = async () => {
  if (!store.selectedProcessId) return
  saving.value = true
  try {
    await store.updateProcess(
      store.selectedProcessId,
      draft.autoRestart,
      draft.autoStart,
      draft.workingDir
    )
    await store.setAutoStart(draft.bootAutoStart)
    await store.saveConfig()
    savedMsg.value = true
    setTimeout(() => { savedMsg.value = false }, 2000)
  } catch (e) {
    await openAlert('Save Failed', String(e))
  } finally {
    saving.value = false
  }
}

const refreshLogs = async () => { await loadLogs() }

const clearLogs = async () => {
  if (!store.selectedProcessId) return
  const ok = await openConfirm(
    'Clear Logs',
    'Delete all output logs for this process? This cannot be undone.',
    { type: 'danger', confirmLabel: 'Clear Logs', cancelLabel: 'Cancel' }
  )
  if (!ok) return
  try { await store.clearLogs(store.selectedProcessId) }
  catch (e) { await openAlert('Clear Failed', String(e)) }
}

const formatUptime = (ms: number) => {
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}s`
  if (s < 3600) return `${Math.floor(s / 60)}m ${s % 60}s`
  return `${Math.floor(s / 3600)}h ${Math.floor((s % 3600) / 60)}m`
}

const formatTs = (ts: string) => ts.length > 12 ? ts.slice(11, 19) : ts

// ── ANSI colour parser ──────────────────────────────────────────────────────
// Maps standard ANSI SGR colour codes to terminal-like hex colours.
const ANSI_FG: Record<number, string> = {
  30: '#4a5568', 31: '#f87171', 32: '#4ade80', 33: '#facc15',
  34: '#60a5fa', 35: '#c084fc', 36: '#22d3ee', 37: '#e2e8f0',
  90: '#6b7280', 91: '#fb923c', 92: '#86efac', 93: '#fde047',
  94: '#93c5fd', 95: '#d8b4fe', 96: '#67e8f9', 97: '#ffffff',
}
const ANSI_BG: Record<number, string> = {
  40: '#1a1a1a', 41: '#450a0a', 42: '#052e16', 43: '#422006',
  44: '#172554', 45: '#2e1065', 46: '#164e63', 47: '#1e293b',
}

function parseAnsi(raw: string): string {
  // Escape HTML first so injected content is safe
  const escaped = raw
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')

  let out = ''
  let bold = false
  let fg = ''
  let bg = ''
  let spanOpen = false

  const closeSpan = () => {
    if (spanOpen) { out += '</span>'; spanOpen = false }
  }
  const openSpan = () => {
    const styles: string[] = []
    if (fg) styles.push(`color:${fg}`)
    if (bg) styles.push(`background:${bg}`)
    if (bold) styles.push('font-weight:700')
    if (styles.length) {
      out += `<span style="${styles.join(';')}"`; out += '>'; spanOpen = true
    }
  }

  // Split on ANSI escape sequences
  const parts = escaped.split(/(\x1b\[[0-9;]*m)/)
  for (const part of parts) {
    const esc = part.match(/^\x1b\[([0-9;]*)m$/)
    if (esc) {
      closeSpan()
      const codes = esc[1] === '' ? [0] : esc[1].split(';').map(Number)
      for (const code of codes) {
        if (code === 0) { bold = false; fg = ''; bg = '' }
        else if (code === 1) bold = true
        else if (code === 22) bold = false
        else if (ANSI_FG[code]) fg = ANSI_FG[code]
        else if (ANSI_BG[code]) bg = ANSI_BG[code]
        else if (code === 39) fg = ''
        else if (code === 49) bg = ''
      }
      openSpan()
    } else if (part) {
      out += part
    }
  }
  closeSpan()
  return out || escaped
}
</script>
<style scoped>
/* ── Shell ── */
.panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #0f0f0f;
  overflow: hidden;
}

/* ── Empty state ── */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: #334155;
}
.empty-state p { font-size: 0.88rem; font-weight: 600; color: #475569; }
.empty-state span { font-size: 0.75rem; color: #2a3a4a; }

/* ── Panel body ── */
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
  padding: 11px 16px;
  border-bottom: 1px solid #1a1a1a;
  flex-shrink: 0;
  gap: 10px;
}
.hdr-left { display: flex; align-items: center; gap: 9px; min-width: 0; overflow: hidden; }

.status-dot {
  width: 9px; height: 9px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot-running { background: #22c55e; box-shadow: 0 0 7px #22c55eaa; }
.dot-stopped { background: #334155; }
.dot-crashed { background: #ef4444; box-shadow: 0 0 7px #ef4444aa; }

.proc-title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
  color: #f1f5f9;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.status-badge {
  font-size: 0.62rem;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  padding: 2px 8px;
  border-radius: 20px;
  flex-shrink: 0;
}
.sb-running { background: #052e16; color: #22c55e; border: 1px solid #14532d; }
.sb-stopped { background: #0f172a; color: #475569; border: 1px solid #1e293b; }
.sb-crashed { background: #2d0a0a; color: #ef4444; border: 1px solid #7f1d1d; }

.hdr-right { display: flex; align-items: center; gap: 5px; flex-shrink: 0; }

.hdr-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 11px;
  border: 1px solid transparent;
  border-radius: 5px;
  cursor: pointer;
  font-size: 0.76rem;
  font-weight: 500;
  transition: opacity 0.12s, background 0.12s;
  white-space: nowrap;
}
.hdr-btn:hover { opacity: 0.82; }
.hdr-btn.start   { background: #052e16; color: #22c55e; border-color: #14532d; }
.hdr-btn.stop    { background: #2d0a0a; color: #ef4444; border-color: #7f1d1d; }
.hdr-btn.restart { background: #1c1a08; color: #f59e0b; border-color: #78350f; }
.hdr-btn.settings-btn { background: #1a1a1a; color: #475569; border-color: #252525; width: 32px; padding: 5px; justify-content: center; }
.hdr-btn.settings-btn.active { background: #1e1b4b; color: #818cf8; border-color: #312e81; }
.hdr-btn.close-btn { background: #1a1a1a; color: #475569; border-color: #252525; width: 32px; padding: 5px; justify-content: center; }
.hdr-btn.close-btn:hover { color: #cbd5e1; }

/* ── Metrics ── */
.metrics-row {
  display: flex;
  flex-wrap: nowrap;
  border-bottom: 1px solid #1a1a1a;
  flex-shrink: 0;
  overflow-x: auto;
}
.metric {
  flex: 1;
  min-width: 80px;
  padding: 9px 14px;
  border-right: 1px solid #1a1a1a;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.metric:last-child { border-right: none; }
.m-label {
  font-size: 0.63rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: #334155;
}
.m-val {
  font-size: 0.84rem;
  font-weight: 600;
  color: #cbd5e1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.m-val.val-dim { color: #334155; font-weight: 400; }
.m-val.val-crash { color: #ef4444; }
.m-sub { font-size: 0.72rem; color: #475569; font-weight: 400; }
.m-mono { font-family: 'Cascadia Code', 'Courier New', monospace; font-size: 0.72rem !important; }

/* ── Settings panel ── */
.settings-panel {
  border-bottom: 1px solid #1a1a1a;
  background: #0d0d0d;
  flex-shrink: 0;
  overflow: hidden;
}
.settings-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0;
  padding: 14px 18px;
}
.settings-col { padding-right: 18px; }
.settings-col:last-child {
  padding-right: 0;
  border-left: 1px solid #1a1a1a;
  padding-left: 18px;
}
.settings-section-title {
  font-size: 0.64rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #334155;
  margin-bottom: 10px;
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 6px 0;
  border-bottom: 1px solid #141414;
}
.toggle-row:last-of-type { border-bottom: none; }
.toggle-info { flex: 1; min-width: 0; }
.toggle-label { display: block; font-size: 0.8rem; color: #94a3b8; }
.toggle-hint { display: block; font-size: 0.7rem; color: #334155; margin-top: 1px; }

.toggle {
  width: 32px; height: 17px;
  border-radius: 9px;
  background: #1e293b;
  border: 1px solid #334155;
  position: relative;
  cursor: pointer;
  flex-shrink: 0;
  transition: background 0.2s, border-color 0.2s;
}
.toggle.on { background: #4338ca; border-color: #6366f1; }
.toggle-thumb {
  position: absolute;
  top: 1.5px; left: 1.5px;
  width: 12px; height: 12px;
  border-radius: 50%;
  background: #475569;
  transition: transform 0.2s, background 0.2s;
}
.toggle.on .toggle-thumb { transform: translateX(15px); background: #fff; }

.settings-field { margin-top: 4px; }
.sf-label {
  display: block;
  font-size: 0.68rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #334155;
  margin-bottom: 5px;
}
.sf-input {
  width: 100%;
  padding: 6px 9px;
  background: #141414;
  border: 1px solid #222;
  border-radius: 5px;
  color: #94a3b8;
  font-size: 0.8rem;
  outline: none;
  transition: border-color 0.12s;
  font-family: 'Cascadia Code', 'Courier New', monospace;
}
.sf-input:focus { border-color: #6366f1; color: #cbd5e1; }
.sf-input::placeholder { color: #1e293b; }

.settings-footer {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 18px 12px;
}
.settings-save {
  padding: 6px 18px;
  background: #1e1b4b;
  border: 1px solid #312e81;
  border-radius: 5px;
  color: #a5b4fc;
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.12s;
}
.settings-save:hover:not(:disabled) { background: #312e81; }
.settings-save:disabled { opacity: 0.4; cursor: not-allowed; }
.saved-msg { font-size: 0.75rem; color: #22c55e; }

.settings-slide-enter-active,
.settings-slide-leave-active { transition: all 0.16s ease; overflow: hidden; }
.settings-slide-enter-from,
.settings-slide-leave-to { opacity: 0; max-height: 0; }
.settings-slide-enter-to,
.settings-slide-leave-from { opacity: 1; max-height: 400px; }

/* ── Logs ── */
.logs-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}
.logs-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 14px;
  border-bottom: 1px solid #1a1a1a;
  flex-shrink: 0;
}
.logs-label {
  font-size: 0.63rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: #334155;
}
.logs-actions { display: flex; align-items: center; gap: 8px; }

.autoscroll { display: flex; align-items: center; gap: 6px; cursor: pointer; }
.as-track {
  width: 26px; height: 14px;
  border-radius: 7px;
  background: #1a1a1a;
  border: 1px solid #252525;
  position: relative;
  cursor: pointer;
  transition: background 0.2s;
}
.as-track.on { background: #1e1b4b; border-color: #4338ca; }
.as-thumb {
  position: absolute;
  top: 1px; left: 1px;
  width: 10px; height: 10px;
  border-radius: 50%;
  background: #334155;
  transition: transform 0.2s, background 0.2s;
}
.as-track.on .as-thumb { transform: translateX(12px); background: #818cf8; }
.as-text { font-size: 0.72rem; color: #475569; }

.log-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 9px;
  background: #141414;
  border: 1px solid #1e1e1e;
  border-radius: 4px;
  color: #475569;
  font-size: 0.72rem;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.log-btn:hover { background: #1e1e1e; color: #94a3b8; }
.log-btn.danger { border-color: #3d1010; color: #f87171; background: #1a0808; }
.log-btn.danger:hover { background: #2d0a0a; color: #fca5a5; border-color: #7f1d1d; }

.logs-body {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
  font-family: 'Cascadia Code', 'Fira Code', 'Courier New', monospace;
  font-size: 0.76rem;
  line-height: 1.6;
  min-height: 0;
}
.log-line {
  display: flex;
  padding: 0 4px;
}
.log-line:hover { background: #141414; }
.ll-ts {
  color: #374151;
  font-size: 0.69rem;
  min-width: 66px;
  flex-shrink: 0;
  padding: 0 6px 0 10px;
  line-height: inherit;
}
.ll-lvl {
  font-size: 0.65rem;
  font-weight: 700;
  min-width: 46px;
  flex-shrink: 0;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  line-height: inherit;
  padding-right: 6px;
}
.lvl-stdout { color: #475569; }
.lvl-stderr { color: #f87171; }
.ll-stdout .ll-msg { color: #94a3b8; }
.ll-stderr .ll-msg { color: #fca5a5; }
.ll-msg { flex: 1; word-break: break-all; white-space: pre-wrap; }

.no-output {
  padding: 2.5rem;
  text-align: center;
  color: #1e293b;
  font-size: 0.82rem;
  font-family: sans-serif;
}
</style>
