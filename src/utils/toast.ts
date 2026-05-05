import { ref } from 'vue';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export const toast = ref<{ show: boolean; msg: string; type: ToastType }>({
  show: false,
  msg: '',
  type: 'info',
});

let timer: ReturnType<typeof setTimeout> | null = null;

export function showToast(msg: string, type: ToastType = 'info', duration = 2500) {
  if (timer) clearTimeout(timer);
  toast.value = { show: true, msg, type };
  if (duration > 0) {
    timer = setTimeout(() => {
      toast.value.show = false;
      timer = null;
    }, duration);
  }
}

export function hideToast() {
  if (timer) clearTimeout(timer);
  toast.value.show = false;
  timer = null;
}
