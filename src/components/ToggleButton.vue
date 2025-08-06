<template>
  <button 
    class="toggle-button" 
    :class="{ 'toggle-on': modelValue }"
    @click="$emit('update:modelValue', !modelValue)"
    :title="title"
  >
    <span class="toggle-label">{{ label }}</span>
    <span class="toggle-switch"></span>
  </button>
</template>

<script setup>
defineProps({
  modelValue: {
    type: Boolean,
    required: true
  },
  label: {
    type: String,
    default: ''
  },
  title: {
    type: String,
    default: ''
  }
});

defineEmits(['update:modelValue']);
</script>

<style scoped>
.toggle-button {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: white;
  border: 1px solid #ccc;
  border-radius: 4px;
  padding: 4px 8px;
  cursor: pointer;
  min-width: 90px;
  transition: all 0.2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  margin-bottom: 8px;
}

.toggle-button:hover {
  background-color: #f0f0f0;
}

.toggle-label {
  font-size: 12px;
  margin-right: 8px;
  color: #333;
}

.toggle-switch {
  position: relative;
  width: 24px;
  height: 14px;
  border-radius: 12px;
  background-color: #ccc;
  transition: all 0.2s;
}

.toggle-switch::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: white;
  transition: all 0.2s;
}

.toggle-on .toggle-switch {
  background-color: #4338ca;
}

.toggle-on .toggle-switch::after {
  left: 12px;
}
</style> 