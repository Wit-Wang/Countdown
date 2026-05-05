import { ref, onUnmounted, type Ref } from 'vue';
import { syncFromCloud } from '../api';
import type { Todo } from '../types/interface';
import { showToast } from '../utils/toast';

const SYNC_DEBOUNCE_MS = 500;
const STORAGE_URL = 'countdown_server_url';
const STORAGE_TOKEN = 'countdown_api_token';

const DEFAULT_URL = 'https://countdownserver.witoncampus.me';

function getServerUrl(): string {
  return localStorage.getItem(STORAGE_URL) || DEFAULT_URL;
}

function getApiToken(): string {
  return localStorage.getItem(STORAGE_TOKEN) || '';
}

export { STORAGE_URL, STORAGE_TOKEN, DEFAULT_URL, getServerUrl, getApiToken };

export function useSync(todos: Ref<Todo[]>) {
  const isSyncing = ref(false);
  let syncTimer: ReturnType<typeof setTimeout> | null = null;

  async function sync(): Promise<void> {
    if (isSyncing.value) return;
    isSyncing.value = true;
    showToast('同步中…', 'warning', 0);
    try {
      const res = await syncFromCloud(getServerUrl(), getApiToken());
      todos.value = res;
      showToast('同步成功', 'success');
    } catch (e) {
      showToast('同步失败：' + (typeof e === 'string' ? e : '网络错误'), 'error');
    } finally {
      isSyncing.value = false;
    }
  }

  function scheduleSync(): void {
    if (syncTimer) clearTimeout(syncTimer);
    syncTimer = setTimeout(sync, SYNC_DEBOUNCE_MS);
  }

  onUnmounted(() => {
    if (syncTimer) clearTimeout(syncTimer);
  });

  return { isSyncing, sync, scheduleSync };
}
