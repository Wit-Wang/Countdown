<script lang="ts" setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import type { Todo, DateTime } from './types/interface';
import { dateTimeToTs, addIntervalToDateTime } from './utils/datetime';
import { toast, showToast } from './utils/toast';
import TodoItem from './components/TodoItem.vue';
import TodoModal from './components/TodoModal.vue';
import { invoke } from '@tauri-apps/api/core';

const todos = ref<Todo[]>([]);
const showTodoModal = ref(false);
const editingTodo = ref<Todo | null>(null);
const isMobile = ref(false);
const isSyncing = ref(false);

function uuidv4(): string {
  const b = new Uint8Array(16);
  crypto.getRandomValues(b);
  b[6] = (b[6] & 0x0f) | 0x40;
  b[8] = (b[8] & 0x3f) | 0x80;
  const h = Array.from(b, x => x.toString(16).padStart(2, '0')).join('');
  return `${h.slice(0,8)}-${h.slice(8,12)}-${h.slice(12,16)}-${h.slice(16,20)}-${h.slice(20)}`;
}

const sortedTodos = computed(() =>
  todos.value.slice().sort((a, b) => dateTimeToTs(a.deadline) - dateTimeToTs(b.deadline))
);

async function loadTodos() {
  try {
    const res = await invoke<Todo[]>('get_todos');
    todos.value = res || [];
  } catch (e) {
    console.error('get_todos', e);
  }
}

async function handleSync() {
  if (isSyncing.value) return;
  isSyncing.value = true;
  try {
    const res = await invoke<Todo[]>('sync_from_cloud');
    todos.value = res || [];
    showToast('同步成功', 'success');
  } catch (e) {
    showToast('同步失败：' + (typeof e === 'string' ? e : '网络错误'), 'error');
  } finally {
    isSyncing.value = false;
  }
}

async function commit(command: string, args: Record<string, unknown>, optimistic: () => void) {
  try {
    await invoke(command, args);
    await loadTodos();
  } catch (e) {
    showToast('操作失败：' + (typeof e === 'string' ? e : '请重试'), 'error');
    optimistic();
  }
}

function openTodoModal() {
  editingTodo.value = null;
  showTodoModal.value = true;
}
function closeTodoModal() {
  showTodoModal.value = false;
  editingTodo.value = null;
}
async function handleAddTodo({ text, deadline, repeat, autoExpire, info }: { text: string; deadline: DateTime; repeat?: DateTime | null; autoExpire?: boolean; info?: string }) {
  if (editingTodo.value) {
    const updated: Todo = {
      id: editingTodo.value.id,
      text,
      deadline,
      repeat: repeat ?? null,
      autoExpire: autoExpire ?? false,
      info: info || undefined,
      completed: editingTodo.value.completed,
    };
    await commit('update_todo', { updated }, () => {
      todos.value = todos.value.map(t => (t.id === updated.id ? updated : t));
    });
    editingTodo.value = null;
  } else {
    const todo: Todo = { id: uuidv4(), text, deadline, repeat: repeat ?? null, autoExpire: autoExpire ?? false, info: info || undefined, completed: false };
    await commit('add_todo', { todo }, () => todos.value.push(todo));
  }
  closeTodoModal();
}

async function handleRemove(id: string) {
  await commit('remove_todo', { id }, () => {
    todos.value = todos.value.filter(t => t.id !== id);
  });
}

async function handleComplete(id: string) {
  const t = todos.value.find(x => x.id === id);
  if (!t) return;
  if (t.repeat) {
    if (!t.repeat) return;
const newDeadline = addIntervalToDateTime(t.deadline, t.repeat);
    const updated: Todo = { ...t, deadline: newDeadline };
    await commit('update_todo', { updated }, () => {
      todos.value = todos.value.map(x => (x.id === updated.id ? updated : x));
    });
  } else {
    await handleRemove(id);
  }
}
function openEdit(todo: Todo) {
  editingTodo.value = todo;
  showTodoModal.value = true;
}

