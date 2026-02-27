<template>
  <teleport to="body">
    <transition name="modal-fade">
      <div v-if="show" class="modal-overlay" @mousedown.self="$emit('close')">
        <div class="modal-card" role="dialog" aria-modal="true">

          <!-- Header -->
          <div class="modal-header">
            <div class="modal-header-icon">
              <svg viewBox="0 0 14 14" width="14" height="14" fill="none">
                <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
              </svg>
            </div>
            <div>
              <p class="modal-title">New Process</p>
              <p class="modal-subtitle">Configure and add a new managed process</p>
            </div>
            <button class="modal-close" @click="$emit('close')" title="Close">
              <svg viewBox="0 0 10 10" width="10" height="10">
                <path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
              </svg>
            </button>
          </div>

          <!-- Body -->
          <div class="modal-body">
            <div class="field-group">
              <label class="field-label">Name <span class="field-required">*</span></label>
              <input
                v-model="form.name"
                ref="nameInput"
                type="text"
                class="field-input"
                placeholder="e.g. API Server"
                @keyup.enter="focusNext('cmd')"
                autocomplete="off"
              />
            </div>

            <div class="field-group">
              <label class="field-label">Command <span class="field-required">*</span></label>
              <input
                v-model="form.command"
                ref="cmdInput"
                type="text"
                class="field-input mono"
                placeholder="e.g. node server.js  or  python app.py"
                @keyup.enter="focusNext('dir')"
                autocomplete="off"
              />
              <span class="field-hint">Arguments can be included inline (e.g. <code>npm run dev -- --port 3000</code>)</span>
            </div>

            <div class="field-group">
              <label class="field-label">Working Directory <span class="field-opt">(optional)</span></label>
              <input
                v-model="form.workingDir"
                ref="dirInput"
                type="text"
                class="field-input mono"
                placeholder="e.g. C:\projects\my-app"
                @keyup.enter="submit"
                autocomplete="off"
              />
            </div>

            <div class="field-row">
              <label class="check-row" @click="form.autoRestart = !form.autoRestart">
                <div :class="['toggle-sm', { on: form.autoRestart }]">
                  <div class="toggle-thumb-sm"></div>
                </div>
                <span class="check-label">Auto-restart on crash</span>
              </label>
              <label class="check-row" @click="form.autoStart = !form.autoStart">
                <div :class="['toggle-sm', { on: form.autoStart }]">
                  <div class="toggle-thumb-sm"></div>
                </div>
                <span class="check-label">Auto-start on launch</span>
              </label>
            </div>

            <p v-if="error" class="form-error">
              <svg viewBox="0 0 12 12" width="11" height="11" fill="none">
                <circle cx="6" cy="6" r="5" stroke="currentColor" stroke-width="1.2"/>
                <path d="M6 4v3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
                <circle cx="6" cy="9" r="0.7" fill="currentColor"/>
              </svg>
              {{ error }}
            </p>
          </div>

          <!-- Footer -->
          <div class="modal-footer">
            <button class="btn-cancel" @click="$emit('close')">Cancel</button>
            <button class="btn-add" @click="submit" :disabled="adding">
              <svg v-if="!adding" viewBox="0 0 12 12" width="11" height="11" fill="none">
                <path d="M6 1v10M1 6h10" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
              </svg>
              <svg v-else viewBox="0 0 24 24" width="13" height="13" fill="none" class="spin">
                <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="2.5" stroke-dasharray="40 20" stroke-linecap="round"/>
              </svg>
              {{ adding ? 'Adding…' : 'Add Process' }}
            </button>
          </div>

        </div>
      </div>
    </transition>
  </teleport>
</template>

<script setup lang="ts">
import { reactive, ref, watch, nextTick } from 'vue'
import { useProcessStore } from '@/stores/processStore'

const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{ (e: 'close'): void; (e: 'added', id: string): void }>()

const store = useProcessStore()

const nameInput = ref<HTMLInputElement>()
const cmdInput  = ref<HTMLInputElement>()
const dirInput  = ref<HTMLInputElement>()

const form = reactive({ name: '', command: '', workingDir: '', autoRestart: false, autoStart: false })
const adding = ref(false)
const error  = ref('')

watch(() => props.show, async (v) => {
  if (v) {
    form.name = ''; form.command = ''; form.workingDir = ''
    form.autoRestart = false; form.autoStart = false
    error.value = ''
    await nextTick()
    nameInput.value?.focus()
  }
})

const focusNext = (target: 'cmd' | 'dir') => {
  if (target === 'cmd') cmdInput.value?.focus()
  else dirInput.value?.focus()
}

const parseCommand = (raw: string) => {
  const parts = raw.trim().split(/\s+/)
  return { cmd: parts[0] ?? '', args: parts.slice(1) }
}

const submit = async () => {
  error.value = ''
  if (!form.name.trim())    { error.value = 'Name is required.'; nameInput.value?.focus(); return }
  if (!form.command.trim()) { error.value = 'Command is required.'; cmdInput.value?.focus(); return }
  adding.value = true
  try {
    const { cmd, args } = parseCommand(form.command)
    const id = await store.addProcess(form.name.trim(), cmd, args, form.workingDir.trim() || undefined)
    emit('added', id)
    emit('close')
  } catch (e) {
    error.value = `Failed to add: ${e}`
  } finally {
    adding.value = false
  }
}

