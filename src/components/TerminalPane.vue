<template>
  <div class="terminal-pane">
    <!-- Header bar -->
    <div class="terminal-header">
      <div class="terminal-header-left">
        <svg class="terminal-icon" viewBox="0 0 16 16" fill="none">
          <polyline points="2,5 7,8 2,11" stroke="#6366f1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          <line x1="8" y1="11" x2="14" y2="11" stroke="#6366f1" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        <span class="terminal-title">Terminal</span>
        <span class="terminal-cwd">{{ cwd }}</span>
      </div>
      <div class="terminal-header-right">
        <button class="th-btn" @click="clearOutput" title="Clear output">
          <svg viewBox="0 0 14 14" width="12" height="12">
            <path d="M2 3h10M5 3V2h4v1M3 3l.5 9h7l.5-9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
          </svg>
        </button>
        <button class="th-btn th-close" @click="$emit('close')" title="Close terminal">
          <svg viewBox="0 0 10 10" width="10" height="10">
            <path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Output area -->
    <div class="terminal-output" ref="outputEl" @click="focusInput">
      <template v-if="lines.length === 0">
        <div class="terminal-empty">
          Type a command and press <kbd>Enter</kbd>, or <kbd>Ctrl+C</kbd> to cancel a running command.
        </div>
      </template>
      <template v-for="(block, bi) in blocks" :key="bi">
        <!-- Command line header -->
        <div class="terminal-cmd-line">
          <span class="cmd-prompt">❯</span>
          <span class="cmd-text">{{ block.command }}</span>
          <span v-if="block.running" class="cmd-badge running">running</span>
          <span v-else-if="block.exitCode === 0" class="cmd-badge ok">done</span>
          <span v-else-if="block.exitCode !== null" class="cmd-badge err">exit {{ block.exitCode }}</span>
          <div class="cmd-actions" v-if="block.running || block.exitCode !== null">
            <button v-if="block.running" class="cmd-btn kill-btn" @click.stop="killJob(block.jobId)" title="Kill process">
              <svg viewBox="0 0 10 10" width="9" height="9"><rect x="1" y="1" width="8" height="8" rx="1" fill="currentColor"/></svg>
              Stop
            </button>
            <button class="cmd-btn promote-btn" @click.stop="openPromote(block)" title="Add to Process Manager">
              <svg viewBox="0 0 10 10" width="9" height="9"><path d="M5 1v8M1 5h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
              Add to processes
            </button>
          </div>
        </div>
        <!-- Output lines for this block -->
        <div
          v-for="(line, li) in block.lines"
          :key="li"
          class="terminal-line"
          :class="{ 'is-error': line.isError }"
          v-html="parseAnsi(line.text)"
        />
      </template>
    </div>

    <!-- Input bar -->
    <div class="terminal-input-bar">
      <span class="input-prompt">❯</span>
      <input
        ref="inputEl"
        v-model="inputValue"
        class="terminal-input"
        :disabled="isRunning"
        :placeholder="isRunning ? '' : 'Enter a command…'"
        spellcheck="false"
        autocomplete="off"
        autocorrect="off"
        @keydown.enter="runCommand"
        @keydown.ctrl.c.prevent="killActiveJob"
        @keydown.up.prevent="historyUp"
        @keydown.down.prevent="historyDown"
      />
      <div class="input-hint" v-if="isRunning">
        <span class="running-dot" />
        <span class="running-text">running</span>
        <button class="kill-inline" @click="killActiveJob" title="Ctrl+C">&#x25A0; stop</button>
      </div>
      <button v-else class="run-btn" @click="runCommand" :disabled="!inputValue.trim()">Run</button>
    </div>

    <!-- Promote modal -->
    <div v-if="promoteTarget" class="promote-overlay" @click.self="promoteTarget = null">
      <div class="promote-modal">
        <h3 class="promote-title">Add to Process Manager</h3>
        <p class="promote-sub">Give this process a name to track it in the sidebar.</p>
        <div class="promote-cmd-preview">{{ promoteTarget.command }}</div>
        <input
          ref="promoteInputEl"
          v-model="promoteName"
          class="promote-name-input"
          placeholder="Process name…"
          @keydown.enter="confirmPromote"
          @keydown.escape="promoteTarget = null"
        />
        <div v-if="promoteError" class="promote-error">{{ promoteError }}</div>
        <div class="promote-actions">
          <button class="promote-cancel" @click="promoteTarget = null">Cancel</button>
          <button class="promote-confirm" @click="confirmPromote" :disabled="!promoteName.trim() || promoteLoading">
            <span v-if="promoteLoading" class="spinner" />
            {{ promoteLoading ? 'Adding…' : 'Add Process' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useProcessStore } from '@/stores/processStore'

const SESSION_ID = 'default'

interface OutputLine { text: string; isError: boolean }
interface Block {
  jobId: string
  command: string
  lines: OutputLine[]
  running: boolean
  exitCode: number | null
}

const store = useProcessStore()
const cwd = ref('…')
const inputValue = ref('')
const inputEl = ref<HTMLInputElement | null>(null)
const outputEl = ref<HTMLDivElement | null>(null)
const promoteInputEl = ref<HTMLInputElement | null>(null)
const blocks = ref<Block[]>([])
const lines = computed(() => blocks.value.flatMap(b => b.lines))
const history = ref<string[]>([])
const historyIdx = ref(-1)
const activeJobId = ref<string | null>(null)
const isRunning = computed(() => activeJobId.value !== null)
const promoteTarget = ref<Block | null>(null)
const promoteName = ref('')
const promoteError = ref('')
const promoteLoading = ref(false)

let unlistenOutput: UnlistenFn | null = null
let unlistenDone: UnlistenFn | null = null

onMounted(async () => {
  try { cwd.value = await invoke<string>('terminal_get_cwd', { sessionId: SESSION_ID }) } catch { /* ignore */ }
  focusInput()
  setupListeners()
})

onUnmounted(() => {
  unlistenOutput?.()
  unlistenDone?.()
})

async function setupListeners() {
  unlistenOutput = await listen<{ jobId: string; line: string; isError: boolean; timestamp: string }>(
    `terminal:output:${SESSION_ID}`,
    (e) => {
      const block = blocks.value.find(b => b.jobId === e.payload.jobId)
      if (block) {
        block.lines.push({ text: e.payload.line, isError: e.payload.isError })
        scrollToBottom()
      }
    }
  )
  unlistenDone = await listen<{ jobId: string; exitCode: number }>(
    `terminal:done:${SESSION_ID}`,
    (e) => {
      const block = blocks.value.find(b => b.jobId === e.payload.jobId)
      if (block) {
        block.running = false
        block.exitCode = e.payload.exitCode
      }
      if (activeJobId.value === e.payload.jobId) {
        activeJobId.value = null
      }
      scrollToBottom()
      focusInput()
    }
  )
}

async function runCommand() {
  const raw = inputValue.value.trim()
  if (!raw || isRunning.value) return

  // save history
  history.value.unshift(raw)
  historyIdx.value = -1
  inputValue.value = ''

  // Handle cd specially
  if (/^cd(\s|$)/i.test(raw)) {
    const path = raw.slice(2).trim()
    const block: Block = {
      jobId: `cd-${Date.now()}`,
      command: raw,
      lines: [],
      running: false,
      exitCode: null,
    }
    blocks.value.push(block)
    if (!path) {
      // just show current directory
      block.exitCode = 0
    } else {
      try {
        cwd.value = await invoke<string>('terminal_set_cwd', { sessionId: SESSION_ID, path })
        block.exitCode = 0
      } catch (e: any) {
        block.lines.push({ text: String(e), isError: true })
        block.exitCode = 1
      }
    }
    scrollToBottom()
    focusInput()
    return
  }

  const jobId = `job-${Date.now()}`
  activeJobId.value = jobId

  const block: Block = {
    jobId,
    command: raw,
    lines: [],
    running: true,
    exitCode: null,
  }
  blocks.value.push(block)
  scrollToBottom()

  try {
    await invoke('terminal_run', { sessionId: SESSION_ID, command: raw, jobId })
  } catch (e: any) {
    block.lines.push({ text: String(e), isError: true })
    block.running = false
    block.exitCode = -1
    activeJobId.value = null
    focusInput()
  }
}

async function killJob(jobId: string) {
  try { await invoke('terminal_kill', { jobId }) } catch { /* ignore */ }
}

async function killActiveJob() {
  if (activeJobId.value) await killJob(activeJobId.value)
}

function clearOutput() {
  blocks.value = []
}

function focusInput() {
  nextTick(() => inputEl.value?.focus())
}

function scrollToBottom() {
  nextTick(() => {
    if (outputEl.value) outputEl.value.scrollTop = outputEl.value.scrollHeight
  })
}

function historyUp() {
  if (history.value.length === 0) return
  historyIdx.value = Math.min(historyIdx.value + 1, history.value.length - 1)
  inputValue.value = history.value[historyIdx.value]
}

function historyDown() {
  if (historyIdx.value <= 0) { historyIdx.value = -1; inputValue.value = ''; return }
  historyIdx.value--
  inputValue.value = history.value[historyIdx.value]
}

function openPromote(block: Block) {
  promoteTarget.value = block
  promoteName.value = block.command.split(' ')[0] || ''
  promoteError.value = ''
  nextTick(() => {
    promoteInputEl.value?.focus()
    promoteInputEl.value?.select()
  })
}

async function confirmPromote() {
  if (!promoteTarget.value || !promoteName.value.trim()) return
  promoteLoading.value = true
  promoteError.value = ''
  try {
    const id = await invoke<string>('terminal_add_process', {
      name: promoteName.value.trim(),
      command: promoteTarget.value.command,
      workingDir: cwd.value || null,
    })
    await store.loadProcesses()
    store.selectedProcessId = id
    promoteTarget.value = null
    promoteName.value = ''
  } catch (e: any) {
    promoteError.value = String(e)
  } finally {
    promoteLoading.value = false
  }
}

// ── ANSI parser (same approach as ProcessPanel) ──
function parseAnsi(raw: string): string {
  const ESC = '\x1b['
  let out = ''
  let i = 0
  let openSpan = false
  const htmlEsc = (s: string) =>
    s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  const colorMap: Record<number, string> = {
    30: '#4a4a4a', 31: '#e06c75', 32: '#98c379', 33: '#e5c07b',
    34: '#61afef', 35: '#c678dd', 36: '#56b6c2', 37: '#abb2bf',
    90: '#5c6370', 91: '#e06c75', 92: '#98c379', 93: '#e5c07b',
    94: '#61afef', 95: '#c678dd', 96: '#56b6c2', 97: '#ffffff',
    40: '#4a4a4a', 41: '#e06c75', 42: '#98c379', 43: '#e5c07b',
    44: '#61afef', 45: '#c678dd', 46: '#56b6c2', 47: '#abb2bf',
  }
  while (i < raw.length) {
    if (raw.startsWith(ESC, i)) {
      const end = raw.indexOf('m', i)
      if (end === -1) { i++; continue }
      const codes = raw.slice(i + 2, end).split(';').map(Number)
      if (openSpan) { out += '</span>'; openSpan = false }
      let fg = '', bg = '', bold = false
      for (const c of codes) {
        if (c === 0) { /* reset */ }
        else if (c === 1) bold = true
        else if ((c >= 30 && c <= 37) || (c >= 90 && c <= 97)) fg = colorMap[c] || ''
        else if ((c >= 40 && c <= 47)) bg = colorMap[c] || ''
      }
      if (fg || bg || bold) {
        const style = [
          fg ? `color:${fg}` : '',
          bg ? `background:${bg}` : '',
          bold ? 'font-weight:700' : '',
        ].filter(Boolean).join(';')
        out += `<span style="${style}">`
        openSpan = true
      }
      i = end + 1
    } else {
      out += htmlEsc(raw[i])
      i++
    }
  }
  if (openSpan) out += '</span>'
  return out
}
</script>

<style scoped>
.terminal-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #080808;
  font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
  font-size: 12.5px;
  overflow: hidden;
  position: relative;
}

