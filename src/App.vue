<script lang="ts" setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import type { Todo, DateTime } from './types/interface';
import { dateTimeToTs, addIntervalToDateTime } from './utils/datetime';
import TodoItem from './components/TodoItem.vue';
import TodoModal from './components/TodoModal.vue';
import { invoke } from '@tauri-apps/api/core';

const todos = ref<Todo[]>([]);
const showTodoModal = ref(false);
const editingTodo = ref<Todo | null>(null);

const sortedTodos = computed(() => {
  return todos.value.slice().sort((a, b) => dateTimeToTs(a.deadline) - dateTimeToTs(b.deadline));
});

async function loadTodos() {
  try {
    const res = await invoke<Todo[]>('get_todos');
    todos.value = res || [];
  } catch (e) {
    console.error('get_todos failed', e);
  }
}

function openTodoModal() {
  // open as "new" -> clear editing state
  editingTodo.value = null;
  showTodoModal.value = true;
}
function closeTodoModal() {
  showTodoModal.value = false;
  editingTodo.value = null;
}
async function handleAddTodo({
  text,
  deadline,
  repeat,
}: {
  text: string;
  deadline: DateTime;
  repeat?: DateTime | null;
}) {
  if (editingTodo.value) {
    const updated: Todo = {
      id: editingTodo.value.id,
      text,
      deadline,
      repeat: repeat ?? null,
      completed: editingTodo.value.completed,
    };
    try {
      const ok = await invoke<boolean>('update_todo', { todo: updated });
      if (ok) await loadTodos();
      else todos.value = todos.value.map(t => (t.id === updated.id ? updated : t));
    } catch (e) {
      console.error('update_todo failed', e);
      todos.value = todos.value.map(t => (t.id === updated.id ? updated : t));
    }
    editingTodo.value = null;
  } else {
    const todo: Todo = { id: Date.now(), text, deadline, repeat: repeat ?? null, completed: false };
    try {
      const ok = await invoke<boolean>('add_todo', { todo });
      if (ok) await loadTodos();
      else todos.value.push(todo);
    } catch (e) {
      console.error('add_todo failed', e);
      todos.value.push(todo);
    }
  }
  closeTodoModal();
}

async function handleRemove(id: number) {
  try {
    const ok = await invoke<boolean>('remove_todo', { id });
    if (ok) await loadTodos();
    else todos.value = todos.value.filter(t => t.id !== id);
  } catch (e) {
    console.error('remove_todo failed', e);
    todos.value = todos.value.filter(t => t.id !== id);
  }
}
// 完成逻辑：若有 repeat（DateTime interval）则把 deadline += repeat 并更新；否则等同于删除
async function handleComplete(id: number) {
  const t = todos.value.find(x => x.id === id);
  if (!t) return;
  if (t.repeat) {
    const newDeadline = addIntervalToDateTime(t.deadline, t.repeat as DateTime);
    const updated: Todo = { ...t, deadline: newDeadline };
    try {
      const ok = await invoke<boolean>('update_todo', { todo: updated });
      if (ok) await loadTodos();
      else todos.value = todos.value.map(x => (x.id === updated.id ? updated : x));
    } catch (e) {
      console.error('update_todo failed', e);
      todos.value = todos.value.map(x => (x.id === updated.id ? updated : x));
    }
  } else {
    // non-repeating: treat completion as delete
    await handleRemove(id);
  }
}
function openEdit(todo: Todo) {
  editingTodo.value = todo;
  showTodoModal.value = true;
}

const editingInitial = computed(() => {
  if (!editingTodo.value) return undefined;
  return {
    text: editingTodo.value.text,
    deadline: editingTodo.value.deadline,
    repeat: editingTodo.value.repeat ?? undefined,
  };
});
// settings and close removed per request

