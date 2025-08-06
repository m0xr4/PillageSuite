<template>
  <div class="node-details" :class="{ 'open': visible }">
    <div class="node-details-header">
      <button class="toggle-button" @click="$emit('toggle')">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M5 12h14M12 5l7 7-7 7" />
        </svg>
      </button>
      <h3>Node Details</h3>
    </div>
    
    <div v-if="nodeData" class="node-details-content">
      <div class="node-type">
        <span class="label">Type:</span>
        <span class="value">{{ nodeLabelsText }}</span>
      </div>
      
      <div class="node-id">
        <span class="label">ID:</span>
        <span class="value">{{ nodeData.id }}</span>
      </div>
      
      <div class="node-properties">
        <h4>Properties</h4>
        <div v-if="nodeData.properties && hasProperties" class="property-list">
          <div v-for="(value, key) in nodeData.properties" :key="key" class="property-item">
            <span class="property-name">{{ key }}:</span>
            <span class="property-value">{{ formatValue(value) }}</span>
          </div>
        </div>
        <div v-else class="no-properties">
          No properties found
        </div>
      </div>
      
      <!-- Predefined Queries Section - Only for Identity nodes -->
      <div v-if="isIdentityNode" class="node-queries">
        <h4>Predefined Queries</h4>
        <div class="query-list">
          <div v-for="(query, index) in predefinedQueries" :key="`predefined-${index}`" class="query-item">
            <div class="query-name">{{ query.name }}</div>
            <button class="query-action" @click="runQuery(query.cypher)">
              <svg xmlns="http://www.w3.org/2000/svg" class="icon-small" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
    
    <div v-else class="empty-state">
      <p>Select a node to view details</p>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
  visible: Boolean,
  nodeData: Object
});

const emit = defineEmits(['toggle', 'runQuery']);

const nodeLabelsText = computed(() => {
  if (!props.nodeData || !props.nodeData.labels || props.nodeData.labels.length === 0) {
    return 'Unknown';
  }
  return props.nodeData.labels.join(', ');
});

const hasProperties = computed(() => {
  return props.nodeData && 
         props.nodeData.properties && 
         Object.keys(props.nodeData.properties).length > 0;
});

// Check if the node is of type Identity
const isIdentityNode = computed(() => {
  return props.nodeData && 
         props.nodeData.labels && 
         props.nodeData.labels.includes('Identity');
});

// Saved queries state
const predefinedQueries = ref([]);
const isLoading = ref(true);

// Load saved queries from config file via Rust
onMounted(async () => {
  try {
    await loadSavedQueries();
  } catch (error) {
    console.error('Error loading saved queries:', error);
  }
});

// Watch for changes in isIdentityNode to load queries when needed
watch(isIdentityNode, async (newValue) => {
  if (newValue && predefinedQueries.value.length === 0) {
    await loadSavedQueries();
  }
});

// Load saved queries from backend
async function loadSavedQueries() {
  try {
    isLoading.value = true;
    const result = await invoke('get_saved_queries');
    
    // Use the dedicated identity queries from the backend
    predefinedQueries.value = result.identity;
  } catch (error) {
    console.error('Error loading saved queries:', error);
  } finally {
    isLoading.value = false;
  }
}

// Format property values for display
const formatValue = (value) => {
  if (value === null || value === undefined) {
    return 'null';
  } else if (Array.isArray(value)) {
    return value.length === 0 ? '[]' : JSON.stringify(value);
  } else if (typeof value === 'object') {
    return JSON.stringify(value);
  } else if (typeof value === 'boolean') {
    return value ? 'true' : 'false';
  } else if (typeof value === 'string' && value.length > 100) {
    return value.substring(0, 100) + '...';
  }
  return value.toString();
};

// Run a predefined query
const runQuery = (cypher) => {
  if (props.nodeData && props.nodeData.id) {
    // Replace $nodeId parameter with the actual node ID
    const formattedQuery = cypher.replace('$nodeId', props.nodeData.id);
    
    // Pass a second parameter to indicate this should behave like an expansion
    emit('runQuery', formattedQuery, true);
  }
};
</script>

<style>
.node-details {
  position: absolute;
  top: 0;
  left: -300px; /* Start off-screen on the left */
  width: 300px;
  height: 100%;
  background-color: var(--color-card-bg);
  border-right: 1px solid var(--color-border); /* Changed from border-left to border-right */
  box-shadow: 2px 0 5px var(--color-card-shadow); /* Changed direction of shadow */
  transition: left 0.3s ease-in-out; /* Changed from right to left */
  z-index: 1000;
  overflow-y: auto;
  padding-bottom: 20px;
}

.node-details.open {
  left: 0; /* Slide in from left */
}

.node-details-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-card-bg);
  position: sticky;
  top: 0;
  z-index: 10;
}

.node-details-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.toggle-button {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-text);
  padding: 4px;
  border-radius: 4px;
}

.toggle-button:hover {
  background-color: var(--color-button-hover);
}

.node-details-content {
  padding: 16px;
}

.node-type, .node-id {
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
}

.label {
  font-size: 12px;
  color: var(--color-text-muted);
  margin-bottom: 2px;
}

.value {
  font-size: 14px;
  word-break: break-word;
}

.node-properties, .node-queries {
  margin-top: 20px;
}

.node-properties h4, .node-queries h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
}

.property-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.property-item {
  padding: 8px;
  background-color: var(--color-bg);
  border-radius: 4px;
  border: 1px solid var(--color-border-light);
}

.property-name {
  display: block;
  font-size: 12px;
  color: var(--color-text-muted);
  margin-bottom: 4px;
}

.property-value {
  display: block;
  font-size: 13px;
  word-break: break-word;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100px;
  color: var(--color-text-muted);
  font-style: italic;
  padding: 16px;
}

.no-properties {
  color: var(--color-text-muted);
  font-style: italic;
}

/* Query styles copied from SavedQueries.vue */
.query-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.query-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.15rem;
  background-color: var(--color-card-bg);
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
}

.query-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.query-action {
  display: flex;
  padding: 0;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border-radius: 0.25rem;
  background-color: var(--color-button-bg);
  border: none;
  color: var(--color-primary);
  cursor: pointer;
  transition: all 0.2s;
}

.query-action:hover {
  background-color: var(--color-button-hover);
  color: var(--color-primary-darker);
}

.icon-small {
  height: 1rem;
  width: 1rem;
}
</style> 