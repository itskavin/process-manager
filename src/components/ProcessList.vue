<template>
  <div class="sidebar">
    <!-- Header -->
    <div class="sidebar-header">
      <span class="sidebar-title">Processes</span>
      <div class="header-actions">
        <button class="icon-btn success" @click="startAllProcesses" title="Start All">▶</button>
        <button class="icon-btn danger" @click="stopAllProcesses" title="Stop All">⏹</button>
      </div>
    </div>

    <!-- Add Process Form -->
    <div class="add-form">
      <input
        v-model="newName"
        type="text"
        placeholder="Name"
        class="input"
        @keyup.enter="focusCommand"
        ref="nameInput"
      />
      <input
        v-model="newCommand"
        type="text"
        placeholder="Command  (e.g. node server.js)"
        class="input"
        @keyup.enter="addProcess"
        ref="commandInput"
      />
      <button class="add-btn" @click="addProcess" :disabled="adding">
        {{ adding ? '…' : '+ Add Process' }}
      </button>
    </div>

    <!-- Process List -->
    <div class="list">
      <div
        v-for="p in store.processes"
        :key="p.id"
        :class="['process-row', { selected: store.selectedProcessId === p.id }]"
        @click="store.selectedProcessId = p.id"
      >
        <div class="row-main">
          <span :class="['dot', dotClass(p.status)]"></span>
          <div class="row-text">
            <span class="p-name">{{ p.name }}</span>
            <span class="p-cmd">{{ p.command }}{{ p.args?.length ? ' ' + p.args.join(' ') : '' }}</span>
          </div>
          <span v-if="p.status === 'Crashed'" class="crash-badge" title="Crash count">{{ p.crashCount }}x</span>
        </div>

        <div class="row-actions" @click.stop>
          <button v-if="p.status !== 'Running'" class="act-btn start" @click="startProcess(p.id)" title="Start">▶</button>
          <button v-if="p.status === 'Running'" class="act-btn stop" @click="stopProcess(p.id)" title="Stop">⏹</button>
          <button class="act-btn restart" @click="restartProcess(p.id)" title="Restart">↻</button>
          <button class="act-btn remove" @click="removeProcess(p.id)" title="Remove">✕</button>
        </div>

        <div v-if="p.pid" class="row-meta">
          PID {{ p.pid }} &nbsp;·&nbsp; {{ formatUptime(p.uptimeMs) }}
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
import { ref } from 'vue'
import { useProcessStore } from '@/stores/processStore'

const store = useProcessStore()
const newName = ref('')
const newCommand = ref('')
const adding = ref(false)
const nameInput = ref<HTMLInputElement>()
const commandInput = ref<HTMLInputElement>()

const focusCommand = () => commandInput.value?.focus()

const parseCommand = (raw: string): { cmd: string; args: string[] } => {
  const parts = raw.trim().split(/\s+/)
  return { cmd: parts[0] ?? '', args: parts.slice(1) }
}

const addProcess = async () => {
  if (!newName.value.trim() || !newCommand.value.trim()) return
  adding.value = true
  try {
    const { cmd, args } = parseCommand(newCommand.value)
    await store.addProcess(newName.value.trim(), cmd, args)
    newName.value = ''
    newCommand.value = ''
    nameInput.value?.focus()
  } catch (e) {
    alert(`Failed to add process: ${e}`)
  } finally {
    adding.value = false
  }
}

const startProcess = async (id: string) => {
  try { await store.startProcess(id) }
  catch (e) { alert(`Start failed: ${e}`) }
}

const stopProcess = async (id: string) => {
  try { await store.stopProcess(id) }
  catch (e) { alert(`Stop failed: ${e}`) }
}

const restartProcess = async (id: string) => {
  try { await store.restartProcess(id) }
  catch (e) { alert(`Restart failed: ${e}`) }
}

const removeProcess = async (id: string) => {
  if (!confirm(`Remove "${store.processes.find(p => p.id === id)?.name}"?`)) return
  try { await store.removeProcess(id) }
  catch (e) { alert(`Remove failed: ${e}`) }
}

