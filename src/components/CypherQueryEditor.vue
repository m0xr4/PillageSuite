<template>
  <div class="card">
    <h2 class="card-title">Cypher Query Editor</h2>
    <textarea
      v-model="query"
      rows="5"
      placeholder="MATCH (n) RETURN n LIMIT 25"
      class="query-textarea"
      :disabled="!isConnected"
    ></textarea>
    <div class="button-row">
      <div class="left-buttons">
        <button
          @click="executeQuery"
          class="button-primary"
          :disabled="!isConnected || isLoading"
        >
          {{ isLoading ? 'Running...' : 'Run Query' }}
        </button>
        <button
          v-if="isLoading"
          @click="handleTerminateQuery"
          class="button-danger"
          title="Terminate running query"
        >
          Terminate
        </button>
        <div v-if="queryStatus" :class="queryStatusClass">
          {{ queryStatus }}
        </div>
      </div>
      <button
        class="button-secondary"
        title="Save current query"
        @click="saveCurrentQuery"
        :disabled="!query.trim()"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="icon-small" viewBox="0 0 20 20" fill="currentColor">
          <path d="M7.707 10.293a1 1 0 10-1.414 1.414l3 3a1 1 0 001.414 0l3-3a1 1 0 00-1.414-1.414L11 11.586V6h5a2 2 0 012 2v7a2 2 0 01-2 2H4a2 2 0 01-2-2V8a2 2 0 012-2h5v5.586l-1.293-1.293zM9 4a1 1 0 012 0v2H9V4z" />
        </svg>
        Save Query
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { executeCypherQuery, terminateQuery } from '../services/neo4jService';

const props = defineProps({
  session: Object,
  isConnected: Boolean
});

const query = ref('MATCH (n) RETURN n LIMIT 25');
const isLoading = ref(false);
const queryStatus = ref('');
const queryStatusClass = ref('status-neutral');

const executeQuery = async () => {
  if (!props.isConnected) {
    queryStatus.value = 'Not connected to Neo4j';
    queryStatusClass.value = 'status-error';
    return;
  }

  isLoading.value = true;
  queryStatus.value = 'Running query...';
  queryStatusClass.value = 'status-info';
  
  // Emit the current query
  emit('querySubmit', query.value);

  try {
    // Use the session from props with the executeCypherQuery service
    const result = await executeCypherQuery(query.value, props.session);
    
    // If the query was terminated, don't process the result
    if (!isLoading.value) {
      return;
    }
    
    // Emit the result
    emit('queryResult', result);
    
    // Update status
    isLoading.value = false;
    queryStatus.value = `Query executed successfully. Returned ${result.records.length} records.`;
    queryStatusClass.value = 'status-success';
  } catch (error) {
    // If it was terminated, this was already handled
    if (!isLoading.value) {
      return;
    }
    
    console.error('Query error:', error);
    queryStatus.value = `Query failed: ${error.message}`;
    queryStatusClass.value = 'status-error';
    
    // Emit the error
    emit('queryError', error);
    isLoading.value = false;
  }
};

// Save the current query
const saveCurrentQuery = () => {
  if (query.value.trim()) {
    emit('saveQuery', query.value);
  }
};

// Method to run a specific query directly (for saved queries)
const runExternalQuery = async (cypherQuery) => {
  query.value = cypherQuery; // Update the textarea with the saved query
  
  if (!props.isConnected) {
    queryStatus.value = 'Not connected to Neo4j';
    queryStatusClass.value = 'status-error';
    return;
  }

  isLoading.value = true;
  queryStatus.value = 'Running query...';
  queryStatusClass.value = 'status-info';
  
  // Emit the current query
  emit('querySubmit', query.value);

  try {
    // Use the session from props with the executeCypherQuery service
    const result = await executeCypherQuery(query.value, props.session);
    
    // If the query was terminated, don't process the result
    if (!isLoading.value) {
      return;
    }
    
    // Add metadata to identify this as a saved query for special handling
    if (result && result.summary) {
      result.summary.fromSavedQuery = true;
    }
    
    // Emit the result
    emit('queryResult', result);
    
    // Update status
    isLoading.value = false;
    queryStatus.value = `Query executed successfully. Returned ${result.records.length} records.`;
    queryStatusClass.value = 'status-success';
  } catch (error) {
    // If it was terminated, this was already handled
    if (!isLoading.value) {
      return;
    }
    
    console.error('Query error:', error);
    queryStatus.value = `Query failed: ${error.message}`;
    queryStatusClass.value = 'status-error';
    
    // Emit the error
    emit('queryError', error);
    isLoading.value = false;
  }
};

// Handle terminating a running query
const handleTerminateQuery = async () => {
  if (isLoading.value) {
    try {
      queryStatus.value = 'Terminating query...';
      queryStatusClass.value = 'status-warning';
      
      const result = await terminateQuery();
      
      if (result.success) {
        isLoading.value = false;
        queryStatus.value = 'Query terminated successfully.';
        queryStatusClass.value = 'status-warning';
        
        // Emit the termination event
        emit('queryTerminated');
      } else {
        queryStatus.value = `Failed to terminate query: ${result.error || 'Unknown error'}`;
        queryStatusClass.value = 'status-error';
      }
    } catch (error) {
      console.error('Error in terminate query:', error);
      queryStatus.value = `Termination error: ${error.message}`;
      queryStatusClass.value = 'status-error';
    }
  }
};

// Define emits
const emit = defineEmits(['queryResult', 'queryError', 'querySubmit', 'saveQuery', 'queryTerminated']);

// Expose the runExternalQuery method to parent components
defineExpose({ runExternalQuery });
</script>

<style scoped>
/* Component-specific styles only */
.query-textarea {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--color-input-border);
  border-radius: 0.375rem;
  background-color: var(--color-input-bg);
  color: var(--color-text);
  box-shadow: 0 1px 2px var(--color-card-shadow);
  font-size: 0.875rem;
  font-family: monospace;
  margin-bottom: 0.75rem;
  box-sizing: border-box;
  min-width: 100%;
  resize: vertical;
}

.query-textarea:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(var(--color-primary-rgb, 59, 130, 246), 0.3);
}

.button-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.left-buttons {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.button-secondary {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.75rem;
  background-color: var(--color-button-bg);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  color: var(--color-text);
  font-weight: 500;
  font-size: 0.875rem;
  transition: all 0.2s;
  cursor: pointer;
}

.button-secondary:hover {
  background-color: var(--color-button-hover);
  color: var(--color-primary);
}

.button-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.button-secondary .icon-small {
  width: 1rem;
  height: 1rem;
}

.button-danger {
  padding: 0.5rem 0.75rem;
  background-color: var(--color-error);
  color: white;
  border: none;
  border-radius: 0.375rem;
  font-weight: 500;
  font-size: 0.875rem;
  transition: all 0.2s;
  cursor: pointer;
}

.button-danger:hover {
  background-color: var(--color-error-hover, darkred);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}
</style> 