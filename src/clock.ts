import { ref } from 'vue';

export const now = ref(Date.now());

if (typeof window !== 'undefined') {
  setInterval(() => {
    now.value = Date.now();
  }, 1000);
}