// Close on Escape
const onKey = (e: KeyboardEvent) => { if (props.show && e.key === 'Escape') emit('close') }
window.addEventListener('keydown', onKey)
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9998;
  padding: 16px;
}

.modal-card {
  background: #131313;
  border: 1px solid #252525;
  border-radius: 14px;
  width: 420px;
  max-width: 100%;
  display: flex;
  flex-direction: column;
  box-shadow: 0 32px 80px rgba(0,0,0,0.8), 0 0 0 1px rgba(255,255,255,0.04);
  overflow: hidden;
}

/* Header */
.modal-header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 18px 18px 14px;
  border-bottom: 1px solid #1e1e1e;
}
.modal-header-icon {
  width: 34px; height: 34px; border-radius: 9px; flex-shrink: 0;
  background: #1e1b4b; border: 1px solid #312e81; color: #a5b4fc;
  display: flex; align-items: center; justify-content: center;
}
.modal-title {
  font-size: 0.9rem; font-weight: 700; color: #f1f5f9; line-height: 1.2;
}
.modal-subtitle {
  font-size: 0.74rem; color: #4b5563; margin-top: 2px; line-height: 1.4;
}
.modal-close {
  margin-left: auto; flex-shrink: 0;
  width: 28px; height: 28px; border-radius: 7px;
  background: transparent; border: 1px solid transparent;
  color: #475569; cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  transition: background 0.12s, color 0.12s, border-color 0.12s;
}
.modal-close:hover { background: #1e1e1e; color: #94a3b8; border-color: #2a2a2a; }

/* Body */
.modal-body {
  padding: 18px;
  display: flex; flex-direction: column; gap: 14px;
}

.field-group { display: flex; flex-direction: column; gap: 5px; }
.field-label {
  font-size: 0.69rem; font-weight: 700;
  text-transform: uppercase; letter-spacing: 0.07em;
  color: #4b5563;
}
.field-required { color: #6366f1; }
.field-opt { font-weight: 400; text-transform: none; letter-spacing: 0; color: #374151; }

.field-input {
  width: 100%; padding: 8px 11px;
  background: #0f0f0f; border: 1px solid #222;
  border-radius: 7px; color: #cbd5e1;
  font-size: 0.83rem; font-family: inherit; outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.field-input.mono { font-family: 'Cascadia Code', 'Consolas', 'Courier New', monospace; font-size: 0.79rem; }
.field-input:focus { border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,0.12); }
.field-input:hover:not(:focus) { border-color: #2e2e2e; }
.field-input::placeholder { color: #2a3a4a; }

.field-hint {
  font-size: 0.69rem; color: #374151; line-height: 1.4;
}
.field-hint code {
  font-family: 'Cascadia Code', 'Consolas', monospace;
  background: #1a1a1a; padding: 1px 4px; border-radius: 3px; color: #6366f1;
}

.field-row {
  display: flex; gap: 18px; flex-wrap: wrap;
}
.check-row {
  display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none;
}
.check-label { font-size: 0.78rem; color: #64748b; }
.check-row:hover .check-label { color: #94a3b8; }

.toggle-sm {
  width: 28px; height: 15px; border-radius: 8px;
  background: #1e293b; border: 1px solid #334155;
  position: relative; flex-shrink: 0;
  transition: background 0.18s, border-color 0.18s;
}
.toggle-sm.on { background: #4338ca; border-color: #6366f1; }
.toggle-thumb-sm {
  position: absolute; top: 1.5px; left: 1.5px;
  width: 10px; height: 10px; border-radius: 50%;
  background: #475569;
  transition: transform 0.18s, background 0.18s;
}
.toggle-sm.on .toggle-thumb-sm { transform: translateX(13px); background: #fff; }

.form-error {
  display: flex; align-items: center; gap: 6px;
  font-size: 0.77rem; color: #f87171;
  background: #200a0a; border: 1px solid #7f1d1d;
  padding: 8px 11px; border-radius: 7px;
}

/* Footer */
.modal-footer {
  display: flex; align-items: center; justify-content: flex-end; gap: 8px;
  padding: 14px 18px;
  border-top: 1px solid #1a1a1a;
  background: #0f0f0f;
}
.btn-cancel {
  padding: 8px 18px; border-radius: 7px;
  background: #1a1a1a; border: 1px solid #2a2a2a;
  color: #64748b; font-size: 0.8rem; font-weight: 600; cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.btn-cancel:hover { background: #222; color: #94a3b8; }

.btn-add {
  padding: 8px 20px; border-radius: 7px;
  background: #1e1b4b; border: 1px solid #312e81;
  color: #a5b4fc; font-size: 0.8rem; font-weight: 700; cursor: pointer;
  display: flex; align-items: center; gap: 7px;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
}
.btn-add:hover:not(:disabled) { background: #2d2a5e; border-color: #4338ca; color: #c7d2fe; }
.btn-add:disabled { opacity: 0.45; cursor: not-allowed; }

/* Spinner */
@keyframes spin { to { transform: rotate(360deg); } }
.spin { animation: spin 0.7s linear infinite; }

/* Animations */
.modal-fade-enter-active { transition: opacity 0.16s, transform 0.16s cubic-bezier(.16,1,.3,1); }
.modal-fade-leave-active { transition: opacity 0.12s, transform 0.1s ease-in; }
.modal-fade-enter-from { opacity: 0; transform: scale(0.94) translateY(8px); }
.modal-fade-leave-to   { opacity: 0; transform: scale(0.97) translateY(4px); }
</style>
