<template>
  <div class="roller-col" @wheel.prevent="onWheel" @pointerdown="onPointerDown">
    <div class="roller-items" :style="{ transform: `translateY(${offset}px)` }">
      <label
        v-for="rel in rels"
        :key="rel"
        class="roller-label roller-item"
        :class="itemClass(rel)"
        :style="itemStyle(rel)"
      >
        {{ getLabel(rel) }}
      </label>
    </div>
    <span class="roller-unit" v-if="unit">{{ unit }}</span>
    <div class="picker-frame" aria-hidden="true" />
  </div>
</template>

<script lang="ts" setup>
import { ref, onBeforeUnmount, type PropType } from 'vue';

const props = defineProps({
  getLabel: { type: Function as PropType<(rel: number) => string>, required: true },
  applyDelta: { type: Function as PropType<(delta: number) => void>, required: true },
  itemHeight: { type: Number, default: 36 },
  sensitivity: { type: Number, default: 0.15 },
  unit: { type: String, default: '' },
});

const rels = [-2, -1, 0, 1, 2];
const offset = ref(0);
const velocity = ref(0);
const VEL_MAX = 10; // cap max velocity from wheel
let rafId: number | null = null;
let dragging = false;
let startY = 0;
let lastY = 0;
let wheelTimer: number | null = null;
let wheelActive = false;
let lastWheelApply = 0;

function ensureAnimating() {
  if (rafId != null) return;

  const VEL_THRESHOLD = 0.5; // when below this and not dragging, start snap

  const SNAP_MS = 180;

  function animateTo(target: number, onDone: () => void) {
    const start = offset.value;
    const delta = target - start;
    const t0 = performance.now();

    function snapStep(now: number) {
      const t = Math.min(1, (now - t0) / SNAP_MS);
      const ease = 0.5 - 0.5 * Math.cos(Math.PI * t); // easeInOut
      offset.value = start + delta * ease;
      if (t < 1) {
        rafId = requestAnimationFrame(snapStep);
      } else {
        rafId = null;
        onDone();
      }
    }

    rafId = requestAnimationFrame(snapStep);
  }

  function step() {
    if (!dragging) {
      // when velocity small, snap to nearest item
      if (Math.abs(velocity.value) < VEL_THRESHOLD) {
        const nearest = Math.round(offset.value / props.itemHeight);
        if (nearest === 0) {
          // just snap to center
          animateTo(0, () => {
            offset.value = 0;
            velocity.value = 0;
            rafId = null;
          });
          return;
        } else {
          const target = nearest * props.itemHeight;
          animateTo(target, () => {
            // apply delta to shift logical value, then adjust offset back into range
            props.applyDelta(-nearest);
            offset.value += -nearest * props.itemHeight;
            velocity.value = 0;
            rafId = null;
          });
          return;
        }
      }
      velocity.value *= 0.92;
    }

    offset.value += velocity.value;
    while (offset.value <= -props.itemHeight) {
      const now = performance.now();
      if (wheelActive && now - lastWheelApply < 120) break;
      props.applyDelta(1);
      lastWheelApply = now;
      offset.value += props.itemHeight;
    }
    while (offset.value >= props.itemHeight) {
      const now = performance.now();
      if (wheelActive && now - lastWheelApply < 120) break;
      props.applyDelta(-1);
      lastWheelApply = now;
      offset.value -= props.itemHeight;
    }

    rafId = requestAnimationFrame(step);
  }

  rafId = requestAnimationFrame(step);
}

function onWheel(e: WheelEvent) {
  // Normalize wheel and avoid firing multiple logical steps per physical scroll.
  const sign = e.deltaY > 0 ? -1 : 1;
  wheelActive = true;

  // If rapid wheel events come in, only increase velocity slightly.
  const add =
    sign * props.itemHeight * props.sensitivity * Math.min(1.0, Math.abs(e.deltaY) / 100) * 0.5;
  velocity.value += add;
  if (velocity.value >= VEL_MAX) velocity.value = VEL_MAX;
  if (velocity.value <= -VEL_MAX) velocity.value = -VEL_MAX;
  ensureAnimating();

  if (wheelTimer) window.clearTimeout(wheelTimer);
  wheelTimer = window.setTimeout(() => {
    wheelActive = false;
    wheelTimer = null;
  }, 140);
}