const startAllProcesses = async () => {
  try { await store.startAll() }
  catch (e) { alert(`Start all failed: ${e}`) }
}

const stopAllProcesses = async () => {
  try { await store.stopAll() }
  catch (e) { alert(`Stop all failed: ${e}`) }
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
  background: #1a1a1a;
  overflow: hidden;
}

/* Header */
.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border-bottom: 1px solid #2e2e2e;
  flex-shrink: 0;
}
.sidebar-title {
  font-size: 0.78rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #6b7280;
}
.header-actions { display: flex; gap: 6px; }
.icon-btn {
  width: 28px; height: 28px;
  border: none; border-radius: 5px;
  cursor: pointer; font-size: 0.72rem;
  display: flex; align-items: center; justify-content: center;
  transition: opacity .15s;
}
.icon-btn:hover { opacity: 0.8; }
.icon-btn.success { background: #14532d; color: #4ade80; }
.icon-btn.danger  { background: #7f1d1d; color: #f87171; }

/* Add Form */
.add-form {
  padding: 10px 12px;
  border-bottom: 1px solid #2e2e2e;
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-shrink: 0;
}
.input {
  width: 100%;
  padding: 6px 10px;
  background: #252525;
  border: 1px solid #3a3a3a;
  border-radius: 5px;
  color: #e8e8e8;
  font-size: 0.82rem;
  outline: none;
  box-sizing: border-box;
}
.input:focus { border-color: #3b82f6; }
.input::placeholder { color: #4b5563; }
.add-btn {
  padding: 7px 12px;
  background: #1e3a5f;
  border: 1px solid #2563eb;
  border-radius: 5px;
  color: #93c5fd;
  font-size: 0.82rem;
  cursor: pointer;
  transition: background .15s;
}
.add-btn:hover:not(:disabled) { background: #1e40af; }
.add-btn:disabled { opacity: 0.5; cursor: not-allowed; }

/* List */
.list {
  flex: 1;
  overflow-y: auto;
}
.process-row {
  padding: 9px 12px;
  border-bottom: 1px solid #212121;
  cursor: pointer;
  transition: background .1s;
  border-left: 2px solid transparent;
}
.process-row:hover { background: #212121; }
.process-row.selected { background: #1c2d4a; border-left-color: #3b82f6; }

.row-main {
  display: flex;
  align-items: center;
  gap: 8px;
}
.dot {
  width: 8px; height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot.running { background: #4ade80; box-shadow: 0 0 5px #4ade8077; }
.dot.stopped { background: #52525b; }
.dot.crashed { background: #f87171; box-shadow: 0 0 5px #f8717177; }

.row-text { flex: 1; min-width: 0; }
.p-name {
  display: block;
  font-size: 0.85rem;
  font-weight: 600;
  color: #e2e8f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.p-cmd {
  display: block;
  font-size: 0.74rem;
  color: #6b7280;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.crash-badge {
  font-size: 0.68rem;
  background: #7f1d1d;
  color: #fca5a5;
  padding: 1px 5px;
  border-radius: 3px;
  flex-shrink: 0;
}

/* Action buttons */
.row-actions {
  display: none;
  gap: 4px;
  margin-top: 6px;
}
.process-row:hover .row-actions,
.process-row.selected .row-actions {
  display: flex;
}
.act-btn {
  padding: 3px 8px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.75rem;
  transition: opacity .15s;
}
.act-btn:hover { opacity: 0.8; }
.act-btn.start   { background: #14532d; color: #4ade80; }
.act-btn.stop    { background: #7f1d1d; color: #f87171; }
.act-btn.restart { background: #713f12; color: #fbbf24; }
.act-btn.remove  { background: #27272a; color: #71717a; }

.row-meta {
  font-size: 0.72rem;
  color: #3f4c5c;
  margin-top: 4px;
  padding-left: 16px;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem 1rem;
  gap: 6px;
  color: #4b5563;
  font-size: 0.85rem;
  text-align: center;
}
.empty-hint { font-size: 0.75rem; color: #374151; }
</style>
