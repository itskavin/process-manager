<template>
  <div class="sidebar">

    <!-- ── Header ── -->
    <div class="sidebar-header">
      <div class="header-left">
        <span class="sidebar-title">Processes</span>
        <span v-if="store.processes.length" class="count-pill">
          <span class="count-running">{{ runningCount }}</span>
          <span class="count-sep">/</span>
          <span>{{ store.processes.length }}</span>
        </span>
      </div>
      <div class="header-actions">
        <button class="icon-btn success" @click="startAllProcesses" title="Start All">
          <svg viewBox="0 0 10 10" width="9" height="9"><path d="M2 1.5l7 3.5-7 3.5V1.5z" fill="currentColor"/></svg>
        </button>
        <button class="icon-btn danger" @click="stopAllProcesses" title="Stop All">
          <svg viewBox="0 0 10 10" width="9" height="9"><rect x="2" y="2" width="6" height="6" rx="1" fill="currentColor"/></svg>
        </button>
      </div>
    </div>

    <!-- ── Search ── -->
    <div class="search-bar">
      <svg viewBox="0 0 14 14" width="13" height="13" fill="none" class="search-icon">
        <circle cx="6" cy="6" r="4.5" stroke="#374151" stroke-width="1.3"/>
        <path d="M9.5 9.5l2.5 2.5" stroke="#374151" stroke-width="1.3" stroke-linecap="round"/>
      </svg>
      <input
        v-model="search"
        type="text"
        placeholder="Filter processes…"
        class="search-input"
        @keyup.escape="search = ''"
      />
      <button v-if="search" class="search-clear" @click="search = ''" title="Clear">
        <svg viewBox="0 0 10 10" width="8" height="8"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>
      </button>
    </div>

    <!-- ── Process list ── -->
    <div class="list">

      <div v-if="store.processes.length === 0" class="empty-state">
        <div class="empty-icon">
          <svg viewBox="0 0 40 40" width="38" height="38" fill="none">
            <rect x="4" y="8" width="32" height="22" rx="3" stroke="#1e293b" stroke-width="1.5"/>
            <path d="M14 36h12M20 30v6" stroke="#1e293b" stroke-width="1.5" stroke-linecap="round"/>
            <path d="M12 18l3 3 5-5" stroke="#312e81" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <p class="empty-title">No processes yet</p>
        <p class="empty-hint">Click <strong>New Process</strong> below to get started.</p>
      </div>

      <div v-else-if="filteredProcesses.length === 0" class="empty-state">
        <p class="empty-title">No matches</p>
        <p class="empty-hint">Try a different search term.</p>
      </div>

      <div
        v-for="p in filteredProcesses"
        :key="p.id"
        :class="['process-row', `row-${p.status.toLowerCase()}`, { selected: store.selectedProcessId === p.id }]"
        @click="store.selectedProcessId = p.id"
      >
        <!-- Main row content -->
        <div class="row-content">
          <div class="row-indicator" :class="`ind-${p.status.toLowerCase()}`"></div>
          <div class="row-body">
            <div class="row-top">
              <span class="p-name">{{ p.name }}</span>
              <span :class="['status-pill', `pill-${p.status.toLowerCase()}`]">{{ p.status }}</span>
            </div>
            <div class="row-bottom">
              <span class="p-cmd">{{ p.command }}{{ p.args?.length ? ' ' + p.args.join(' ') : '' }}</span>
              <span v-if="p.status === 'Crashed' && p.crashCount" class="crash-badge">{{ p.crashCount }}x</span>
            </div>
            <div v-if="p.pid && p.status === 'Running'" class="row-meta">
              PID {{ p.pid }}&nbsp; · &nbsp;{{ formatUptime(p.uptimeMs) }}
            </div>
          </div>
        </div>

        <!-- Action bar (hover/selected) -->
        <div class="row-actions" @click.stop>
          <button v-if="p.status !== 'Running'" class="ra-btn start" @click="startProcess(p.id)" title="Start">
            <svg viewBox="0 0 10 10" width="8" height="8"><path d="M2 1.5l7 3.5-7 3.5V1.5z" fill="currentColor"/></svg>
            Start
          </button>
          <button v-if="p.status === 'Running'" class="ra-btn stop" @click="stopProcess(p.id)" title="Stop">
            <svg viewBox="0 0 10 10" width="8" height="8"><rect x="1.5" y="1.5" width="7" height="7" rx="1" fill="currentColor"/></svg>
            Stop
          </button>
          <button class="ra-btn restart" @click="restartProcess(p.id)" title="Restart">
            <svg viewBox="0 0 12 12" width="9" height="9" fill="none">
              <path d="M10 6A4 4 0 112 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              <path d="M10 3v3h-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </button>
          <button class="ra-btn remove" @click="removeProcess(p.id)" title="Remove">
            <svg viewBox="0 0 10 12" width="8" height="9" fill="none">
              <path d="M1 3h8M3 3V1.5h4V3M2 3l.5 7h5L8 3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </button>
        </div>
      </div>

    </div>

    <!-- ── Footer ── -->
    <div class="sidebar-footer">
      <button class="new-process-btn" @click="showAddModal = true">
        <svg viewBox="0 0 12 12" width="11" height="11" fill="none">
          <path d="M6 1v10M1 6h10" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
        New Process
      </button>
    </div>

    <!-- Add process modal -->
    <AddProcessModal
      :show="showAddModal"
      @close="showAddModal = false"
      @added="onProcessAdded"
    />

  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useProcessStore } from '@/stores/processStore'
