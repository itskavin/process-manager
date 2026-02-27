<template>
  <div class="sidebar">
    <!-- Header -->
    <div class="sidebar-header">
      <span class="sidebar-title">Processes</span>
      <div class="header-actions">
        <button class="icon-btn success" @click="startAllProcesses" title="Start All">
          <svg viewBox="0 0 10 10" width="9" height="9"><path d="M2 1.5l7 3.5-7 3.5V1.5z" fill="currentColor"/></svg>
        </button>
        <button class="icon-btn danger" @click="stopAllProcesses" title="Stop All">
          <svg viewBox="0 0 10 10" width="9" height="9"><rect x="2" y="2" width="6" height="6" rx="1" fill="currentColor"/></svg>
        </button>
      </div>
    </div>

    <!-- Add Process Form -->
    <div class="add-form">
      <div class="input-group">
        <label class="input-label">Name</label>
        <input
          v-model="newName"
          type="text"
          placeholder="e.g. API Server"
          class="field-input"
          @keyup.enter="focusCommand"
          ref="nameInput"
        />
      </div>
      <div class="input-group">
        <label class="input-label">Command</label>
        <input
          v-model="newCommand"
          type="text"
          placeholder="e.g. node server.js"
          class="field-input mono"
          @keyup.enter="focusWorkingDir"
          ref="commandInput"
        />
      </div>
      <div class="input-group">
        <label class="input-label">Working Directory <span class="label-opt">(optional)</span></label>
        <input
          v-model="newWorkingDir"
          type="text"
          placeholder="e.g. C:\\projects\\api"
          class="field-input mono"
          @keyup.enter="addProcess"
          ref="workingDirInput"
        />
      </div>
      <button class="add-btn" @click="addProcess" :disabled="adding">
        <svg v-if="!adding" viewBox="0 0 10 10" width="10" height="10"><path d="M5 1v8M1 5h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
        <span>{{ adding ? 'Adding…' : 'Add Process' }}</span>
      </button>
    </div>

    <!-- Process List -->
    <div class="list">
      <div
        v-for="p in filteredProcesses"
        :key="p.id"
        :class="['process-row', { selected: store.selectedProcessId === p.id }]"
        @click="store.selectedProcessId = p.id"
      >
        <div class="row-top">
          <span :class="['dot', dotClass(p.status)]"></span>
          <div class="row-info">
            <span class="p-name">{{ p.name }}</span>
            <span class="p-cmd">{{ p.command }}{{ p.args?.length ? ' ' + p.args.join(' ') : '' }}</span>
          </div>
          <span v-if="p.status === 'Crashed'" class="crash-tag" title="Crash count">{{ p.crashCount }}x</span>
        </div>

        <div class="row-actions" @click.stop>
          <button v-if="p.status !== 'Running'" class="act start" @click="startProcess(p.id)" title="Start">▶</button>
          <button v-if="p.status === 'Running'" class="act stop" @click="stopProcess(p.id)" title="Stop">⏹</button>
          <button class="act restart" @click="restartProcess(p.id)" title="Restart">↻</button>
          <button class="act remove" @click="removeProcess(p.id)" title="Remove">✕</button>
        </div>

        <div v-if="p.pid" class="row-meta">
          <span class="meta-item">PID&nbsp;{{ p.pid }}</span>
          <span class="meta-sep">·</span>
          <span class="meta-item">{{ formatUptime(p.uptimeMs) }}</span>
        </div>
      </div>

      <div v-if="store.processes.length === 0" class="empty">
        <span>No processes yet.</span>
        <span class="empty-hint">Add one above to get started.</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useProcessStore } from '@/stores/processStore'

const store = useProcessStore()
const search = ref('')
const showAdd = ref(false)
const newName = ref('')
const newCommand = ref('')
const newWorkingDir = ref('')
const adding = ref(false)
const commandInput = ref<HTMLInputElement>()
const workingDirInput = ref<HTMLInputElement>()

const focusCommand = () => {
  commandInput.value?.focus()
}

const focusWorkingDir = () => {
  workingDirInput.value?.focus()
}

const filteredProcesses = computed(() => {
  if (!search.value.trim()) return store.processes
  const q = search.value.toLowerCase()
  return store.processes.filter(
    (p: any) => p.name.toLowerCase().includes(q) || p.command.toLowerCase().includes(q)
  )
})

