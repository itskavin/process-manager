<template>
  <div class="app">
    <!-- Custom frameless titlebar -->
    <div class="titlebar" data-tauri-drag-region>
      <div class="titlebar-left" data-tauri-drag-region>
        <svg class="app-logo" viewBox="0 0 16 16" fill="none">
          <path d="M9.5 1L3 9h5.5L6.5 15 13 7H7.5L9.5 1z" fill="#6366f1" stroke="#818cf8" stroke-width="0.5" stroke-linejoin="round"/>
        </svg>
        <span class="app-name">Process Manager</span>
      </div>
      <div class="titlebar-center" data-tauri-drag-region>
        <span class="process-count" v-if="store.processes.length">
          {{ runningCount }}/{{ store.processes.length }} running
        </span>
      </div>
      <div class="titlebar-controls">
        <button class="wc wc-terminal" :class="{ active: terminalOpen }" @click.stop="terminalOpen = !terminalOpen" title="Toggle terminal">
          <svg viewBox="0 0 14 14" width="12" height="12" fill="none">
            <polyline points="1,4 5,7 1,10" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
            <line x1="6" y1="10" x2="13" y2="10" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
          </svg>
        </button>
        <button class="wc wc-min" @click.stop="minimize" title="Minimize">
          <svg viewBox="0 0 10 1" width="10" height="1"><rect width="10" height="1" fill="currentColor"/></svg>
        </button>
        <button class="wc wc-max" @click.stop="toggleMaximize" title="Maximize">
          <svg viewBox="0 0 10 10" width="10" height="10"><rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor" fill="none"/></svg>
        </button>
        <button class="wc wc-close" @click.stop="hideToTray" title="Minimize to tray">
          <svg viewBox="0 0 10 10" width="10" height="10">
            <path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Main workspace -->
    <div class="workspace">
      <aside class="sidebar">
        <ProcessList />
      </aside>
      <div class="main-col">
        <main class="main">
          <ProcessPanel />
        </main>
        <!-- Resizable terminal panel -->
        <div
          v-if="terminalOpen"
          class="terminal-panel"
          :style="{ height: terminalHeight + 'px' }"
        >
          <div class="terminal-resize-handle" @mousedown="startResize" />
          <TerminalPane @close="terminalOpen = false" />
        </div>
      </div>
    </div>
    <AppDialog />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useProcessStore } from '@/stores/processStore'
import ProcessList from '@/components/ProcessList.vue'
import ProcessPanel from '@/components/ProcessPanel.vue'
import AppDialog from '@/components/AppDialog.vue'
import TerminalPane from '@/components/TerminalPane.vue'

const store = useProcessStore()
const win = getCurrentWindow()

const terminalOpen = ref(false)
const terminalHeight = ref(280)
let resizing = false
let resizeStartY = 0
let resizeStartH = 0

function startResize(e: MouseEvent) {
  resizing = true
  resizeStartY = e.clientY
  resizeStartH = terminalHeight.value
  window.addEventListener('mousemove', onResize)
  window.addEventListener('mouseup', stopResize)
}
function onResize(e: MouseEvent) {
  if (!resizing) return
  const delta = resizeStartY - e.clientY
  terminalHeight.value = Math.max(140, Math.min(700, resizeStartH + delta))
}
function stopResize() {
  resizing = false
  window.removeEventListener('mousemove', onResize)
  window.removeEventListener('mouseup', stopResize)
}

const runningCount = computed(() =>
  store.processes.filter((p: any) => p.status === 'Running').length
)

const minimize = () => win.minimize()
const toggleMaximize = () => win.toggleMaximize()
const hideToTray = () => win.hide()

let unlistenStartAll: UnlistenFn | null = null
let unlistenStopAll: UnlistenFn | null = null
let pollInterval: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  try { await store.loadConfig() } catch { /* fresh install */ }
  await store.loadProcesses()

  pollInterval = setInterval(async () => {
    try { await store.loadProcesses() } catch { /* ignore */ }
  }, 2000)

  unlistenStartAll = await listen('app:start_all', async () => {
    try { await store.startAll() } catch { /* ignore */ }
  })
  unlistenStopAll = await listen('app:stop_all', async () => {
    try { await store.stopAll() } catch { /* ignore */ }
  })
})

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval)
  unlistenStartAll?.()
  unlistenStopAll?.()
})
</script>

<style>
* { box-sizing: border-box; margin: 0; padding: 0; }

html, body, #app {
  width: 100%;
  height: 100%;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif;
  background: #0a0a0a;
  color: #e2e8f0;
  font-size: 13px;
}

.app {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
  overflow: hidden;
  background: #0a0a0a;
}

/* ── Titlebar ── */
.titlebar {
  display: flex;
  align-items: center;
  height: 38px;
  background: #0d0d0d;
  border-bottom: 1px solid #1e1e1e;
  flex-shrink: 0;
  user-select: none;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 14px;
  flex: 0 0 auto;
}

.app-logo { width: 16px; height: 16px; flex-shrink: 0; }

.app-name {
  font-size: 0.8rem;
  font-weight: 600;
  color: #94a3b8;
  letter-spacing: 0.01em;
  white-space: nowrap;
}

.titlebar-center {
  flex: 1;
  display: flex;
  justify-content: center;
}

.process-count {
  font-size: 0.72rem;
  color: #475569;
  letter-spacing: 0.02em;
}

.titlebar-controls {
  display: flex;
  align-items: stretch;
  height: 38px;
  flex-shrink: 0;
}

.wc {
  width: 46px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #6b7280;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.wc:hover { background: #1e1e1e; color: #e2e8f0; }
.wc-close:hover { background: #c0392b !important; color: #fff !important; }
.wc-terminal.active { color: #6366f1; background: #1e1e2e; }

/* ── Workspace ── */
.workspace {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

.sidebar {
  width: 290px;
  min-width: 220px;
  flex-shrink: 0;
  border-right: 1px solid #1a1a1a;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-col {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.terminal-panel {
  flex-shrink: 0;
  border-top: 1px solid #1a1a1a;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}

.terminal-resize-handle {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  cursor: ns-resize;
  z-index: 10;
  background: transparent;
  transition: background 0.15s;
}
.terminal-resize-handle:hover { background: #6366f1; }

/* ── Scrollbar ── */
::-webkit-scrollbar { width: 5px; height: 5px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: #2a2a2a; border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: #3a3a3a; }
</style>