/* ── Header ── */
.terminal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 34px;
  padding: 0 10px 0 12px;
  background: #0d0d0d;
  border-bottom: 1px solid #1e1e1e;
  flex-shrink: 0;
  user-select: none;
}
.terminal-header-left { display: flex; align-items: center; gap: 8px; overflow: hidden; min-width: 0; }
.terminal-header-right { display: flex; align-items: center; gap: 4px; flex-shrink: 0; }
.terminal-icon { width: 14px; height: 14px; flex-shrink: 0; }
.terminal-title { font-size: 0.75rem; font-weight: 600; color: #94a3b8; letter-spacing: 0.02em; flex-shrink: 0; }
.terminal-cwd {
  font-size: 0.7rem;
  color: #4b5563;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  direction: rtl;
  text-align: left;
  max-width: 400px;
}

.th-btn {
  width: 26px; height: 26px;
  display: flex; align-items: center; justify-content: center;
  background: transparent; border: none;
  color: #4b5563; cursor: pointer; border-radius: 4px;
  transition: background 0.12s, color 0.12s;
}
.th-btn:hover { background: #1e1e1e; color: #94a3b8; }
.th-close:hover { background: #7f1d1d !important; color: #fca5a5 !important; }

/* ── Output ── */
.terminal-output {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 1px;
  cursor: text;
}

.terminal-empty {
  color: #374151;
  font-family: inherit;
  font-size: 0.75rem;
  padding: 12px 0;
  user-select: none;
}
.terminal-empty kbd {
  background: #1e1e1e;
  border: 1px solid #2a2a2a;
  border-radius: 3px;
  padding: 1px 5px;
  color: #6366f1;
  font-family: inherit;
}

/* Command block header */
.terminal-cmd-line {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 10px;
  flex-wrap: wrap;
}
.terminal-cmd-line:first-child { margin-top: 0; }
.cmd-prompt { color: #6366f1; font-weight: 700; flex-shrink: 0; }
.cmd-text { color: #e2e8f0; word-break: break-all; flex: 1; min-width: 0; }
.cmd-badge {
  font-size: 0.62rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  padding: 1px 5px;
  border-radius: 3px;
  flex-shrink: 0;
}
.cmd-badge.running { background: #1e3a5f; color: #60a5fa; }
.cmd-badge.ok { background: #14402c; color: #4ade80; }
.cmd-badge.err { background: #3b0f0f; color: #f87171; }
.cmd-actions { display: flex; align-items: center; gap: 4px; margin-left: auto; }
.cmd-btn {
  display: flex; align-items: center; gap: 3px;
  padding: 2px 7px;
  background: #1e1e1e; border: 1px solid #2a2a2a;
  color: #94a3b8; cursor: pointer; border-radius: 4px;
  font-size: 0.68rem; font-family: inherit;
  transition: background 0.12s, border-color 0.12s;
}
.kill-btn:hover { background: #3b0f0f; border-color: #7f1d1d; color: #f87171; }
.promote-btn:hover { background: #1e2a1f; border-color: #166534; color: #4ade80; }

/* Output lines */
.terminal-line {
  color: #94a3b8;
  line-height: 1.55;
  white-space: pre-wrap;
  word-break: break-all;
  padding: 0 0 0 18px;
}
.terminal-line.is-error { color: #f87171; }

/* ── Input bar ── */
.terminal-input-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px 8px;
  background: #0d0d0d;
  border-top: 1px solid #1e1e1e;
  flex-shrink: 0;
}
.input-prompt { color: #6366f1; font-weight: 700; font-size: 1rem; }
.terminal-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #e2e8f0;
  font-family: inherit;
  font-size: inherit;
  caret-color: #6366f1;
}
.terminal-input::placeholder { color: #374151; }
.terminal-input:disabled { opacity: 0.5; cursor: not-allowed; }

.input-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
.running-dot {
  width: 6px; height: 6px;
  border-radius: 50%;
  background: #60a5fa;
  animation: pulse 1s infinite;
}
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}
.running-text { font-size: 0.7rem; color: #4b5563; }
.kill-inline {
  background: transparent; border: 1px solid #374151;
  color: #6b7280; cursor: pointer; border-radius: 3px;
  font-size: 0.68rem; padding: 1px 6px;
  font-family: inherit;
  transition: all 0.12s;
}
.kill-inline:hover { border-color: #7f1d1d; color: #f87171; background: #3b0f0f; }

.run-btn {
  padding: 3px 10px;
  background: #312e81; border: 1px solid #4338ca;
  color: #a5b4fc; cursor: pointer; border-radius: 4px;
  font-size: 0.72rem; font-family: inherit;
  font-weight: 600; flex-shrink: 0;
  transition: background 0.12s, border-color 0.12s;
}
.run-btn:hover:not(:disabled) { background: #3730a3; border-color: #6366f1; color: #c7d2fe; }
.run-btn:disabled { opacity: 0.3; cursor: not-allowed; }

/* ── Promote modal ── */
.promote-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0,0,0,0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  backdrop-filter: blur(3px);
}
.promote-modal {
  background: #111111;
  border: 1px solid #2a2a2a;
  border-radius: 10px;
  padding: 20px 22px;
  width: 360px;
  box-shadow: 0 20px 40px rgba(0,0,0,0.6);
}
.promote-title { font-size: 0.9rem; font-weight: 600; color: #e2e8f0; margin-bottom: 4px; }
.promote-sub { font-size: 0.75rem; color: #4b5563; margin-bottom: 12px; }
.promote-cmd-preview {
  background: #0a0a0a;
  border: 1px solid #1e1e1e;
  border-radius: 5px;
  padding: 6px 10px;
  font-size: 0.8rem;
  color: #6366f1;
  margin-bottom: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.promote-name-input {
  width: 100%;
  background: #0a0a0a;
  border: 1px solid #2a2a2a;
  border-radius: 6px;
  color: #e2e8f0;
  font-size: 0.82rem;
  padding: 7px 10px;
  outline: none;
  transition: border-color 0.15s;
  box-sizing: border-box;
}
.promote-name-input:focus { border-color: #4338ca; }
.promote-error { color: #f87171; font-size: 0.72rem; margin-top: 6px; }
.promote-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}
.promote-cancel {
  padding: 6px 14px;
  background: transparent; border: 1px solid #2a2a2a;
  color: #6b7280; border-radius: 6px; cursor: pointer;
  font-size: 0.8rem;
  transition: all 0.12s;
}
.promote-cancel:hover { background: #1e1e1e; color: #94a3b8; }
.promote-confirm {
  padding: 6px 14px;
  background: #166534; border: 1px solid #15803d;
  color: #4ade80; border-radius: 6px; cursor: pointer;
  font-size: 0.8rem; font-weight: 600;
  display: flex; align-items: center; gap: 6px;
  transition: all 0.12s;
}
.promote-confirm:hover:not(:disabled) { background: #14532d; border-color: #16a34a; }
.promote-confirm:disabled { opacity: 0.5; cursor: not-allowed; }

.spinner {
  width: 10px; height: 10px;
  border: 2px solid rgba(74,222,128,0.3);
  border-top-color: #4ade80;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  display: inline-block;
}
@keyframes spin { to { transform: rotate(360deg); } }

/* Scrollbar */
.terminal-output::-webkit-scrollbar { width: 4px; }
.terminal-output::-webkit-scrollbar-track { background: transparent; }
.terminal-output::-webkit-scrollbar-thumb { background: #1e1e1e; border-radius: 2px; }
</style>
