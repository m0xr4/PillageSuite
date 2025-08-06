<template>
  <div class="card">
    <h2 class="card-title">Saved Queries</h2>
    
    <div class="query-sections">
      <!-- Predefined Queries -->
      <div class="query-section predefined-queries-section">
        <h3 class="section-title">Predefined Queries</h3>
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
      
      <!-- User Saved Queries -->
      <div class="query-section user-queries-section">
        <div class="section-header">
          <h3 class="section-title">Your Saved Queries</h3>
        </div>
        
        <div v-if="userQueries.length === 0" class="empty-state">
          <div class="empty-state-message">
            <div class="empty-state-icon-container">
              <svg class="empty-state-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </div>
            <h3 class="empty-state-title">No Saved Queries</h3>
            <p class="empty-state-description">
              You haven't saved any queries yet. Run a query and click the save button to add it here.
            </p>
          </div>
        </div>
        
        <div v-else class="query-list">
          <div v-for="(query, index) in userQueries" :key="`user-${index}`" class="query-item">
            <div class="query-name">{{ query.name }}</div>
            <div class="query-actions">
              <button class="query-action" @click="runQuery(query.cypher)">
                <svg xmlns="http://www.w3.org/2000/svg" class="icon-small" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd" />
                </svg>
              </button>
              <button class="query-action" @click="confirmDeleteDialog(index)">
                <svg xmlns="http://www.w3.org/2000/svg" class="icon-small" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Save Query Dialog -->
    <div v-if="showDialog" class="save-dialog-container">
      <div class="save-dialog-backdrop" @click="cancelSave"></div>
      
      <div class="save-dialog-panel">
        <div class="save-dialog-content">
          <div class="save-dialog-header">
            <div class="save-icon">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
              </svg>
            </div>
            <div class="save-text">
              <h3 class="save-title">
                Save Query
              </h3>
            </div>
          </div>
          
          <div class="save-dialog-form">
            <label for="query-name" class="save-form-label">Query Name</label>
            <input 
              id="query-name" 
              ref="queryNameInput"
              v-model="queryName" 
              type="text" 
              class="save-form-input"
              placeholder="Enter a name for this query"
              @keyup.enter="confirmSave"
            >
          </div>
          
          <div class="save-dialog-actions">
            <button 
              type="button" 
              class="save-confirm-button"
              @click="confirmSave"
            >
              Save
            </button>
            <button 
              type="button" 
              class="save-cancel-button"
              @click="cancelSave"
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Delete Query Confirmation Dialog -->
    <div v-if="showDeleteDialog" class="save-dialog-container">
      <div class="save-dialog-backdrop" @click="cancelDelete"></div>
      
      <div class="save-dialog-panel">
        <div class="save-dialog-content">
          <div class="save-dialog-header">
            <div class="save-icon delete-icon">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </div>
            <div class="save-text">
              <h3 class="save-title">
                Delete Query
              </h3>
            </div>
          </div>
          
          <p class="delete-message">Are you sure you want to delete this saved query? This action cannot be undone.</p>
          
          <div class="save-dialog-actions">
            <button 
              type="button" 
              class="delete-confirm-button"
              @click="confirmDelete"
            >
              Delete
            </button>
            <button 
              type="button" 
              class="save-cancel-button"
              @click="cancelDelete"
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
  currentQuery: {
    type: String,
    default: ''
  }
});

// Define emits
const emit = defineEmits(['runQuery']);

// Saved queries state
const predefinedQueries = ref([]);
const userQueries = ref([]);
const isLoading = ref(true);

// Save dialog state
const showDialog = ref(false);
const queryName = ref('');
const queryNameInput = ref(null);

// Load saved queries from config file via Rust
onMounted(async () => {
  try {
    await loadSavedQueries();
  } catch (error) {
    console.error('Error loading saved queries:', error);
  }
});

// Load saved queries from backend
async function loadSavedQueries() {
  try {
    isLoading.value = true;
    const result = await invoke('get_saved_queries');
    
    predefinedQueries.value = result.predefined;
    userQueries.value = result.user;
  } catch (error) {
    console.error('Error loading saved queries:', error);
  } finally {
    isLoading.value = false;
  }
}

// Run a query
const runQuery = (cypher) => {
  emit('runQuery', cypher);
};

// Show save dialog
const showSaveDialog = () => {
  if (!props.currentQuery.trim()) return;
  
  queryName.value = '';
  showDialog.value = true;
  
  // Focus the input field after dialog is shown
  nextTick(() => {
    if (queryNameInput.value) {
      queryNameInput.value.focus();
    }
  });
};

// Expose methods to parent components
defineExpose({ showSaveDialog });

