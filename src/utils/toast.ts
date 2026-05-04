import { ref } from 'vue';

export type ToastType = 'success' | 'error' | 'info';

export const toast = ref<{ show: boolean; msg: string; type: ToastType }>({
  show: false,
  msg: '',
  type: 'info',
});

let timer: ReturnType<typeof setTimeout> | null = null;

export function showToast(msg: string, type: ToastType = 'info') {
  if (timer) clearTimeout(timer);
  toast.value = { show: true, msg, type };
  timer = setTimeout(() => {
    toast.value.show = false;
    timer = null;
  }, 2500);
}
