<template>
  <div class="modal-mask center-modal" @click.self="close">
    <div class="modal-container center-modal">
      <div class="modal-header">
        <span>{{ isEditing ? '编辑待办事项' : '新建待办事项' }}</span>
        <div class="modal-actions">
          <button class="window-btn" @click="close">
            <span class="iconfont" style="font-family: 'Segoe MDL2 Assets', 'Segoe UI Symbol', 'Segoe UI', Arial, sans-serif;">&#xE8BB;</span>
          </button>
        </div>
      </div>
      <form class="modal-form" @submit.prevent="submit">
        <input v-model="text" placeholder="事项内容" class="modal-input">
        <div class="datetime-row">
          <PickerColumn :getLabel="getYearLabel" :applyDelta="d => applyDelta('year', d)" unit="年" class="col-year" />
          <PickerColumn :getLabel="getMonthLabel" :applyDelta="d => applyDelta('month', d)" unit="月" class="col-month" />
          <PickerColumn :getLabel="getDayLabel" :applyDelta="d => applyDelta('day', d)" unit="日" class="col-day" />
          <PickerColumn :getLabel="getHourLabel" :applyDelta="d => applyDelta('hour', d)" unit="时" class="col-hour" />
          <PickerColumn :getLabel="getMinuteLabel" :applyDelta="d => applyDelta('minute', d)" unit="分" class="col-minute" />
        </div>
        <div class="repeat-row">
          <label class="repeat-label">重复</label>
          <div style="flex: 1; display: flex; align-items: center">
            <PickerColumn :getLabel="getRepeatLabel" :applyDelta="applyRepeatDelta" class="col-repeat" />
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

const now = new Date();
const year = ref(now.getFullYear());
const month = ref(now.getMonth() + 1);
const day = ref(now.getDate());
const hour = ref(now.getHours());
const minute = ref((Math.round(now.getMinutes() / 5) * 5) % 60);

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

const repeatIndex = ref(Math.max(0, REPEAT_KEYS.indexOf(repeatKey.value)));
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

const deltaHandlers: Record<string, (delta: number) => void> = {
  year: d => { year.value = Math.max(2000, Math.min(2100, year.value + d)); },
  month: d => {
    month.value = ((month.value - 1 + d + 12) % 12) + 1;
    const maxDay = new Date(year.value, month.value, 0).getDate();
    if (day.value > maxDay) day.value = maxDay;
  },
  day: d => {
    const maxDay = new Date(year.value, month.value, 0).getDate();
    day.value = ((day.value - 1 + d + maxDay) % maxDay) + 1;
  },
  hour: d => { hour.value = (hour.value + d + 24) % 24; },
  minute: d => {
    let next = minute.value + d * 5;
    minute.value = next > 59 ? 0 : next < 0 ? 55 : next;
  },
};

function applyDelta(type: string, delta: number) {
  deltaHandlers[type]?.(delta);
}

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
  background: #ffffff;
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
.datetime-row {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 8px;
  justify-content: space-between;
  flex-wrap: nowrap;
}
</style>
