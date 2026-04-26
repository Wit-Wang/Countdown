<template>
  <div class="modal-mask center-modal" @click.self="close">
    <div class="modal-container center-modal">
      <div class="modal-header">
        <span>{{ isEditing ? '编辑待办事项' : '新建待办事项' }}</span>
        <div class="modal-actions">
          <button class="window-btn" @click="close">
            <span
              class="iconfont"
              style="
                font-family: 'Segoe MDL2 Assets', 'Segoe UI Symbol', 'Segoe UI', Arial, sans-serif;
              "
              >&#xE8BB;</span
            >
          </button>
        </div>
      </div>
      <form class="modal-form" @submit.prevent="submit">
        <input v-model="text" placeholder="事项内容" class="modal-input" />
        <div class="datetime-row">
          <PickerColumn
            :getLabel="getYearLabel"
            :applyDelta="d => applyDelta('year', d)"
            unit="年"
            class="col-year"
          />

          <PickerColumn
            :getLabel="getMonthLabel"
            :applyDelta="d => applyDelta('month', d)"
            unit="月"
            class="col-month"
          />

          <PickerColumn
            :getLabel="getDayLabel"
            :applyDelta="d => applyDelta('day', d)"
            unit="日"
            class="col-day"
          />

          <PickerColumn
            :getLabel="getHourLabel"
            :applyDelta="d => applyDelta('hour', d)"
            unit="时"
            class="col-hour"
          />

          <PickerColumn
            :getLabel="getMinuteLabel"
            :applyDelta="d => applyDelta('minute', d)"
            unit="分"
            class="col-minute"
          />
        </div>
        <div class="repeat-row">
          <label class="repeat-label">重复</label>
          <div style="flex: 1; display: flex; align-items: center">
            <PickerColumn
              :getLabel="getRepeatLabel"
              :applyDelta="applyRepeatDelta"
              class="col-repeat"
            />
          </div>
        </div>
        <button class="main-btn" type="submit" :aria-label="isEditing ? '保存' : '创建'">
          {{ isEditing ? '保存' : '创建' }}
        </button>
      </form>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed, watch } from 'vue';
import PickerColumn from './PickerColumn.vue';
import type { DateTime } from '../types/interface';
import { REPEAT_OPTIONS, REPEAT_KEYS } from '../constants/repeatOptions';

const props = defineProps<{
  initial?: { text?: string; deadline?: DateTime; repeat?: DateTime | null };
}>();
const emit = defineEmits(['submit', 'close']);
const text = ref('');
const repeatKey = ref<string>('none');
const isEditing = computed(() => !!props.initial);

function dateTimeEqual(a?: DateTime | null, b?: DateTime | null) {
  if (!a && !b) return true;
  if (!a || !b) return false;
  return (
    a.year === b.year &&
    a.month === b.month &&
    a.day === b.day &&
    a.hour === b.hour &&
    a.minute === b.minute &&
    a.second === b.second
  );
}

function keyForInterval(interval?: DateTime | null) {
  if (!interval) return 'none';
  for (const k of REPEAT_KEYS) {
    const iv = REPEAT_OPTIONS[k].interval;
    if (iv && dateTimeEqual(iv, interval)) return String(k);
  }
  return 'none';
}
// 日期时间相关

const now = new Date();
const year = ref(now.getFullYear());
const month = ref(now.getMonth() + 1);
const day = ref(now.getDate());
const hour = ref(now.getHours());
// 分钟默认吸附到最近5分钟
const minute = ref((Math.round(now.getMinutes() / 5) * 5) % 60);

// 当作为编辑打开时，使用传入的初始值覆盖默认
watch(
  () => props.initial,
  v => {
    if (v) {
      text.value = v.text || '';
      repeatKey.value = keyForInterval(v.repeat ?? null);
      if (v.deadline) {
        const dt = v.deadline as DateTime;
        year.value = dt.year ?? year.value;
        month.value = dt.month ?? month.value;
        day.value = dt.day ?? day.value;
        hour.value = dt.hour ?? hour.value;
        minute.value = (Math.round((dt.minute ?? minute.value) / 5) * 5) % 60;
      }
    } else {
      const n = new Date();
      text.value = '';
      year.value = n.getFullYear();
      month.value = n.getMonth() + 1;
      day.value = n.getDate();
      hour.value = n.getHours();
      minute.value = (Math.round(n.getMinutes() / 5) * 5) % 60;
    }
  },
  { immediate: true }
);

function pad(n: number | string): string {
  n = Number(n);
  return n < 10 ? '0' + n : String(n);
}

// 旧的 prev/next 计算已由 PickerColumn 内部动态计算，不再需要单独的 computed 值

// 每列显示文本获取函数（相对索引）
const getYearLabel = (rel: number): string => {
  const v = Math.max(2000, Math.min(2100, year.value + rel));
  return String(v);
};
const getMonthLabel = (rel: number): string => {
  const m = ((month.value - 1 + rel + 1200) % 12) + 1;
  return pad(m);
};
const getDayLabel = (rel: number): string => {
  const d = new Date(year.value, month.value - 1, day.value + rel).getDate();
  return pad(d);
};
const getHourLabel = (rel: number): string => {
  const h = (hour.value + rel + 24) % 24;
  return pad(h);
};
const getMinuteLabel = (rel: number): string => {
  let m = minute.value + rel * 5;
  m = ((m % 60) + 60) % 60;
  return pad(m);
};