// Save current query
const confirmSave = async () => {
  if (!queryName.value.trim()) return;
  
  try {
    // Save query to backend
    const result = await invoke('add_user_query', {
      name: queryName.value,
      cypher: props.currentQuery
    });
    
    // Update the queries from the result
    predefinedQueries.value = result.predefined;
    userQueries.value = result.user;
    
    // Close the dialog
    showDialog.value = false;
  } catch (error) {
    console.error('Error saving query:', error);
    alert('Failed to save query: ' + error);
  }
};

// Cancel saving
const cancelSave = () => {
  showDialog.value = false;
};

// Delete a saved query
const deleteQueryIndex = ref(-1);
const showDeleteDialog = ref(false);

const confirmDeleteDialog = (index) => {
  deleteQueryIndex.value = index;
  showDeleteDialog.value = true;
};

const confirmDelete = async () => {
  try {
    // Delete query from backend
    const result = await invoke('delete_user_query', { 
      index: deleteQueryIndex.value 
    });
    
    // Update the queries from the result
    predefinedQueries.value = result.predefined;
    userQueries.value = result.user;
    
    // Close the dialog
    showDeleteDialog.value = false;
  } catch (error) {
    console.error('Error deleting query:', error);
    alert('Failed to delete query: ' + error);
  }
};

const cancelDelete = () => {
  showDeleteDialog.value = false;
};
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.query-sections {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.query-section {
  margin-bottom: 1.5rem;
}

.predefined-queries-section {
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--color-border-light);
}

.user-queries-section {
  padding-top: 0.5rem;
}

.section-header {
  margin-top: 1rem;
}

.section-title {
  font-size: 1rem;
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: 0.5rem;
}

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

.query-actions {
  display: flex;
  gap: 0.25rem;
}

.query-action, .save-query-btn {
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

.query-action:hover, .save-query-btn:hover {
  background-color: var(--color-button-hover);
  color: var(--color-primary-darker);
}

.empty-state {
  padding: 0.5rem;
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

.save-query-btn {
  padding: 0.25rem;
}

.icon-small {
  height: 1rem;
  width: 1rem;
}

/* Save Dialog Styles */
.save-dialog-container {
  position: fixed;
  inset: 0;
  z-index: 50;
  overflow-y: auto;
  display: flex;
  align-items: center;
  justify-content: center;
}

.save-dialog-backdrop {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  transition: opacity 0.3s ease;
}

.save-dialog-panel {
  position: relative;
  width: 100%;
  max-width: 28rem;
  background-color: var(--color-card-bg);
  border-radius: 0.5rem;
  overflow: hidden;
  box-shadow: 0 10px 15px -3px var(--color-card-shadow), 0 4px 6px -2px var(--color-card-shadow);
  transform: translateY(0);
  transition: transform 0.3s ease;
  border: 1px solid var(--color-border);
}

.save-dialog-content {
  padding: 1.5rem;
}

.save-dialog-header {
  display: flex;
  align-items: flex-start;
  margin-bottom: 1rem;
}

.save-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 9999px;
  background-color: rgba(var(--color-primary-rgb, 59, 130, 246), 0.15);
  margin-right: 1rem;
  color: var(--color-primary);
}

.save-text {
  flex: 1;
}

.save-title {
  font-size: 1.125rem;
  font-weight: 500;
  color: var(--color-text);
}

.save-dialog-form {
  margin-top: 1rem;
}

.save-form-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: 0.5rem;
}

.save-form-input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  background-color: var(--color-input-bg);
  color: var(--color-text);
  font-size: 0.875rem;
}

.save-form-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(var(--color-primary-rgb, 59, 130, 246), 0.25);
}

.save-dialog-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 1.5rem;
  gap: 0.75rem;
}

.save-confirm-button {
  padding: 0.5rem 1rem;
  background-color: var(--color-primary);
  color: white;
  font-weight: 500;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  transition: background-color 0.2s;
  border: none;
  cursor: pointer;
}

.save-confirm-button:hover {
  background-color: var(--color-primary-darker, var(--color-primary));
  filter: brightness(90%);
}

.save-cancel-button {
  padding: 0.5rem 1rem;
  background-color: var(--color-button-bg);
  color: var(--color-text);
  font-weight: 500;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  font-size: 0.875rem;
  transition: background-color 0.2s;
  cursor: pointer;
}

.save-cancel-button:hover {
  background-color: var(--color-button-hover);
}

.delete-icon {
  background-color: rgba(var(--color-error-rgb, 239, 68, 68), 0.15);
  color: var(--color-error);
}

.delete-message {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  margin: 0.5rem 0;
}

.delete-confirm-button {
  padding: 0.5rem 1rem;
  background-color: var(--color-error);
  color: white;
  font-weight: 500;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  transition: background-color 0.2s;
  border: none;
  cursor: pointer;
}

.delete-confirm-button:hover {
  background-color: var(--color-error-darker, var(--color-error));
  filter: brightness(90%);
}
</style> 