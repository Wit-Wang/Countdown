import { invoke } from '@tauri-apps/api/core';
import type { Todo } from './types/interface';

export async function fetchTodos(): Promise<Todo[]> {
  return (await invoke<Todo[]>('get_todos')) || [];
}

export async function addTodo(todo: Todo): Promise<void> {
  await invoke('add_todo', { todo });
}

export async function removeTodo(id: string): Promise<void> {
  await invoke('remove_todo', { id });
}

export async function updateTodo(updated: Todo): Promise<void> {
  await invoke('update_todo', { updated });
}

export async function syncFromCloud(serverUrl: string, apiToken: string): Promise<Todo[]> {
  return (await invoke<Todo[]>('sync_from_cloud', { serverUrl, apiToken })) || [];
}

export async function testConnection(serverUrl: string, apiToken: string): Promise<string> {
  return await invoke<string>('test_connection', { serverUrl, apiToken });
}
