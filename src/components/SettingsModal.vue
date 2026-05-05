<template>
  <BaseModal title="设置" @close="close">
    <div class="settings-body">
      <div class="settings-row">
        <label class="settings-label">服务器地址</label>
        <input v-model="serverUrl" class="form-input" placeholder="https://countdownserver.witoncampus.me">
      </div>
      <div class="settings-row">
        <label class="settings-label">API Token</label>
        <div class="token-wrap">
          <input v-model="apiToken" class="form-input token-input" :type="showToken ? 'text' : 'password'" placeholder="请输入 token">
          <button class="token-toggle" @click="showToken = !showToken" type="button" tabindex="-1">
            <svg viewBox="0 0 24 24" width="18" height="18">
              <path v-if="showToken" :d="mdiEyeOff" fill="currentColor" />
              <path v-else :d="mdiEye" fill="currentColor" />
            </svg>
          </button>
        </div>
      </div>
      <button class="test-btn" :disabled="testing" @click="handleTest">
        {{ testing ? '测试中…' : '测试连接' }}
      </button>
      <div class="settings-actions">
        <button class="cancel-btn" @click="close">取消</button>
        <button class="save-btn" @click="handleSave">保存</button>
      </div>
    </div>
  </BaseModal>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { mdiEye, mdiEyeOff } from '@mdi/js';
import BaseModal from './BaseModal.vue';
import { testConnection } from '../api';
import { showToast } from '../utils/toast';
import { STORAGE_URL, STORAGE_TOKEN, getServerUrl, getApiToken } from '../composables/useSync';

const emit = defineEmits(['close']);

const serverUrl = ref(getServerUrl());
const apiToken = ref(getApiToken());
const showToken = ref(false);
const testing = ref(false);

async function handleTest() {
  testing.value = true;
  try {
    const msg = await testConnection(serverUrl.value, apiToken.value);
    showToast(msg, 'success');
  } catch (e) {
    showToast(typeof e === 'string' ? e : '连接失败', 'error');
  } finally {
    testing.value = false;
  }
}

function handleSave() {
  localStorage.setItem(STORAGE_URL, serverUrl.value);
  localStorage.setItem(STORAGE_TOKEN, apiToken.value);
  showToast('设置已保存', 'success');
  emit('close');
}

function close() {
  emit('close');
}
</script>

<style scoped>
.settings-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 20px 16px;
}
.settings-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.settings-label {
  color: #666;
  font-size: 14px;
}
.token-wrap {
  position: relative;
  display: flex;
  align-items: center;
}
.token-input {
  padding-right: 40px;
}
.token-input::-ms-reveal,
.token-input::-ms-clear,
.token-input::-webkit-credentials-auto-fill-button,
.token-input::-webkit-reveal {
  display: none !important;
}
.token-toggle {
  position: absolute;
  right: 4px;
  background: none;
  border: none;
  padding: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #888;
  border-radius: 6px;
  transition: background 0.15s;
}
.token-toggle:hover {
  background: rgba(0,0,0,0.05);
}
.test-btn {
  width: 100%;
  border-radius: 20px;
  font-size: 15px;
  font-weight: 500;
  height: 38px;
  cursor: pointer;
  border: 1px solid #dcdfe6;
  background: #ffffff;
  color: #333;
  transition: background 0.2s, border-color 0.2s;
  font-family: inherit;
  outline: none;
  margin-top: 4px;
}
.test-btn:hover {
  border-color: #409eff;
  color: #409eff;
}
.test-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.settings-actions {
  display: flex;
  gap: 10px;
  margin-top: 4px;
}
.cancel-btn {
  flex: 1;
  border-radius: 20px;
  font-size: 15px;
  font-weight: 500;
  height: 38px;
  cursor: pointer;
  border: 1px solid #dcdfe6;
  background: #ffffff;
  color: #333;
  font-family: inherit;
  outline: none;
  transition: background 0.2s;
}
.cancel-btn:hover {
  background: #f5f5f5;
}
.save-btn {
  flex: 1;
  border-radius: 20px;
  font-size: 15px;
  font-weight: 500;
  height: 38px;
  cursor: pointer;
  border: none;
  background: #409eff;
  color: #fff;
  font-family: inherit;
  outline: none;
  transition: background 0.2s;
}
.save-btn:hover {
  background: #66b1ff;
}
</style>
