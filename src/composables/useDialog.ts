import { reactive } from 'vue'

type DialogType = 'confirm' | 'alert' | 'danger'

interface DialogState {
  show: boolean
  type: DialogType
  title: string
  message: string
  confirmLabel: string
  cancelLabel: string
  resolve: ((val: boolean) => void) | null
}

const state = reactive<DialogState>({
  show: false,
  type: 'alert',
  title: '',
  message: '',
  confirmLabel: 'OK',
  cancelLabel: 'Cancel',
  resolve: null,
})

export function useDialog() {
  const openConfirm = (
    title: string,
    message: string,
    opts?: { type?: DialogType; confirmLabel?: string; cancelLabel?: string }
  ): Promise<boolean> => {
    return new Promise((res) => {
      state.show = true
      state.type = opts?.type ?? 'confirm'
      state.title = title
      state.message = message
      state.confirmLabel = opts?.confirmLabel ?? 'Confirm'
      state.cancelLabel = opts?.cancelLabel ?? 'Cancel'
      state.resolve = res
    })
  }

  const openAlert = (title: string, message: string): Promise<void> => {
    return new Promise<boolean>((res) => {
      state.show = true
      state.type = 'alert'
      state.title = title
      state.message = message
      state.confirmLabel = 'OK'
      state.cancelLabel = ''
      state.resolve = res
    }).then(() => {})
  }

  const dismiss = (val: boolean) => {
    state.show = false
    state.resolve?.(val)
    state.resolve = null
  }

  return { state, openConfirm, openAlert, dismiss }
}