// Smooth wheel-based scrolling for `.todo-list` to create a nicer feel
let wheelListener: (e: WheelEvent) => void;
// 调整这两个常量以控制滚动速度与阻尼：
// `SCROLL_MULTIPLIER` 缩放原始滚轮增量（越小越慢）
// `SMOOTH_FACTOR` 控制插值步长（越小到达目标越慢）
const SCROLL_MULTIPLIER = 0.6;
const SMOOTH_FACTOR = 0.14;
onMounted(() => {
  loadTodos();
  const el = document.querySelector('.todo-list') as HTMLElement | null;
  if (!el) return;
  let isAnimating = false;
  let target = el.scrollTop;

  wheelListener = (e: WheelEvent) => {
    if (e.ctrlKey) return; // allow zoom gestures
    e.preventDefault();
    // accumulate target scroll position (apply multiplier to slow the effect)
    target = Math.max(
      0,
      Math.min(el.scrollHeight - el.clientHeight, target + e.deltaY * SCROLL_MULTIPLIER)
    );

    if (!isAnimating) {
      isAnimating = true;
      const step = () => {
        const current = el.scrollTop;
        const diff = target - current;
        const delta = diff * SMOOTH_FACTOR; // smoothing factor (smaller = slower)
        if (Math.abs(diff) < 0.5) {
          el.scrollTop = target;
          isAnimating = false;
          return;
        }
        el.scrollTop = current + delta;
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
  <div class="todo-app fullscreen">
    <div class="header-row">
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
        <div v-if="todos.length === 0" class="empty-tip">点击右下角“+”添加待办事项</div>
      </div>
    </div>
    <TodoModal
      v-if="showTodoModal"
      :initial="editingInitial"
      @submit="handleAddTodo"
      @close="closeTodoModal"
    />
    <button v-show="!showTodoModal" class="fab" @click.stop="openTodoModal" title="新建">
      <span class="iconfont">&#xE710;</span>
    </button>
  </div>
</template>

<style scoped>
.window-btn-group {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-right: 0;
  z-index: 1;
  -webkit-app-region: no-drag;
}
/* fixed-right removed; floating new button used instead */
.iconfont {
  font-family: 'Segoe MDL2 Assets', 'Segoe UI Symbol', 'Segoe UI', Arial, sans-serif;
  font-style: normal;
  font-weight: normal;
  font-size: 18px;
  line-height: 1;
  vertical-align: middle;
  pointer-events: none;
  user-select: none;
  letter-spacing: 0;
  display: inline-block;
}
.window-btn {
  background: #f5f7fa;
  border: none;
  border-radius: 50%;
  font-size: 18px;
  color: #222;
  font-weight: 500;
  transition:
    background 0.2s,
    color 0.2s;
  outline: none;
  cursor: pointer;
  user-select: none;
  box-sizing: border-box;
  width: 40px;
  height: 40px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  z-index: 1;
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
  transition:
    background 0.2s,
    color 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  box-sizing: border-box;
}
.main-btn:hover {
  background: #66b1ff;
  color: #fff;
}
.window-btn:hover,
.modal-btn:hover {
  background: #e6f0ff;
  color: #409eff;
}
.window-btn.close {
  color: #222;
}
.window-btn.close:hover {
  background: #e6f0ff;
  color: #409eff;
}
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
  padding: 0 0 24px 0;
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
  pointer-events: none; /* transparent top layer that doesn't block interactions except handle */
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
  pointer-events: auto; /* the only interactive area in top strip */
  width: 140px;
  height: 8px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.12);
  -webkit-app-region: drag;
}
.window-bar > .window-btn {
  position: relative;
  z-index: 1;
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
  transition:
    background 0.2s,
    color 0.2s;
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
.window-btn:hover {
  background: #e6f0ff;
  color: #409eff;
}
.window-btn.close {
  color: #222;
}
.window-btn.close:hover {
  background: #e6f0ff;
  color: #409eff;
}
.todo-list {
  position: absolute;
  top: 20px; /* sit directly under the top 20px strip */
  left: 0;
  right: 0;
  bottom: 0;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  scroll-behavior: smooth;
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE 10+ */
}
.todo-list::-webkit-scrollbar {
  width: 0;
  height: 0;
}
.todo-list-inner {
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
  padding: 12px 12px 32px 12px;
  box-sizing: border-box;
  padding-top: 0px; /* minimal gap under the top strip removed per request */
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
.fab:hover {
  background: #66b1ff;
}

/* Global (non-scoped) styles to unify icon/button look and fab offset variable */
</style>

<style>
:root {
  --fab-offset: 20px;
  --icon-default: #888;
  --icon-hover: #409eff;
}
/* unify svg stroke / icon color */
.window-btn svg path,
.icon-btn svg path {
  stroke: var(--icon-default);
  transition:
    stroke 0.18s ease,
    fill 0.18s ease,
    color 0.18s ease;
}
.window-btn:hover svg path,
.icon-btn:hover svg path,
.window-btn:hover,
.icon-btn:hover {
  stroke: var(--icon-hover) !important;
  color: var(--icon-hover) !important;
}
/* ensure iconfont glyphs inherit color */
.empty-tip {
  user-select: none;
  -webkit-user-select: none;
  -ms-user-select: none;
}
.iconfont {
  color: inherit;
}

/* Global button styles so modal buttons inherit same look */
.window-btn {
  background: #f5f7fa;
  border: none;
  border-radius: 12px;
  width: 40px;
  height: 40px;
  padding: 0;
  font-size: 22px;
  cursor: pointer;
  transition:
    background 0.2s,
    color 0.2s;
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
  transition:
    background 0.2s,
    color 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  box-sizing: border-box;
}
.icon-btn {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
}

.empty-tip {
  text-align: center;
  color: #bbb;
  margin-top: 80px;
  font-size: 18px;
}
form {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}
input[type='datetime-local'] {
  flex: 1;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
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
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
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
</style>