// repeat picker state (use PickerColumn)
const repeatIndex = ref(Math.max(0, REPEAT_KEYS.indexOf(repeatKey.value)));
// keep repeatKey and repeatIndex in sync
watch(repeatKey, v => {
  const idx = REPEAT_KEYS.indexOf(v);
  if (idx >= 0) repeatIndex.value = idx;
});
watch(repeatIndex, i => {
  const n = REPEAT_KEYS.length;
  repeatKey.value = REPEAT_KEYS[((i % n) + n) % n];
});

const getRepeatLabel = (rel: number): string => {
  const n = REPEAT_KEYS.length;
  const idx = (repeatIndex.value + rel + n) % n;
  const key = REPEAT_KEYS[idx];
  return REPEAT_OPTIONS[key].label;
};

const applyRepeatDelta = (delta: number) => {
  const n = REPEAT_KEYS.length;
  repeatIndex.value = (((repeatIndex.value + delta) % n) + n) % n;
  repeatKey.value = REPEAT_KEYS[repeatIndex.value];
};

function applyDelta(type: string, delta: number) {
  if (type === 'year') {
    year.value = Math.max(2000, Math.min(2100, year.value + delta));
  } else if (type === 'month') {
    month.value = ((month.value - 1 + delta + 12) % 12) + 1;
    const maxDay = new Date(year.value, month.value, 0).getDate();
    if (day.value > maxDay) day.value = maxDay;
  } else if (type === 'day') {
    const maxDay = new Date(year.value, month.value, 0).getDate();
    day.value = ((day.value - 1 + delta + maxDay) % maxDay) + 1;
  } else if (type === 'hour') {
    hour.value = (hour.value + delta + 24) % 24;
  } else if (type === 'minute') {
    let next = minute.value + delta * 5;
    if (next > 59) next = 0;
    if (next < 0) next = 55;
    minute.value = next;
  }
}

// 2. 月份变更时自动修正天数不溢出
watch([year, month], () => {
  const maxDay = new Date(year.value, month.value, 0).getDate();
  if (day.value > maxDay) day.value = maxDay;
});
function submit() {
  if (!text.value) return;
  emit('submit', {
    text: text.value,
    deadline: {
      year: year.value,
      month: month.value,
      day: day.value,
      hour: hour.value,
      minute: minute.value,
      second: 0,
    } as DateTime,
    repeat: repeatKey.value === 'none' ? null : REPEAT_OPTIONS[repeatKey.value].interval,
  });
  text.value = '';
}
function close() {
  emit('close');
}
</script>

<style scoped>
/* Shared modal layout rules were moved to src/styles/common.css */
.modal-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
}
.modal-input {
  border: 1px solid #dcdfe6;
  border-radius: 6px;
  padding: 12px 14px;
  font-size: 18px;
  margin-bottom: 8px;
  background: #ffffff; /* override system dark-mode input backgrounds */
  color: #222;
}
.repeat-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.repeat-label {
  color: #666;
  font-size: 14px;
  width: 56px;
}
.modal-select {
  flex: 1;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid #dcdfe6;
  background: #fff;
  font-size: 14px;
}
.datetime-row {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 8px;
  justify-content: space-between;
  flex-wrap: nowrap; /* force single-line layout */
}
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
/* Percentage widths per request: year 20%, month/day/hour/minute each 10% */
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
.roller-label {
  display: block;
  width: 100%;
  text-align: center;
  user-select: none;
  transition:
    font-size 0.15s,
    color 0.15s;
}
.roller-label.main {
  font-size: 1.2rem; /* 调小主要数字字号 */
  color: #409eff;
  font-weight: 700;
  background: #f5f7fa;
  border-radius: 6px;
  margin: 2px 0;
  cursor: pointer;
  letter-spacing: 1px;
  padding: 6px 4px;
}
.roller-label.small {
  font-size: 0.85rem;
  color: #999;
  margin: 2px 0;
}
.roller-unit {
  font-size: 0.85rem;
  color: #888;
  margin-top: 2px;
}
.roller-label.small {
  font-size: 1rem;
  color: #bbb;
  margin: 2px 0;
}
.roller-unit {
  font-size: 0.9rem;
  color: #888;
  margin-top: 2px;
}
.col-repeat {
  flex: 1 1 auto;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* roller list: five visible items, center emphasized, sides透明度递减，滚动时平滑位移 */
.roller-items {
  display: flex;
  flex-direction: column;
  align-items: center;
  transition: none; /* transform controlled by JS for smooth continuous animation */
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
  opacity: 0.75;
}
.roller-item.far {
  font-size: 0.95rem;
  color: #aaa;
  opacity: 0.45;
}

/* center frame overlay */
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

/* far-item opacity handled dynamically via inline styles */
</style>