async function handleAutoExpire() {
  const now = Date.now();
  for (const t of todos.value) {
    if (!t.autoExpire) continue;
    if (dateTimeToTs(t.deadline) > now) continue;
    if (t.repeat) {
      const newDeadline = addIntervalToDateTime(t.deadline, t.repeat);
      const updated: Todo = { ...t, deadline: newDeadline };
      await commit('update_todo', { updated }, () => {
        todos.value = todos.value.map(x => (x.id === updated.id ? updated : x));
      });
    } else {
      await handleRemove(t.id);
    }
  }
}

const editingInitial = computed(() => {
  if (!editingTodo.value) return undefined;
  return {
    text: editingTodo.value.text,
    deadline: editingTodo.value.deadline,
    repeat: editingTodo.value.repeat ?? undefined,
    autoExpire: editingTodo.value.autoExpire,
    info: editingTodo.value.info,
  };
});

const SCROLL_MULTIPLIER = 0.6;
const SMOOTH_FACTOR = 0.14;
let wheelListener: (e: WheelEvent) => void;

onMounted(() => {
  loadTodos().then(() => handleAutoExpire());
  isMobile.value = /android|ios|iphone|ipad|ipod/i.test(navigator.userAgent);
  if (isMobile.value) {
    import('@tauri-apps/api/app').then(({ onBackButtonPress }) => {
      onBackButtonPress(() => {
        if (showTodoModal.value) closeTodoModal();
      });
    });
  }
  const el = document.querySelector('.todo-list') as HTMLElement | null;
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
  const el = document.querySelector('.todo-list') as HTMLElement | null;
  if (el && wheelListener) el.removeEventListener('wheel', wheelListener as any);
});
</script>

<template>
  <div class="todo-app fullscreen" :class="{ mobile: isMobile }">
    <div v-if="!isMobile" class="header-row">
      <div class="window-bar" @dblclick.prevent>
        <div class="window-bar-bg" />
        <div class="drag-wrap">
          <div class="drag-handle" data-tauri-drag-region />
        </div>
      </div>
    </div>
    <div class="todo-list">
      <div class="todo-list-inner">
        <TodoItem
          v-for="todo in sortedTodos"
          :key="todo.id"
          :todo="todo"
          @remove="handleRemove"
          @edit="openEdit"
          @complete="handleComplete"
        />
        <div v-if="todos.length === 0" class="empty-tip">点击右下角"+"添加待办事项</div>
      </div>
    </div>
    <TodoModal
      v-if="showTodoModal"
      :initial="editingInitial"
      @submit="handleAddTodo"
      @close="closeTodoModal"
    />
    <button v-show="!showTodoModal" class="fab-sync" :class="{ syncing: isSyncing }" @click.stop="handleSync" :title="isSyncing ? '同步中…' : '从云端刷新'">
      <span class="sync-icon-wrap">
        <svg viewBox="0 0 24 24" class="sync-svg">
          <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
        </svg>
      </span>
    </button>
    <button v-show="!showTodoModal" class="fab" @click.stop="openTodoModal" title="新建">
      <svg viewBox="0 0 24 24" width="28" height="28" fill="#fff">
        <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
      </svg>
    </button>
    <Transition name="toast">
      <div v-if="toast.show" class="toast" :class="toast.type">{{ toast.msg }}</div>
    </Transition>
  </div>
</template>

