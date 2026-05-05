<template>
  <button class="icon-btn" :title="title" @click="$emit('click')">
    <svg viewBox="0 0 24 24" :width="size" :height="size">
      <path :d="icon" fill="currentColor" />
    </svg>
  </button>
</template>

<script lang="ts" setup>
defineProps<{
  icon: string;
  title?: string;
  size?: number;
}>();
defineEmits<{ click: [] }>();
</script>

<style scoped>
.icon-btn {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.18s ease;
  color: #888;
}
.icon-btn:hover { color: #409eff; }
</style>
