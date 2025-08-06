<template>
  <div 
    v-if="visible" 
    class="node-context-menu"
    :style="{ left: `${position.x}px`, top: `${position.y}px` }"
  >
    <div 
      class="menu-option"
      @click="$emit('expand', nodeData)"
    >
      Expand
    </div>
  </div>
</template>

<script setup>

const props = defineProps({
  visible: Boolean,
  position: Object,
  nodeData: Object
});

defineEmits(['expand']);
</script>

<style scoped>
.node-context-menu {
  position: absolute;
  background-color: var(--color-card-bg);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  box-shadow: 0 2px 5px var(--color-card-shadow);
  z-index: 10001;
}

.menu-option {
  padding: 8px 12px;
  cursor: pointer;
  color: var(--color-text);
}

.menu-option:hover {
  background-color: var(--color-button-hover);
}
</style> 