<style scoped>
.todo-app.fullscreen {
  position: fixed;
  left: 0;
  top: 0;
  right: 0;
  bottom: 0;
  width: 100vw;
  height: 100vh;
  background: #fff;
  border-radius: 0;
  box-shadow: none;
  margin: 0;
  padding: 0;
  z-index: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.header-row {
  display: flex;
  align-items: center;
  margin-bottom: 0px;
  position: relative;
  height: 20px;
  justify-content: flex-start;
  width: 100%;
}
.window-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 20px;
  background-color: rgba(230, 230, 230, 0.2);
  z-index: 999;
  pointer-events: none;
}
.window-bar-bg {
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  bottom: 0;
  background: transparent;
  z-index: 0;
  pointer-events: none;
}
.drag-wrap {
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}
.drag-handle {
  pointer-events: auto;
  width: 140px;
  height: 8px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.12);
  -webkit-app-region: drag;
}
.todo-list {
  position: absolute;
  top: 20px;
  left: 0;
  right: 0;
  bottom: 0;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  scroll-behavior: smooth;
  scrollbar-width: none;
  -ms-overflow-style: none;
}
.todo-app.mobile .todo-list {
  top: 0;
}
.todo-list::-webkit-scrollbar {
  width: 0;
  height: 0;
}
.todo-list-inner {
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
  padding: 4px 12px 24px 12px;
  box-sizing: border-box;
}
.fab {
  --fab-offset: 20px;
  position: fixed;
  right: var(--fab-offset);
  bottom: var(--fab-offset);
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: #409eff;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 8px 20px rgba(64, 158, 255, 0.2);
  border: none;
  z-index: 1001;
  cursor: pointer;
}
.fab:hover {
  background: #66b1ff;
}
.fab-sync {
  --fab-offset: 20px;
  position: fixed;
  right: calc(var(--fab-offset) + 56px + 12px);
  bottom: var(--fab-offset);
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: #f0f2f5;
  color: #333;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.1);
  border: none;
  z-index: 1001;
  cursor: pointer;
  transition: background 0.2s, box-shadow 0.2s;
}
.fab-sync:hover {
  background: #e4e7ed;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.15);
}
.fab-sync .sync-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
}
.fab-sync .sync-svg {
  width: 30px;
  height: 30px;
  fill: #333;
}
.fab-sync.syncing .sync-icon-wrap {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
.toast {
  position: fixed;
  top: 32px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  pointer-events: none;
  box-shadow: 0 4px 16px rgba(0,0,0,0.12);
}
.toast.success { background: #e8f5e9; color: #2e7d32; }
.toast.error   { background: #fce4ec; color: #c62828; }
.toast.info    { background: #e3f2fd; color: #1565c0; }
</style>

<style>
:root {
  --fab-offset: 20px;
  --icon-default: #888;
  --icon-hover: #409eff;
}
.window-btn svg path,
.icon-btn svg path {
  stroke: var(--icon-default);
  transition: stroke 0.18s ease, fill 0.18s ease, color 0.18s ease;
}
.window-btn:hover svg path,
.icon-btn:hover svg path,
.window-btn:hover,
.icon-btn:hover {
  stroke: var(--icon-hover) !important;
  color: var(--icon-hover) !important;
}
.window-btn {
  background: #f5f7fa;
  border: none;
  border-radius: 12px;
  width: 40px;
  height: 40px;
  padding: 0;
  font-size: 22px;
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
  -webkit-app-region: no-drag;
  user-select: none;
  color: #222;
  font-weight: 500;
  outline: none;
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
}
.main-btn {
  background: #409eff;
  color: #fff;
  border-radius: 20px;
  font-size: 18px;
  font-weight: 500;
  width: 100%;
  height: 40px;
  margin-top: 16px;
  border: none;
  outline: none;
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  box-sizing: border-box;
  -webkit-app-region: no-drag;
}
.icon-btn {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  -webkit-app-region: no-drag;
}
.empty-tip {
  text-align: center;
  color: #bbb;
  margin-top: 80px;
  font-size: 18px;
  user-select: none;
  -webkit-user-select: none;
  -ms-user-select: none;
}
button {
  cursor: pointer;
}
input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  outline: none;
}
button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}
@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }
  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}
.toast-enter-active { transition: opacity 0.25s ease, transform 0.25s ease; }
.toast-leave-active { transition: opacity 0.2s ease, transform 0.2s ease; }
.toast-enter-from { opacity: 0; transform: translateX(-50%) translateY(-10px); }
.toast-leave-to   { opacity: 0; transform: translateX(-50%) translateY(-10px); }
</style>
