<template>
  <div class="modal-mask center-modal" @click.self="close">
    <div class="modal-container center-modal">
      <div class="modal-header">
        <span>{{ isEditing ? '编辑待办事项' : '新建待办事项' }}</span>
        <div class="modal-actions">
          <button class="window-btn" @click="close">
            <svg viewBox="0 0 24 24" width="18" height="18">
              <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" fill="none" stroke="#888" stroke-width="2"/>
            </svg>
          </button>
        </div>
      </div>
      <form class="modal-form" @submit.prevent="submit">
        <input v-model="text" placeholder="事项内容" class="modal-input" maxlength="200">
        <div class="datetime-row">
          <PickerColumn :labels="yearLabels" v-model="yearIndex" unit="年" class="col-year" />
          <PickerColumn :labels="monthLabels" v-model="monthIndex" unit="月" class="col-month" />
          <PickerColumn :labels="dayLabels" v-model="dayIndex" unit="日" class="col-day" />
          <PickerColumn :labels="hourLabels" v-model="hourIndex" unit="时" class="col-hour" />
          <PickerColumn :labels="minuteLabels" v-model="minuteIndex" unit="分" class="col-minute" />
        </div>
        <div class="repeat-row">
          <label class="repeat-label">重复</label>
          <div style="flex: 1; display: flex; align-items: center">
            <PickerColumn :labels="repeatLabels" v-model="repeatModelIndex" class="col-repeat" :itemHeight="28" />
          </div>
          <div class="auto-expire-btn" :class="{ active: autoExpire }" @click="autoExpire = !autoExpire">
            <span class="auto-expire-text">自动截止</span>
          </div>
        </div>
        <textarea v-model="info" class="info-textarea" placeholder="备注信息（可选）" rows="3" />
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
import { showToast } from '../utils/toast';

const props = defineProps<{
  initial?: { text?: string; deadline?: DateTime; repeat?: DateTime | null; autoExpire?: boolean; info?: string };
}>();
const emit = defineEmits(['submit', 'close']);
const text = ref('');
const info = ref('');
const autoExpire = ref(false);
const repeatKey = ref<string>('none');
const isEditing = computed(() => !!props.initial);

const now = new Date();
const year = ref(now.getFullYear());
const month = ref(now.getMonth() + 1);
const day = ref(now.getDate());
const hour = ref(now.getHours());
const minute = ref((Math.round(now.getMinutes() / 5) * 5) % 60);

const yearLabels = computed(() =>
  Array.from({ length: 101 }, (_, i) => String(2000 + i))
);
const monthLabels = computed(() =>
  Array.from({ length: 12 }, (_, i) => pad(i + 1))
);
const maxDay = computed(() => new Date(year.value, month.value, 0).getDate());
const dayLabels = computed(() =>
  Array.from({ length: maxDay.value }, (_, i) => pad(i + 1))
);
const hourLabels = computed(() =>
  Array.from({ length: 24 }, (_, i) => pad(i))
);
const minuteLabels = computed(() =>
  Array.from({ length: 12 }, (_, i) => pad(i * 5))
);
const repeatLabels = REPEAT_KEYS.map(k => REPEAT_OPTIONS[k].label);

const yearIndex = computed({
  get: () => year.value - 2000,
  set: v => { year.value = 2000 + v; },
});
const monthIndex = computed({
  get: () => month.value - 1,
  set: v => { month.value = v + 1; },
});
const dayIndex = computed({
  get: () => day.value - 1,
  set: v => { day.value = v + 1; },
});
const hourIndex = computed({
  get: () => hour.value,
  set: v => { hour.value = v; },
});
const minuteIndex = computed({
  get: () => minute.value / 5,
  set: v => { minute.value = v * 5; },
});
const repeatModelIndex = computed({
  get: () => Math.max(0, REPEAT_KEYS.indexOf(repeatKey.value)),
  set: v => { repeatKey.value = REPEAT_KEYS[v]; },
});

watch(
  () => props.initial,
  v => {
    if (v) {
      text.value = v.text || '';
      info.value = v.info ?? '';
      autoExpire.value = v.autoExpire ?? false;
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
      info.value = '';
      autoExpire.value = false;
      year.value = n.getFullYear();
      month.value = n.getMonth() + 1;
      day.value = n.getDate();
      hour.value = n.getHours();
      minute.value = (Math.round(n.getMinutes() / 5) * 5) % 60;
      repeatKey.value = 'none';
    }
  },
  { immediate: true }
);

function keyForInterval(interval?: DateTime | null) {
  if (!interval) return 'none';
  for (const k of REPEAT_KEYS) {
    const iv = REPEAT_OPTIONS[k].interval;
    if (iv && dateTimeEqual(iv, interval)) return String(k);
  }
  return 'none';
}

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

function pad(n: number | string): string {
  n = Number(n);
  return n < 10 ? '0' + n : String(n);
}

watch([year, month], () => {
  const maxDay = new Date(year.value, month.value, 0).getDate();
  if (day.value > maxDay) day.value = maxDay;
});

function submit() {
  text.value = text.value.trim();
  if (!text.value) { showToast('请输入事项内容', 'error'); return; }
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
    autoExpire: autoExpire.value,
    info: info.value || undefined,
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
.auto-expire-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 6px 14px 8px;
  border-radius: 10px;
  background: #f0f2f5;
  color: #888;
  cursor: pointer;
  user-select: none;
  transition: background 0.2s, color 0.2s;
  white-space: nowrap;
  min-width: 48px;
}
.auto-expire-btn.active {
  background: #409eff;
  color: #fff;
}
.auto-expire-text {
  font-size: 11px;
  line-height: 1.2;
}
.info-textarea {
  border: 1px solid #dcdfe6;
  border-radius: 6px;
  padding: 10px 14px;
  font-size: 14px;
  background: #ffffff;
  color: #222;
  resize: none;
  min-height: 60px;
  font-family: inherit;
  line-height: 1.5;
}
.info-textarea:focus {
  outline: none;
  border-color: #409eff;
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