import { useDialog } from '@/composables/useDialog'
import AddProcessModal from '@/components/AddProcessModal.vue'

const store = useProcessStore()
const { openConfirm, openAlert } = useDialog()

const search = ref('')
const showAddModal = ref(false)

const runningCount = computed(() =>
  store.processes.filter((p: any) => p.status === 'Running').length
)

const filteredProcesses = computed(() => {
  if (!search.value.trim()) return store.processes
  const q = search.value.toLowerCase()
  return store.processes.filter(
    (p: any) => p.name.toLowerCase().includes(q) || p.command.toLowerCase().includes(q)
  )
})

const onProcessAdded = (id: string) => {
  store.selectedProcessId = id
}

const startProcess = async (id: string) => {
  try { await store.startProcess(id) }
  catch (e) { await openAlert('Start Failed', String(e)) }
}
const stopProcess = async (id: string) => {
  try { await store.stopProcess(id) }
  catch (e) { await openAlert('Stop Failed', String(e)) }
}
const restartProcess = async (id: string) => {
  try { await store.restartProcess(id) }
  catch (e) { await openAlert('Restart Failed', String(e)) }
}
const removeProcess = async (id: string) => {
  const p = store.processes.find((p: any) => p.id === id)
  const ok = await openConfirm(
    'Remove Process',
    `Remove "${p?.name}"? This cannot be undone.`,
    { type: 'danger', confirmLabel: 'Remove', cancelLabel: 'Cancel' }
  )
  if (!ok) return
  try { await store.removeProcess(id) }
  catch (e) { await openAlert('Remove Failed', String(e)) }
}
const startAllProcesses = async () => {
  try { await store.startAll() }
  catch (e) { await openAlert('Start All Failed', String(e)) }
}
const stopAllProcesses = async () => {
  try { await store.stopAll() }
  catch (e) { await openAlert('Stop All Failed', String(e)) }
}