const parseCommand = (raw: string): { cmd: string; args: string[] } => {
  const parts = raw.trim().split(/\s+/)
  return { cmd: parts[0] ?? '', args: parts.slice(1) }
}

const addProcess = async () => {
  if (!newName.value.trim() || !newCommand.value.trim()) return
  adding.value = true
  try {
    const { cmd, args } = parseCommand(newCommand.value)
    await store.addProcess(
      newName.value.trim(),
      cmd,
      args,
      newWorkingDir.value.trim() || undefined
    )
    newName.value = ''
    newCommand.value = ''
    newWorkingDir.value = ''
    showAdd.value = false
  } catch (e) {
    alert(`Failed to add: ${e}`)
  } finally {
    adding.value = false
  }
}

const startProcess = async (id: string) => {
  try { await store.startProcess(id) } catch (e) { alert(`Start failed: ${e}`) }
}
const stopProcess = async (id: string) => {
  try { await store.stopProcess(id) } catch (e) { alert(`Stop failed: ${e}`) }
}
const restartProcess = async (id: string) => {
  try { await store.restartProcess(id) } catch (e) { alert(`Restart failed: ${e}`) }
}
const removeProcess = async (id: string) => {
  const p = store.processes.find((p: any) => p.id === id)
  if (!confirm(`Remove "${p?.name}"?`)) return
  try { await store.removeProcess(id) } catch (e) { alert(`Remove failed: ${e}`) }
}
const startAllProcesses = async () => {
  try { await store.startAll() } catch (e) { alert(`Start all failed: ${e}`) }
}
const stopAllProcesses = async () => {
  try { await store.stopAll() } catch (e) { alert(`Stop all failed: ${e}`) }
}

const dotClass = (status: string) => ({
  running: status === 'Running',
  stopped: status === 'Stopped',
  crashed: status === 'Crashed',
})