function onPointerDown(e: PointerEvent) {
  const el = (e.currentTarget as HTMLElement) || null;
  try {
    if (el) el.setPointerCapture(e.pointerId);
  } catch {
    // ignore
  }

  dragging = true;
  startY = e.clientY;
  lastY = e.clientY;
  velocity.value = 0;

  function onMove(ev: PointerEvent) {
    const dy = ev.clientY - lastY;
    lastY = ev.clientY;
    offset.value += dy;

    while (offset.value <= -props.itemHeight) {
      props.applyDelta(1);
      offset.value += props.itemHeight;
    }
    while (offset.value >= props.itemHeight) {
      props.applyDelta(-1);
      offset.value -= props.itemHeight;
    }
  }

  function onUp(ev: PointerEvent) {
    const total = ev.clientY - startY;
    velocity.value = total * 0.25;
    dragging = false;

    try {
      if (el) el.releasePointerCapture(e.pointerId);
    } catch {
      // ignore
    }

    window.removeEventListener('pointermove', onMove as any);
    window.removeEventListener('pointerup', onUp as any);

    ensureAnimating();
  }

  window.addEventListener('pointermove', onMove as any);
  window.addEventListener('pointerup', onUp as any);
}

function itemStyle(rel: number) {
  const posPx = rel * props.itemHeight + offset.value;
  const dist = Math.abs(posPx) / props.itemHeight;
  let opacity = 1 - dist / 2;
  opacity = Math.max(0, Math.min(1, opacity));
  const scale = 1 - Math.min(0.12, dist * 0.06);
  return {
    opacity: String(opacity),
    transform: `scale(${scale})`,
  } as Record<string, string>;
}

function itemClass(rel: number) {
  const posPx = rel * props.itemHeight + offset.value;
  const dist = Math.abs(posPx) / props.itemHeight;
  if (dist < 0.5) return 'main';
  if (dist < 1.5) return 'near';
  return 'far';
}

onBeforeUnmount(() => {
  if (rafId) cancelAnimationFrame(rafId);
});
</script>

<style scoped>
.roller-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  box-sizing: border-box;
  margin: 0 6px;
  position: relative;
  --item-h: 36px;
  height: calc(var(--item-h) * 5);
  overflow: hidden;
}
.roller-items {
  display: flex;
  flex-direction: column;
  align-items: center;
  transition: none;
  will-change: transform;
}
.roller-item {
  display: block;
  width: 100%;
  text-align: center;
  padding: 6px 4px;
  box-sizing: border-box;
  transition:
    transform 160ms ease,
    opacity 160ms ease;
  height: var(--item-h);
  line-height: var(--item-h);
}
.roller-item.main {
  font-size: 1.2rem;
  color: #409eff;
  font-weight: 700;
  background: #f5f7fa;
  border-radius: 6px;
  opacity: 1;
}
.roller-item.near {
  font-size: 1rem;
  color: #6b9fdc;
}
.roller-item.far {
  font-size: 0.95rem;
  color: #aaa;
}
.picker-frame {
  position: absolute;
  left: 6px;
  right: 6px;
  top: 50%;
  transform: translateY(-50%);
  height: var(--item-h);
  border-radius: 6px;
  pointer-events: none;
  box-shadow: inset 0 0 0 1px rgba(64, 158, 255, 0.08);
}
.roller-unit {
  font-size: 0.9rem;
  color: #888;
  margin-top: 2px;
}
.col-year {
  flex: 0 0 20%;
  max-width: 20%;
}
.col-month,
.col-day,
.col-hour,
.col-minute {
  flex: 0 0 10%;
  max-width: 10%;
}
.col-repeat {
  flex: 1 1 auto;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
