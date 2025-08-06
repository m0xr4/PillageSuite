<template>
  <div class="card">
    <h2 class="card-title">Live Search</h2>
    <div class="search-container">
      <input
        v-model="searchTerm"
        type="text"
        placeholder="Search nodes by name..."
        class="search-input"
        :disabled="!isConnected"
      />
      <div v-if="isLoading" class="loading-indicator">
        <svg class="spinner" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="spinner-track" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="spinner-path" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      </div>
    </div>
    
    <div v-if="searchResults.length > 0" class="results-container">
      <ul class="results-list">
        <li 
          v-for="(result, index) in searchResults" 
          :key="index"
          class="result-item"
          @click="selectNode(result)"
        >
          <div class="result-title">{{ result.properties.name }}</div>
          <div class="result-subtitle">{{ result.labels.join(', ') }}</div>
        </li>
      </ul>
    </div>
    
    <div v-else-if="searchTerm && !isLoading" class="no-results">
      <div class="empty-state-message">
        <div class="empty-state-icon-container">
          <svg class="empty-state-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <h3 class="empty-state-title">No Results Found</h3>
        <p class="empty-state-description">
          No nodes matching "{{ searchTerm }}" were found. Try a different search term.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue';

const props = defineProps({
  session: Object,
  isConnected: Boolean
});

const searchTerm = ref('');
const searchResults = ref([]);
const isLoading = ref(false);
let searchTimeout = null;

// Watch for changes in the search term
watch(searchTerm, (newValue) => {
  if (!props.isConnected || !props.session) return;
  
  // Clear previous timeout
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
  
  // Set a new timeout for debouncing
  searchTimeout = setTimeout(() => {
    if (newValue.trim().length > 0) {
      performSearch(newValue);
    } else {
      searchResults.value = [];
    }
  }, 300); // 300ms debounce
});

// Perform the search
const performSearch = async (term) => {
  if (!props.isConnected || !props.session) return;
  
  isLoading.value = true;
  
  try {
    const query = `
      MATCH (n)
      WHERE toLower(n.name) CONTAINS toLower($searchTerm)
      RETURN n
      LIMIT 10
    `;
    
    const result = await props.session.run(query, { searchTerm: term });
    
    // Process results
    searchResults.value = result.records.map(record => record.get('n'));
    
  } catch (error) {
    console.error('Search error:', error);
    searchResults.value = [];
  } finally {
    isLoading.value = false;
  }
};

// Select a node from the search results
const selectNode = (node) => {
  emit('nodeSelected', node);
};

// Define emits
const emit = defineEmits(['nodeSelected']);
</script>

<style scoped>
.search-container {
  position: relative;
  margin-bottom: 0.5rem;
}

.search-input {
  width: 90%;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--color-input-border);
  border-radius: 0.375rem;
  background-color: var(--color-input-bg);
  color: var(--color-text);
  box-shadow: 0 1px 2px var(--color-card-shadow);
  font-size: 0.875rem;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(var(--color-primary-rgb), 0.3);
}

.loading-indicator {
  position: absolute;
  right: 0.75rem;
  top: 0.5rem;
}

.spinner {
  animation: spin 1s linear infinite;
  height: 1.25rem;
  width: 1.25rem;
  color: var(--color-primary);
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.spinner-track {
  opacity: 0.25;
}

.spinner-path {
  opacity: 0.75;
}

.results-container {
  margin-top: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  background-color: var(--color-card-bg);
  max-height: 15rem;
  overflow-y: auto;
}

.results-list {
  list-style-type: none;
  margin: 0;
  padding: 0;
}

.result-item {
  padding: 0.5rem 1rem;
  border-bottom: 1px solid var(--color-border-light);
  cursor: pointer;
  color: var(--color-text);
}

.result-item:last-child {
  border-bottom: none;
}

.result-item:hover {
  background-color: var(--color-button-hover);
}

.result-title {
  font-weight: 500;
  color: var(--color-text);
}

.result-subtitle {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.no-results {
  margin-top: 0.5rem;
  padding: 1rem;
}

.empty-state-message {
  text-align: center;
  padding: 1.5rem;
  background-color: var(--color-card-bg);
  border-radius: 0.375rem;
  border: 1px dashed var(--color-border);
}

.empty-state-icon-container {
  margin-bottom: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 4rem;
  height: 4rem;
  border-radius: 9999px;
  background-color: rgba(var(--color-info-rgb, 59, 130, 246), 0.1);
  color: var(--color-info);
  margin-left: auto;
  margin-right: auto;
}

.empty-state-icon {
  width: 2.5rem;
  height: 2.5rem;
  margin: auto;
}

.empty-state-title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: var(--color-text);
}

.empty-state-description {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  max-width: 28rem;
  margin-left: auto;
  margin-right: auto;
}
</style> 