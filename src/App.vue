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

async function commit(command: string, args: Record<string, unknown>, optimistic: () => void) {
  try {
    const ok = await invoke<boolean>(command, args);
    if (ok) await loadTodos();
    else optimistic();
  } catch (e) {
    console.error(`invoke ${command}`, e);
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
async function handleAddTodo({ text, deadline, repeat }: { text: string; deadline: DateTime; repeat?: DateTime | null }) {
  if (editingTodo.value) {
    const updated: Todo = {
      id: editingTodo.value.id,
      text,
      deadline,
      repeat: repeat ?? null,
      completed: editingTodo.value.completed,
    };
    await commit('update_todo', { todo: updated }, () => {
      todos.value = todos.value.map(t => (t.id === updated.id ? updated : t));
    });
    editingTodo.value = null;
  } else {
    const todo: Todo = { id: Date.now(), text, deadline, repeat: repeat ?? null, completed: false };
    await commit('add_todo', { todo }, () => todos.value.push(todo));
  }
  closeTodoModal();
}

async function handleRemove(id: number) {
  await commit('remove_todo', { id }, () => {
    todos.value = todos.value.filter(t => t.id !== id);
  });
}

async function handleComplete(id: number) {
  const t = todos.value.find(x => x.id === id);
  if (!t) return;
  if (t.repeat) {
    const newDeadline = addIntervalToDateTime(t.deadline, t.repeat as DateTime);
    const updated: Todo = { ...t, deadline: newDeadline };
    await commit('update_todo', { todo: updated }, () => {
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

const editingInitial = computed(() => {
  if (!editingTodo.value) return undefined;
  return {
    text: editingTodo.value.text,
    deadline: editingTodo.value.deadline,
    repeat: editingTodo.value.repeat ?? undefined,
  };
});

const SCROLL_MULTIPLIER = 0.6;
const SMOOTH_FACTOR = 0.14;
let wheelListener: (e: WheelEvent) => void;

onMounted(() => {
  loadTodos();
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
        <div v-if="todos.length === 0" class="empty-tip">点击右下角"+"添加待办事项</div>
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
  padding-top: 0px;
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
.iconfont {
  color: inherit;
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
</style>
