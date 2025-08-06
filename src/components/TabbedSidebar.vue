<template>
  <div class="tabbed-sidebar">
    <!-- Tab Content -->
    <div class="tab-content">
      <!-- Connection Tab -->
      <div v-show="activeTab === 0" class="tab-panel">
        <slot name="connection-tab"></slot>
      </div>
      
      <!-- Discovery Tab -->
      <div v-show="activeTab === 1" class="tab-panel">
        <slot name="discovery-tab"></slot>
      </div>
      
      <!-- Import Tab -->
      <div v-show="activeTab === 2" class="tab-panel">
        <slot name="import-tab"></slot>
      </div>

      <!-- Stats Tab -->
      <div v-show="activeTab === 3" class="tab-panel">
        <slot name="stats-tab"></slot>
      </div>

      <!-- Active Index Tab -->
      <div v-show="activeTab === 4" class="tab-panel">
        <slot name="active-index-tab"></slot>
      </div>

      <!-- About Tab -->
      <div v-show="activeTab === 5" class="tab-panel">
        <slot name="about-tab"></slot>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';

// Active tab state, now controlled by parent via prop
const props = defineProps({
  activeTab: {
    type: Number,
    default: 0
  }
});
</script>

<style scoped>
.tabbed-sidebar {
  display: flex;
  flex-direction: column;
  min-height: 100%;
}

.tab-content {
  flex: 1;
}

.tab-panel {
  padding: 1rem;
  margin-top: 1rem;
  margin-bottom: 1rem;
  background-color: #373838;
  border-radius: 10px;
  box-shadow: 0 0 10px 0 rgba(0, 0, 0, 0.5);
}
</style> 