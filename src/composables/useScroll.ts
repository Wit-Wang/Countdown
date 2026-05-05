import { onMounted, onUnmounted } from 'vue';

const SCROLL_MULTIPLIER = 0.6;
const SMOOTH_FACTOR = 0.14;

export function useScroll(selector: string) {
  let wheelListener: ((e: WheelEvent) => void) | null = null;

  onMounted(() => {
    const el = document.querySelector(selector) as HTMLElement | null;
    if (!el) return;

    let isAnimating = false;
    let target = el.scrollTop;

    wheelListener = (e: WheelEvent) => {
      if (e.ctrlKey) return;
      e.preventDefault();
      target = Math.max(0, Math.min(el.scrollHeight - el.clientHeight, target + e.deltaY * SCROLL_MULTIPLIER));

      if (!isAnimating) {
        isAnimating = true;
        const step = () => {
          const current = el.scrollTop;
          const diff = target - current;
          if (Math.abs(diff) < 0.5) {
            el.scrollTop = target;
            isAnimating = false;
            return;
          }
          el.scrollTop = current + diff * SMOOTH_FACTOR;
          requestAnimationFrame(step);
        };
        requestAnimationFrame(step);
      }
    };

    el.addEventListener('wheel', wheelListener, { passive: false });
  });

  onUnmounted(() => {
    if (wheelListener) {
      const el = document.querySelector(selector) as HTMLElement | null;
      if (el) el.removeEventListener('wheel', wheelListener);
    }
  });
}
