<template>
  <div class="modal-mask center-modal" @click.self="$emit('close')">
    <div class="modal-container center-modal">
      <div class="modal-header">
        <span>{{ title }}</span>
        <div class="modal-actions">
          <IconButton :icon="mdiClose" title="关闭" :size="18" @click="$emit('close')" />
        </div>
      </div>
      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { mdiClose } from '@mdi/js';
import IconButton from './IconButton.vue';

defineProps<{
  title: string;
}>();
defineEmits<{ close: [] }>();
</script>