const formatUptime = (ms: number) => {
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}s`
  if (s < 3600) return `${Math.floor(s / 60)}m ${s % 60}s`
  return `${Math.floor(s / 3600)}h ${Math.floor((s % 3600) / 60)}m`
}
</script>

<style scoped>
/* ── Shell ── */
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #0e0e0e;
  overflow: hidden;
}

/* ── Header ── */
.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 13px;
  height: 44px;
  border-bottom: 1px solid #1a1a1a;
  flex-shrink: 0;
}
.header-left { display: flex; align-items: center; gap: 8px; }
.sidebar-title {
  font-size: 0.7rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #374151;
}
.count-pill {
  display: flex;
  align-items: center;
  gap: 1px;
  font-size: 0.67rem;
  font-weight: 600;
  background: #141414;
  border: 1px solid #1e1e1e;
  border-radius: 20px;
  padding: 1px 8px;
  color: #4b5563;
}
.count-running { color: #4ade80; }
.count-sep { color: #222; margin: 0 1px; }
.header-actions { display: flex; gap: 5px; }
.icon-btn {
  width: 26px; height: 26px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 6px; cursor: pointer;
  transition: background 0.12s, border-color 0.12s;
  border: 1px solid transparent;
}
.icon-btn.success { background: #052e16; color: #22c55e; border-color: #14532d; }
.icon-btn.success:hover { background: #0a3d20; border-color: #16a34a; }
.icon-btn.danger  { background: #2d0a0a; color: #ef4444; border-color: #7f1d1d; }
.icon-btn.danger:hover  { background: #3d1010; border-color: #dc2626; }

/* ── Search ── */
.search-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 13px;
  height: 36px;
  border-bottom: 1px solid #161616;
  flex-shrink: 0;
  background: #080808;
}
.search-icon { flex-shrink: 0; }
.search-input {
  flex: 1; background: transparent; border: none; outline: none;
  font-size: 0.79rem; color: #94a3b8; caret-color: #6366f1;
}
.search-input::placeholder { color: #212d3a; }
.search-clear {
  background: none; border: none; color: #374151; cursor: pointer;
  padding: 2px; display: flex; align-items: center; transition: color 0.12s;
}
.search-clear:hover { color: #64748b; }

/* ── List ── */
.list {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

/* ── Empty ── */
.empty-state {
  display: flex; flex-direction: column; align-items: center;
  text-align: center; padding: 3rem 1.5rem 2rem; gap: 8px;
}
.empty-icon { margin-bottom: 6px; opacity: 0.6; }
.empty-title { font-size: 0.85rem; font-weight: 600; color: #374151; }
.empty-hint { font-size: 0.74rem; color: #1e293b; line-height: 1.55; }
.empty-hint strong { color: #4338ca; font-weight: 700; }

/* ── Process row ── */
.process-row {
  position: relative;
  cursor: pointer;
  transition: background 0.08s;
  border-bottom: 1px solid #111;
}
.process-row:last-child { border-bottom: none; }
.process-row:hover { background: #141414; }
.process-row.selected { background: #0c1224; }

.row-content {
  display: flex;
  align-items: stretch;
  padding: 10px 12px 10px 0;
  min-height: 52px;
}
.row-indicator {
  width: 3px;
  border-radius: 0 3px 3px 0;
  flex-shrink: 0;
  margin-right: 11px;
  align-self: stretch;
  transition: background 0.2s;
}
.ind-running { background: #22c55e; box-shadow: 0 0 6px #22c55e55; }
.ind-stopped { background: #1e1e1e; }
.ind-crashed { background: #ef4444; box-shadow: 0 0 6px #ef444455; }
.process-row.selected .ind-stopped { background: #312e81; }

.row-body { flex: 1; min-width: 0; }
.row-top {
  display: flex; align-items: center; gap: 7px; margin-bottom: 3px;
}
.p-name {
  flex: 1; font-size: 0.84rem; font-weight: 600;
  color: #94a3b8;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis; line-height: 1.3;
}
.process-row:hover .p-name { color: #cbd5e1; }
.process-row.selected .p-name { color: #e2e8f0; }

.status-pill {
  font-size: 0.57rem; font-weight: 800;
  letter-spacing: 0.06em; text-transform: uppercase;
  padding: 1.5px 6px; border-radius: 20px; flex-shrink: 0;
  border: 1px solid transparent;
}
.pill-running { background: #052e16; color: #4ade80; border-color: #14532d; }
.pill-stopped { background: #0f172a; color: #374151; border-color: #1e293b; }
.pill-crashed { background: #200a0a; color: #f87171; border-color: #450a0a; }

.row-bottom { display: flex; align-items: center; gap: 6px; }
.p-cmd {
  font-size: 0.69rem; color: #1e2d3d;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  font-family: 'Cascadia Code', 'Consolas', 'Courier New', monospace;
  flex: 1; min-width: 0;
}
.process-row:hover .p-cmd,
.process-row.selected .p-cmd { color: #2a3f55; }

.crash-badge {
  font-size: 0.62rem; font-weight: 700; color: #f87171;
  background: #200a0a; border: 1px solid #450a0a;
  padding: 0 5px; border-radius: 10px; flex-shrink: 0;
}

.row-meta {
  margin-top: 4px; font-size: 0.67rem;
  color: #2a3f3a;
  font-family: 'Cascadia Code', 'Consolas', monospace;
}
.process-row.selected .row-meta { color: #3a5a40; }

/* ── Row actions ── */
.row-actions {
  display: none;
  align-items: center;
  gap: 4px;
  padding: 0 12px 9px 14px;
}
.process-row:hover .row-actions,
.process-row.selected .row-actions { display: flex; }

.ra-btn {
  display: flex; align-items: center; gap: 4px;
  padding: 3px 9px;
  border: 1px solid transparent; border-radius: 5px;
  cursor: pointer; font-size: 0.69rem; font-weight: 600;
  transition: opacity 0.1s, filter 0.1s;
}
.ra-btn:hover { opacity: 0.85; }
.ra-btn.start   { background: #052e16; color: #4ade80; border-color: #14532d; }
.ra-btn.stop    { background: #200a0a; color: #f87171; border-color: #450a0a; }
.ra-btn.restart { background: #17120a; color: #fbbf24; border-color: #78350f; padding: 3px 7px; }
.ra-btn.remove  { background: #111; color: #374151; border-color: #1e1e1e; padding: 3px 7px; }
.ra-btn.remove:hover { background: #1a0505; color: #f87171; border-color: #450a0a; opacity: 1; }

/* ── Footer ── */
.sidebar-footer {
  padding: 10px 12px;
  border-top: 1px solid #1a1a1a;
  flex-shrink: 0;
  background: #090909;
}
.new-process-btn {
  width: 100%;
  display: flex; align-items: center; justify-content: center; gap: 8px;
  padding: 9px;
  background: #100e1a;
  border: 1.5px dashed #2d2a5e;
  border-radius: 8px;
  color: #4338ca;
  font-size: 0.79rem; font-weight: 700;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
  letter-spacing: 0.01em;
}
.new-process-btn:hover {
  background: #1e1b4b;
  border-color: #6366f1;
  border-style: solid;
  color: #a5b4fc;
}
</style>
