<template>
  <div class="todo-card">
    <!-- 左侧文本区 -->
    <div class="left-section">
      <!-- 状态标签 -->
      <div class="urgency-label" :style="{ background: urgency.color, color: urgency.textColor }">
        {{ urgency.label }}
      </div>
      <!-- 主标题 -->
      <div class="main-title">{{ todo.text }}</div>
      <!-- 副标题区 -->
      <div class="subtitle-area">
        <div class="subtitle">截止时间 | {{ deadlineStr }}</div>
      </div>
    </div>
    <!-- 右侧信息区 -->
    <div class="right-section">
      <div class="time-info">
        <div class="main-time" :style="{ color: urgency.color }">{{ timeMain }}</div>
      </div>
      <div class="action-btns">
        <button class="icon-btn" title="完成" @click.stop="onComplete">
          <svg viewBox="0 0 24 24" width="16" height="16">
            <path d="M20 6L9 17l-5-5" fill="none" stroke="#888" stroke-width="2" />
          </svg>
        </button>
        <button class="icon-btn" title="编辑" @click.stop="onEdit">
          <svg viewBox="0 0 24 24" width="16" height="16">
            <path
              d="M3 17.25V21h3.75l11.06-11.06-3.75-3.75L3 17.25zM20.71 7.04a1.003 1.003 0 0 0 0-1.42l-2.34-2.34a1.003 1.003 0 0 0-1.42 0l-1.83 1.83 3.75 3.75 1.84-1.82z"
              fill="none"
              stroke="#888"
              stroke-width="2"
            />
          </svg>
        </button>
        <button class="icon-btn" title="删除" @click.stop="onRemove">
          <svg viewBox="0 0 24 24" width="16" height="16">
            <path
              d="M6 19a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"
              fill="none"
              stroke="#888"
              stroke-width="2"
            />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from 'vue';
import { now } from '../clock';
import type { Todo, DateTime } from '../types/interface';
import { URGENCY } from '../types/interface';
import { dateTimeToTs, formatDateTime } from '../utils/datetime';

const props = defineProps<{ todo: Todo }>();
const emit = defineEmits(['remove', 'edit', 'complete']);

function onRemove() {
  emit('remove', props.todo.id);
}
function onEdit() {
  emit('edit', props.todo);
}
function onComplete() {
  emit('complete', props.todo.id);
}

function getUrgency(deadline: DateTime) {
  const diff = dateTimeToTs(deadline) - now.value;
  for (const [start, end, label, color] of URGENCY) {
    if (diff >= Number(start) && diff < Number(end)) {
      return {
        label: label as string,
        color: color as string,
        textColor: label === '已到期' ? '#fff' : label === '即将' ? '#7c4a00' : '#222',
        type: label,
      };
    }
  }
  return { label: '未知', color: '#ccc', textColor: '#222', type: '未知' };
}

function formatCountdown(ms: number) {
  if (ms <= 0) return '0s';
  const d = Math.floor(ms / (24 * 60 * 60 * 1000));
  const h = Math.floor((ms % (24 * 60 * 60 * 1000)) / (60 * 60 * 1000));
  const m = Math.floor((ms % (60 * 60 * 1000)) / (60 * 1000));

  if (d >= 10) {
    return `${d}d ${h}h`;
  } else if (d <= 0) {
    return `${h}h ${m}m`;
  } else if (d <= 0 && h <= 0) {
    return `${m}m`;
  }
  return `${d}d ${h}h ${m}m`;
}

function formatDate(dt: DateTime) {
  return formatDateTime(dt);
}

const urgency = computed(() => getUrgency(props.todo.deadline));
const deadlineStr = computed(() => formatDate(props.todo.deadline));

const timeMain = computed(() => {
  const diff = dateTimeToTs(props.todo.deadline) - now.value;
  if (urgency.value.type === '已到期') {
    return formatCountdown(-diff);
  } else {
    return formatCountdown(diff);
  }
});
</script>

<style scoped>
.todo-card {
  background: rgba(255, 255, 255, 0.92);
  border-radius: 12px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.07);
  margin: 16px 0;
  padding: 16px 20px;
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: flex-start;
  position: relative;
}
.left-section {
  flex: 7;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}
.urgency-label {
  display: inline-block;
  border-radius: 10px;
  padding: 2px 12px;
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 8px;
  min-height: 22px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);
  letter-spacing: 1px;
  overflow: hidden;
}
.main-title {
  font-size: 18px;
  color: #222;
  font-weight: bold;
  margin-bottom: 6px;
  max-width: 100%;
  word-break: break-all;
  line-height: 1.3;
  display: -webkit-box;
  line-clamp: 2;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.subtitle-area {
  margin-bottom: 12px;
}
.subtitle {
  /* Allow the deadline text to use the full available width and wrap if needed
     so the date won't be truncated on narrow viewports. */
  width: 100%;
  font-size: 12px;
  color: #888;
  line-height: 1.2;
  white-space: normal;
  overflow: visible;
  text-overflow: unset;
  word-break: normal;
}
.right-section {
  flex: 3;
  min-width: 120px;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: flex-start;
  margin-left: 16px;
}
.time-info {
  text-align: right;
  margin-bottom: 8px;
  width: 100%;
}
.main-time {
  font-size: 20px;
  font-weight: bold;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.action-btns {
  display: flex;
  flex-direction: row;
  gap: 8px;
}
.icon-btn {
  width: 24px;
  height: 24px;
  border: none;
  background: none;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 50%;
  transition: background 0.2s;
}
.icon-btn:hover svg path {
  stroke: #409eff;
}
.icon-btn:active {
  background: #f0f0f0;
}
</style>
