import { ref } from 'vue';

export const now = ref(Date.now());

let intervalId: ReturnType<typeof setInterval> | null = null;

function startClock() {
  if (intervalId != null) return;
  now.value = Date.now();
  intervalId = setInterval(() => {
    now.value = Date.now();
  }, 1000);
}

function stopClock() {
  if (intervalId != null) {
    clearInterval(intervalId);
    intervalId = null;
  }
}

if (typeof window !== 'undefined') {
  startClock();
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) stopClock();
    else startClock();
  });
}
