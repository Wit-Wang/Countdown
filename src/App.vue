<script lang="ts" setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import type { Todo, DateTime } from './types/interface';
import { toast } from './utils/toast';
import TodoItem from './components/TodoItem.vue';
import TodoModal from './components/TodoModal.vue';
import SettingsModal from './components/SettingsModal.vue';
import { mdiDotsHorizontal, mdiPlus } from '@mdi/js';
import { useTodos } from './composables/useTodos';
import { useSync } from './composables/useSync';
import { useScroll } from './composables/useScroll';

const { todos, loaded, sortedTodos, loadPromise, create, update, remove, complete, autoExpire } = useTodos();
const { sync, scheduleSync } = useSync(todos);

const showTodoModal = ref(false);
const showSettings = ref(false);
const editingTodo = ref<Todo | null>(null);
const isMobile = ref(false);

function openTodoModal() {
  editingTodo.value = null;
  showTodoModal.value = true;
}
function closeTodoModal() {
  showTodoModal.value = false;
  editingTodo.value = null;
}
function openSettings() { showSettings.value = true; }
function closeSettings() { showSettings.value = false; }

async function handleAddTodo({ text, deadline, repeat, autoExpire: ae, info }: { text: string; deadline: DateTime; repeat?: DateTime | null; autoExpire?: boolean; info?: string }) {
  if (editingTodo.value) {
    await update(editingTodo.value.id, { text, deadline, repeat, autoExpire: ae, info });
    editingTodo.value = null;
  } else {
    await create({ text, deadline, repeat, autoExpire: ae, info });
  }
  scheduleSync();
  closeTodoModal();
}

async function handleRemove(id: string) {
  await remove(id);
  scheduleSync();
}

async function handleComplete(id: string) {
  await complete(id);
  scheduleSync();
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
    autoExpire: editingTodo.value.autoExpire,
    info: editingTodo.value.info,
  };
});

let expireTimer: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  loadPromise.then(async () => { await autoExpire(); sync(); });
  expireTimer = setInterval(() => autoExpire(), 30000);
  isMobile.value = /android|ios|iphone|ipad|ipod/i.test(navigator.userAgent);
  if (isMobile.value) {
    import('@tauri-apps/api/app').then(({ onBackButtonPress }) => {
      onBackButtonPress(() => {
        if (showTodoModal.value) closeTodoModal();
      });
    });
  }
});

onUnmounted(() => {
  if (expireTimer) clearInterval(expireTimer);
});

useScroll('.todo-list');
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
    <button class="settings-btn" v-show="!showSettings && !showTodoModal" @click.stop="openSettings" title="设置">
      <svg viewBox="0 0 24 24" class="settings-icon">
        <path :d="mdiDotsHorizontal" fill="#888" />
      </svg>
    </button>
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
        <div v-if="loaded && todos.length === 0" class="empty-tip">点击右下角"+"添加待办事项</div>
      </div>
    </div>
    <TodoModal
      v-if="showTodoModal"
      :initial="editingInitial"
      @submit="handleAddTodo"
      @close="closeTodoModal"
    />
    <SettingsModal
      v-if="showSettings"
      @close="closeSettings"
    />
    <button v-show="!showTodoModal" class="fab" @click.stop="openTodoModal" title="新建">
      <svg viewBox="0 0 24 24" width="28" height="28" color="#fff">
        <path :d="mdiPlus" fill="currentColor" />
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
  background-color: #fff;
  z-index: 998;
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
.settings-btn {
  z-index: 999;
  position: fixed;
  top: 8px;
  right: 10px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: none;
  padding: 0;
  box-sizing: border-box;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 998;
  box-shadow: 0 0 0 1px rgba(0,0,0,0.08);
  transition: background 0.2s;
}
.settings-btn:hover {
  background: rgba(0,0,0,0.04);
}
.settings-btn:active {
  background: rgba(0,0,0,0.08);
}
.todo-app.mobile .settings-btn {
  top: calc(8px + env(safe-area-inset-top, 0px));
}
.settings-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
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
.toast.warning { background: #fff8e1; color: #f57f17; }
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
