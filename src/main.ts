import { createApp } from 'vue';
import App from './App.vue';
import './styles/common.css';

const app = createApp(App);
app.mount('#app');

// 全局禁止右键菜单（阻止所有 contextmenu 事件）
if (typeof document !== 'undefined') {
  document.addEventListener('contextmenu', e => e.preventDefault());
}
