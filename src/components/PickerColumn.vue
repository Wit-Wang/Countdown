<template>
  <div class="roller-col" :style="{ '--item-h': itemHeight + 'px' }">
    <div class="roller-viewport" ref="viewportRef" @wheel="onWheel">
      <div class="roller-track">
        <div
          v-for="(label, index) in tripleLabels"
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
let scrollTimer: ReturnType<typeof setTimeout> | null = null;
let animId: number | null = null;
let touching = false;
let lastWheelTs = 0;

const total = computed(() => props.labels.length);
const tripleLabels = computed(() => [...props.labels, ...props.labels, ...props.labels]);

function itemClass(index: number) {
  const center = scrollTop.value / props.itemHeight;
  const dist = Math.abs(index - center);
  if (dist < 0.5) return 'main';
  if (dist < 1.5) return 'near';
  return 'far';
}

function itemStyle(index: number) {
  const center = scrollTop.value / props.itemHeight;
  const dist = Math.abs(index - center);
  const opacity = Math.max(0.15, 1 - dist / 2.5);
  const scale = 1 - Math.min(0.15, dist * 0.07);
  return {
    opacity,
    transform: `scale(${scale})`,
  };
}

function animateScrollTo(target: number) {
  const el = viewportRef.value;
  if (!el) return;
  if (animId != null) cancelAnimationFrame(animId);
  const start = el.scrollTop;
  const delta = target - start;
  if (Math.abs(delta) < 1) { el.scrollTop = target; return; }
  const duration = performance.now() - lastWheelTs < 150 ? 40 : 100;
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

function doTeleport(raw: number) {
  const el = viewportRef.value;
  if (!el) return;
  const norm = ((raw % total.value) + total.value) % total.value;
  el.scrollTop = (norm + total.value) * props.itemHeight;
  scrollTop.value = el.scrollTop;
  if (norm !== props.modelValue) emit('update:modelValue', norm);
}

function snapOne(dir: 1 | -1) {
  const el = viewportRef.value;
  if (!el) return;
  const raw = Math.round(el.scrollTop / props.itemHeight);
  const next = raw + dir;
  const t = total.value;
  const norm = ((next % t) + t) % t;
  if (next < 0 || next >= t * 3) {
    el.scrollTop = (norm + t) * props.itemHeight;
    scrollTop.value = el.scrollTop;
  } else {
    animateScrollTo(next * props.itemHeight);
  }
  if (norm !== props.modelValue) emit('update:modelValue', norm);
}

function onWheel(e: WheelEvent) {
  if (Math.abs(e.deltaY) < 10) return;
  e.preventDefault();
  lastWheelTs = performance.now();
  snapOne(e.deltaY > 0 ? 1 : -1);
}

function onScroll() {
  const el = viewportRef.value;
  if (!el) return;
  scrollTop.value = el.scrollTop;
  const raw = Math.round(el.scrollTop / props.itemHeight);
  const t = total.value;
  if (!touching && (raw < 2 || raw >= t * 3 - 2)) {
    doTeleport(raw);
    return;
  }
  if (scrollTimer) clearTimeout(scrollTimer);
  scrollTimer = setTimeout(() => {
    if (animId != null) { scrollTimer = null; return; }
    const raw2 = Math.round(el.scrollTop / props.itemHeight);
    const snapTarget = raw2 * props.itemHeight;
    const t2 = total.value;
    const norm = ((raw2 % t2) + t2) % t2;
    if (raw2 < 2 || raw2 >= t2 * 3 - 2) {
      doTeleport(raw2);
    } else if (Math.abs(el.scrollTop - snapTarget) > 2) {
      animateScrollTo(snapTarget);
      scrollTimer = null;
      return;
    }
    if (norm !== props.modelValue) emit('update:modelValue', norm);
    scrollTimer = null;
  }, 80);
}

onMounted(() => {
  const el = viewportRef.value;
  if (!el) return;
  el.scrollTop = (props.modelValue + total.value) * props.itemHeight;
  scrollTop.value = el.scrollTop;
  el.addEventListener('scroll', onScroll, { passive: true });
  el.addEventListener('pointerdown', () => { touching = true; });
  el.addEventListener('pointerup', () => { touching = false; });
});

watch(total, () => {
  const el = viewportRef.value;
  if (!el) return;
  const v = Math.min(props.modelValue, total.value - 1);
  el.scrollTop = (v + total.value) * props.itemHeight;
  scrollTop.value = el.scrollTop;
});

watch(() => props.modelValue, (v) => {
  const el = viewportRef.value;
  if (!el) return;
  const target = (v + total.value) * props.itemHeight;
  if (Math.abs(el.scrollTop - target) > 1) {
    animateScrollTo(target);
  }
});

onBeforeUnmount(() => {
  if (scrollTimer) clearTimeout(scrollTimer);
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
  height: calc(var(--item-h) * 5);
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
  padding: calc(var(--item-h) * 2) 0;
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
