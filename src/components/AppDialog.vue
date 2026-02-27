<template>
  <teleport to="body">
    <transition name="dialog-fade">
      <div v-if="state.show" class="dialog-overlay" @mousedown.self="onBackdrop">
        <div :class="['dialog-card', `dialog-${state.type}`]" role="dialog" aria-modal="true">
          <!-- Icon -->
          <div class="dialog-icon-wrap">
            <svg v-if="state.type === 'alert'" viewBox="0 0 20 20" width="20" height="20" fill="none">
              <circle cx="10" cy="10" r="8.5" stroke="currentColor" stroke-width="1.5"/>
              <path d="M10 6v4.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
              <circle cx="10" cy="13.5" r="0.9" fill="currentColor"/>
            </svg>
            <svg v-else-if="state.type === 'danger'" viewBox="0 0 20 20" width="20" height="20" fill="none">
              <path d="M10 2L18.5 17H1.5L10 2z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
              <path d="M10 8v4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
              <circle cx="10" cy="14.5" r="0.9" fill="currentColor"/>
            </svg>
            <svg v-else viewBox="0 0 20 20" width="20" height="20" fill="none">
              <circle cx="10" cy="10" r="8.5" stroke="currentColor" stroke-width="1.5"/>
              <path d="M7 10l2 2 4-4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>

          <div class="dialog-body">
            <p class="dialog-title">{{ state.title }}</p>
            <p class="dialog-msg">{{ state.message }}</p>
          </div>

          <div class="dialog-actions">
            <button
              v-if="state.cancelLabel"
              class="dialog-btn cancel"
              @click="dismiss(false)"
            >{{ state.cancelLabel }}</button>
            <button
              :class="['dialog-btn', 'confirm', state.type]"
              @click="dismiss(true)"
              ref="confirmBtn"
            >{{ state.confirmLabel }}</button>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>

<script setup lang="ts">
import { watch, nextTick, ref } from 'vue'
import { useDialog } from '@/composables/useDialog'

const { state, dismiss } = useDialog()
const confirmBtn = ref<HTMLButtonElement>()

// auto-focus confirm button when dialog opens
watch(() => state.show, async (v) => {
  if (v) { await nextTick(); confirmBtn.value?.focus() }
})

// close on Escape
const onKey = (e: KeyboardEvent) => {
  if (!state.show) return
  if (e.key === 'Escape') dismiss(false)
  if (e.key === 'Enter') dismiss(true)
}
window.addEventListener('keydown', onKey)

// backdrop only closes alert, not confirm
const onBackdrop = () => {
  if (state.type === 'alert') dismiss(true)
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(3px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.dialog-card {
  background: #161616;
  border: 1px solid #2a2a2a;
  border-radius: 12px;
  width: 340px;
  max-width: calc(100vw - 32px);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.7), 0 0 0 1px rgba(255,255,255,0.03);
}

.dialog-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  flex-shrink: 0;
  align-self: flex-start;
}
.dialog-alert .dialog-icon-wrap { background: #1c2333; color: #60a5fa; border: 1px solid #1e3a5f; }
.dialog-confirm .dialog-icon-wrap { background: #0f1f12; color: #4ade80; border: 1px solid #14532d; }
.dialog-danger .dialog-icon-wrap { background: #200a0a; color: #f87171; border: 1px solid #7f1d1d; }

.dialog-body { display: flex; flex-direction: column; gap: 5px; }
.dialog-title {
  font-size: 0.92rem;
  font-weight: 600;
  color: #f1f5f9;
  line-height: 1.3;
}
.dialog-msg {
  font-size: 0.8rem;
  color: #64748b;
  line-height: 1.5;
}

.dialog-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 4px;
}

.dialog-btn {
  padding: 7px 18px;
  border-radius: 7px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
  outline: none;
}
.dialog-btn:focus-visible { outline: 2px solid #6366f1; outline-offset: 2px; }

.dialog-btn.cancel {
  background: #1a1a1a;
  border-color: #2a2a2a;
  color: #64748b;
}
.dialog-btn.cancel:hover { background: #222; color: #94a3b8; border-color: #333; }

.dialog-btn.confirm.alert,
.dialog-btn.confirm.confirm {
  background: #1e1b4b;
  border-color: #312e81;
  color: #a5b4fc;
}
.dialog-btn.confirm.alert:hover,
.dialog-btn.confirm.confirm:hover { background: #2d2a5e; border-color: #4338ca; color: #c7d2fe; }

.dialog-btn.confirm.danger {
  background: #450a0a;
  border-color: #7f1d1d;
  color: #fca5a5;
}
.dialog-btn.confirm.danger:hover { background: #5a0f0f; border-color: #dc2626; color: #fecaca; }

/* ── Animations ── */
.dialog-fade-enter-active { transition: opacity 0.15s, transform 0.15s; }
.dialog-fade-leave-active { transition: opacity 0.12s, transform 0.1s; }
.dialog-fade-enter-from { opacity: 0; transform: scale(0.95); }
.dialog-fade-leave-to { opacity: 0; transform: scale(0.97); }
</style>
