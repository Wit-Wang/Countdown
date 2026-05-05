import { ref, computed } from 'vue';
import { fetchTodos, addTodo, removeTodo, updateTodo } from '../api';
import { dateTimeToTs, addIntervalToDateTime } from '../utils/datetime';
import type { Todo, DateTime } from '../types/interface';
import { useUuid } from './useUuid';

export type AddTodoPayload = {
  text: string;
  deadline: DateTime;
  repeat?: DateTime | null;
  autoExpire?: boolean;
  info?: string;
};

export function useTodos() {
  const todos = ref<Todo[]>([]);
  const loaded = ref(false);
  const { v4 } = useUuid();

  const sortedTodos = computed(() =>
    todos.value.slice().sort((a, b) => dateTimeToTs(a.deadline) - dateTimeToTs(b.deadline))
  );

  let loadResolve: (() => void) | null = null;
  const loadPromise = new Promise<void>(resolve => { loadResolve = resolve; });

  async function load(): Promise<void> {
    try {
      todos.value = await fetchTodos();
    } catch (e) {
      console.error('get_todos', e);
    } finally {
      loaded.value = true;
      loadResolve?.();
    }
  }

  async function create(payload: AddTodoPayload): Promise<void> {
    const todo: Todo = {
      id: v4(),
      text: payload.text,
      deadline: payload.deadline,
      repeat: payload.repeat ?? null,
      autoExpire: payload.autoExpire ?? false,
      info: payload.info || undefined,
      completed: false,
    };
    await addTodo(todo);
    todos.value.push(todo);
  }

  async function update(id: string, payload: Partial<AddTodoPayload> & { completed?: boolean }): Promise<void> {
    const existing = todos.value.find(t => t.id === id);
    if (!existing) return;
    const updated: Todo = {
      ...existing,
      text: payload.text ?? existing.text,
      deadline: payload.deadline ?? existing.deadline,
      repeat: payload.repeat !== undefined ? (payload.repeat ?? null) : existing.repeat,
      autoExpire: payload.autoExpire ?? existing.autoExpire,
      info: 'info' in payload ? (payload.info ?? undefined) : existing.info,
      completed: payload.completed ?? existing.completed,
    };
    await updateTodo(updated);
    todos.value = todos.value.map(t => (t.id === id ? updated : t));
  }

  async function remove(id: string): Promise<void> {
    await removeTodo(id);
    todos.value = todos.value.filter(t => t.id !== id);
  }

  async function complete(id: string): Promise<void> {
    const t = todos.value.find(x => x.id === id);
    if (!t) return;
    if (t.repeat) {
      const newDeadline = addIntervalToDateTime(t.deadline, t.repeat);
      const updated: Todo = { ...t, deadline: newDeadline };
      await updateTodo(updated);
      todos.value = todos.value.map(x => (x.id === id ? updated : x));
    } else {
      await remove(id);
    }
  }

  async function autoExpire(): Promise<void> {
    const now = Date.now();
    for (const t of todos.value) {
      if (!t.autoExpire) continue;
      if (dateTimeToTs(t.deadline) > now) continue;
      if (t.repeat) {
        const newDeadline = addIntervalToDateTime(t.deadline, t.repeat);
        const updated: Todo = { ...t, deadline: newDeadline };
        await updateTodo(updated);
        todos.value = todos.value.map(x => (x.id === t.id ? updated : x));
      } else {
        await removeTodo(t.id);
        todos.value = todos.value.filter(x => x.id !== t.id);
      }
    }
  }

  load();

  return {
    todos,
    loaded,
    sortedTodos,
    loadPromise,
    load,
    create,
    update,
    remove,
    complete,
    autoExpire,
  };
}