const formatUptime = (ms: number) => {
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}s`
  if (s < 3600) return `${Math.floor(s / 60)}m ${s % 60}s`
  return `${Math.floor(s / 3600)}h ${Math.floor((s % 3600) / 60)}m`
}
</script>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #111;
  overflow: hidden;
}

/* Header */
.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid #1a1a1a;
  flex-shrink: 0;
}
.sidebar-title {
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: #475569;
}
.count-badge {
  font-size: 0.65rem;
  background: #1e1e1e;
  color: #64748b;
  padding: 1px 6px;
  border-radius: 10px;
  border: 1px solid #2a2a2a;
}
.header-actions { display: flex; gap: 5px; }
.icon-btn {
  width: 28px; height: 28px;
  display: flex; align-items: center; justify-content: center;
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
  color: #475569;
}
.icon-btn:hover { background: #252525; border-color: #333; }
.icon-btn.success { background: #052e16; color: #22c55e; border-color: #14532d; }
.icon-btn.success:hover { background: #0a3d20; border-color: #16a34a; }
.icon-btn.danger  { background: #2d0a0a; color: #ef4444; border-color: #7f1d1d; }
.icon-btn.danger:hover  { background: #3d1010; border-color: #dc2626; }

/* Search */
.search-wrap {
  position: relative;
  padding: 8px 10px;
  border-bottom: 1px solid #1a1a1a;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 7px;
}
.search-icon { flex-shrink: 0; }
.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 0.8rem;
  color: #cbd5e1;
  caret-color: #6366f1;
}
.search-input::placeholder { color: #374151; }
.search-clear {
  background: none;
  border: none;
  color: #475569;
  cursor: pointer;
  font-size: 0.9rem;
  padding: 0 2px;
  line-height: 1;
}
.search-clear:hover { color: #94a3b8; }

/* List */
.list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.process-row {
  padding: 8px 12px;
  cursor: pointer;
  border-left: 2px solid transparent;
  transition: background 0.1s, border-color 0.1s;
}
.process-row:hover { background: #161616; }
.process-row.selected {
  background: #141c2e;
  border-left-color: #6366f1;
}

.row-top {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* Dot */
.dot {
  width: 7px; height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  transition: box-shadow 0.3s;
}
.dot.running { background: #22c55e; box-shadow: 0 0 6px #22c55e88; }
.dot.stopped { background: #334155; }
.dot.crashed { background: #ef4444; box-shadow: 0 0 6px #ef444488; }

.row-info { flex: 1; min-width: 0; }
.p-name {
  display: block;
  font-size: 0.84rem;
  font-weight: 600;
  color: #cbd5e1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}
.p-cmd {
  display: block;
  font-size: 0.72rem;
  color: #374151;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: 'Cascadia Code', 'Courier New', monospace;
}

.status-pill {
  font-size: 0.62rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  padding: 2px 7px;
  border-radius: 20px;
  flex-shrink: 0;
}
.pill-running { background: #052e16; color: #22c55e; border: 1px solid #14532d; }
.pill-stopped { background: #0f172a; color: #475569; border: 1px solid #1e293b; }
.pill-crashed { background: #2d0a0a; color: #ef4444; border: 1px solid #7f1d1d; }

.row-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 4px;
  padding-left: 15px;
  font-size: 0.69rem;
  color: #4b5563;
  font-family: 'Cascadia Code', 'Consolas', monospace;
}
.meta-item { color: #4b5563; }
.meta-sep { color: #2a2a2a; }
.crash-tag {
  font-size: 0.67rem;
  color: #ef4444;
  opacity: 0.8;
  margin-left: 4px;
}

/* Actions */
.row-actions {
  display: none;
  gap: 4px;
  margin-top: 6px;
  padding-left: 15px;
}
.process-row:hover .row-actions,
.process-row.selected .row-actions { display: flex; }

.act {
  padding: 2px 10px;
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.72rem;
  font-weight: 500;
  transition: opacity 0.12s;
}
.act:hover { opacity: 0.8; }
.act.start  { background: #052e16; color: #22c55e; border-color: #14532d; }
.act.stop   { background: #2d0a0a; color: #ef4444; border-color: #7f1d1d; }
.act.restart{ background: #1c1a08; color: #f59e0b; border-color: #78350f; }
.act.remove { background: #1a1a1a; color: #475569; border-color: #2a2a2a; }

/* Empty */
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 3rem 1rem;
  color: #334155;
  font-size: 0.82rem;
  text-align: center;
}
.empty-hint { font-size: 0.72rem; color: #1e293b; }

/* Add section */
.add-section {
  border-top: 1px solid #1a1a1a;
  flex-shrink: 0;
  padding: 8px 10px;
}

.toggle-add {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 6px;
  background: #161616;
  border: 1px solid #1e1e1e;
  border-radius: 6px;
  color: #475569;
  font-size: 0.78rem;
  cursor: pointer;
  transition: background 0.12s, color 0.12s, border-color 0.12s;
}
.toggle-add:hover { background: #1e1e1e; color: #94a3b8; border-color: #2a2a2a; }

.add-form {
  padding: 10px 12px;
  border-bottom: 1px solid #1a1a1a;
  display: flex;
  flex-direction: column;
  gap: 9px;
  flex-shrink: 0;
  background: #0f0f0f;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.input-label {
  font-size: 0.67rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: #4b5563;
  user-select: none;
}
.label-opt {
  font-weight: 400;
  text-transform: none;
  letter-spacing: 0;
  color: #374151;
}

.field-input {
  width: 100%;
  padding: 7px 10px;
  background: #141414;
  border: 1px solid #222;
  border-radius: 6px;
  color: #cbd5e1;
  font-size: 0.82rem;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
  font-family: inherit;
}
.field-input.mono {
  font-family: 'Cascadia Code', 'Consolas', 'Courier New', monospace;
  font-size: 0.78rem;
}
.field-input:focus {
  border-color: #6366f1;
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.12);
}
.field-input::placeholder { color: #2a3a4a; }
.field-input:hover:not(:focus) { border-color: #2e2e2e; }

.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px;
  background: #1e1b4b;
  border: 1px solid #312e81;
  border-radius: 6px;
  color: #a5b4fc;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;
  margin-top: 2px;
}
.add-btn:hover:not(:disabled) { background: #2d2a5e; border-color: #4338ca; color: #c7d2fe; }
.add-btn:disabled { opacity: 0.4; cursor: not-allowed; }

/* Slide transition */
.slide-enter-active,
.slide-leave-active { transition: all 0.18s ease; }
.slide-enter-from,
.slide-leave-to { opacity: 0; transform: translateY(-6px); max-height: 0; }
.slide-enter-to,
.slide-leave-from { opacity: 1; transform: translateY(0); max-height: 300px; }
</style>
