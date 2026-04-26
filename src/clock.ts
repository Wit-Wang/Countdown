import { ref } from 'vue';

// 共享的全局时钟：所有需要每秒刷新的组件都可以引用此 `now`。
// 这样可以避免每个组件都创建自己的 setInterval，减少定时器数量和重绘次数。
export const now = ref(Date.now());

if (typeof window !== 'undefined') {
  // 全局每秒更新一次
  setInterval(() => {
    now.value = Date.now();
  }, 1000);
}
