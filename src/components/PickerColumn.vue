<template>
  <div class="roller-col" :style="{ '--item-h': itemHeight + 'px' }">
    <div class="roller-viewport" ref="viewportRef" @wheel="onWheel" @pointerdown="onPointerDown" @pointerup="onPointerUp" @pointerleave="onPointerUp">
      <div class="roller-track">
        <div
          v-for="(label, index) in labels"
          :key="index"
          class="roller-item"
          :class="itemClass(index)"
          :style="itemStyle(index)"
        >
          {{ label }}
        </div>
      </div>
    </div>
    <span class="roller-unit" v-if="unit">{{ unit }}</span>
    <div class="picker-frame" aria-hidden="true" />
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, watch, onMounted, onBeforeUnmount, type PropType } from 'vue';

const props = defineProps({
  labels: { type: Array as PropType<string[]>, required: true },
  modelValue: { type: Number, required: true },
  itemHeight: { type: Number, default: 36 },
  unit: { type: String, default: '' },
});
const emit = defineEmits(['update:modelValue']);

const viewportRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
let snapTimer: ReturnType<typeof setTimeout> | null = null;
let animId: number | null = null;
let touching = false;

const total = computed(() => props.labels.length);
const maxScroll = computed(() => Math.max(0, (total.value - 1) * props.itemHeight));
const minScroll = 0;

const centerIndex = computed(() => {
  const raw = Math.round(scrollTop.value / props.itemHeight);
  return Math.max(0, Math.min(total.value - 1, raw));
});

function itemClass(index: number) {
  const c = centerIndex.value;
  const dist = Math.abs(index - c);
  if (dist === 0) return 'main';
  if (dist === 1) return 'near';
  return 'far';
}

function itemStyle(index: number) {
  const c = scrollTop.value / props.itemHeight;
  const dist = Math.abs(index - c);
  const opacity = Math.max(0.15, 1 - dist / 2.5);
  const scale = 1 - Math.min(0.15, dist * 0.07);
  return { opacity, transform: `scale(${scale})` };
}

function clamp(v: number): number {
  return Math.max(minScroll, Math.min(maxScroll.value, v));
}

function snap() {
  const el = viewportRef.value;
  if (!el) return;
  const target = centerIndex.value * props.itemHeight;
  if (Math.abs(el.scrollTop - target) > 1) {
    animateScrollTo(target);
  }
  const norm = centerIndex.value;
  if (norm !== props.modelValue) emit('update:modelValue', norm);
}

function animateScrollTo(target: number) {
  const el = viewportRef.value;
  if (!el) return;
  if (animId != null) cancelAnimationFrame(animId);
  target = clamp(target);
  const start = el.scrollTop;
  const delta = target - start;
  if (Math.abs(delta) < 1) return;
  const duration = 120;
  const t0 = performance.now();
  function step(now: number) {
    const t = Math.min(1, (now - t0) / duration);
    const ease = 0.5 - 0.5 * Math.cos(Math.PI * t);
    const el2 = viewportRef.value;
    if (!el2) { animId = null; return; }
    el2.scrollTop = start + delta * ease;
    scrollTop.value = el2.scrollTop;
    if (t < 1) { animId = requestAnimationFrame(step); }
    else { animId = null; el2.scrollTop = target; }
  }
  animId = requestAnimationFrame(step);
}

function onWheel(e: WheelEvent) {
  if (Math.abs(e.deltaY) < 10) return;
  e.preventDefault();
  const el = viewportRef.value;
  if (!el) return;
  const target = clamp(el.scrollTop + (e.deltaY > 0 ? props.itemHeight : -props.itemHeight));
  animateScrollTo(target);
  if (snapTimer) clearTimeout(snapTimer);
  snapTimer = setTimeout(snap, 80);
}

function onPointerDown() {
  touching = true;
  if (snapTimer) clearTimeout(snapTimer);
  if (animId != null) { cancelAnimationFrame(animId); animId = null; }
}

function onPointerUp() {
  touching = false;
  snap();
}

function onScroll() {
  const el = viewportRef.value;
  if (!el) return;
  scrollTop.value = el.scrollTop;
  if (!touching) {
    if (el.scrollTop < 0 || el.scrollTop > maxScroll.value) {
      el.scrollTop = clamp(el.scrollTop);
      scrollTop.value = el.scrollTop;
    }
    if (snapTimer) clearTimeout(snapTimer);
    snapTimer = setTimeout(snap, 120);
  }
}

onMounted(() => {
  const el = viewportRef.value;
  if (!el) return;
  el.scrollTop = clamp(props.modelValue * props.itemHeight);
  scrollTop.value = el.scrollTop;
  el.addEventListener('scroll', onScroll, { passive: true });
});

watch(total, () => {
  const el = viewportRef.value;
  if (!el) return;
  const v = Math.min(props.modelValue, total.value - 1);
  el.scrollTop = clamp(v * props.itemHeight);
  scrollTop.value = el.scrollTop;
});

watch(() => props.modelValue, (v) => {
  const el = viewportRef.value;
  if (!el) return;
  const target = clamp(v * props.itemHeight);
  if (Math.abs(el.scrollTop - target) > 1) {
    animateScrollTo(target);
  }
});

onBeforeUnmount(() => {
  if (snapTimer) clearTimeout(snapTimer);
  if (animId) cancelAnimationFrame(animId);
});
</script>

<style scoped>
.roller-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
  --item-h: 36px;
  height: calc(var(--item-h) * 3);
  overflow: hidden;
}
.roller-viewport {
  height: 100%;
  width: 100%;
  overflow-y: scroll;
  -webkit-overflow-scrolling: touch;
  overscroll-behavior: none;
  scrollbar-width: none;
  -ms-overflow-style: none;
  touch-action: pan-y;
}
.roller-viewport::-webkit-scrollbar {
  display: none;
}
.roller-track {
  padding: calc(var(--item-h) * 1) 0;
}
.roller-item {
  height: var(--item-h);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
  color: #aaa;
  box-sizing: border-box;
  line-height: 1;
}
.roller-item.main {
  font-size: 1.2rem;
  color: #409eff;
  font-weight: 700;
  opacity: 1;
}
.roller-item.near {
  font-size: 1rem;
  color: #6b9fdc;
}
.picker-frame {
  position: absolute;
  left: 0;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  height: var(--item-h);
  border-radius: 6px;
  pointer-events: none;
  box-shadow: inset 0 0 0 1px rgba(64, 158, 255, 0.12);
}
.roller-unit {
  font-size: 0.85rem;
  color: #888;
  margin-top: 2px;
  line-height: 1;
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
