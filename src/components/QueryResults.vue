<template>
  <div v-if="results && !isPathOnlyResult" class="card">
    <h2 class="card-title">Query Results</h2>
    
    <div v-if="results.records && results.records.length > 0" class="results-container">
      <div class="results-actions">
        <div class="pagination-info">
          Showing {{ paginatedRecords.length ? (currentPage - 1) * pageSize + 1 : 0 }} to {{ Math.min(currentPage * pageSize, results.records.length) }} of {{ results.records.length }} results
        </div>
        <button @click="downloadCSV" class="download-button" title="Download as CSV">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="download-icon">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="7 10 12 15 17 10"></polyline>
            <line x1="12" y1="15" x2="12" y2="3"></line>
          </svg>
          CSV
        </button>
      </div>
      
      <table class="results-table">
        <thead class="table-header">
          <tr>
            <th 
              v-for="(key, index) in results.records[0].keys" 
              :key="index"
              class="header-cell sortable-header"
              :class="{ 
                'sorted-asc': sortKey === key && sortDirection === 'asc', 
                'sorted-desc': sortKey === key && sortDirection === 'desc' 
              }"
              @click="sortBy(key)"
            >
              {{ key }}
            </th>
          </tr>
        </thead>
        <tbody class="table-body">
          <tr v-for="(record, recordIndex) in paginatedRecords" :key="recordIndex">
            <td 
              v-for="(key, keyIndex) in record.keys" 
              :key="keyIndex"
              class="data-cell clickable-cell"
              @click="copyToClipboard(record.get(key))"
              title="Click to copy value"
            >
              <div v-if="isNode(record.get(key))">
                <div class="node-label">
                  {{ getNodeLabel(record.get(key)) }}
                </div>
                <div class="props-container">
                  <table class="props-table">
                    <thead class="props-header">
                      <tr>
                        <th class="prop-header-cell">Property</th>
                        <th class="prop-header-cell">Value</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="(value, prop) in record.get(key).properties" :key="prop">
                        <td class="prop-name-cell">{{ prop }}</td>
                        <td class="prop-value-cell">{{ formatValue(value) }}</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
              <div v-else-if="isRelationship(record.get(key))">
                <div class="rel-type">
                  {{ getRelationshipType(record.get(key)) }}
                </div>
                <div class="props-container">
                  <table class="props-table">
                    <thead class="props-header">
                      <tr>
                        <th class="prop-header-cell">Property</th>
                        <th class="prop-header-cell">Value</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="(value, prop) in record.get(key).properties" :key="prop">
                        <td class="prop-name-cell">{{ prop }}</td>
                        <td class="prop-value-cell">{{ formatValue(value) }}</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
              <div v-else-if="isPath(record.get(key))">
                <div class="path-info">
                  Path with {{ getPathLength(record.get(key)) }} segments
                </div>
              </div>
              <div v-else-if="isObject(record.get(key))">
                <table class="props-table">
                  <thead class="props-header">
                    <tr>
                      <th class="prop-header-cell">Property</th>
                      <th class="prop-header-cell">Value</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(value, prop) in record.get(key)" :key="prop">
                      <td class="prop-name-cell">{{ prop }}</td>
                      <td class="prop-value-cell">{{ formatValue(value) }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <div v-else>
                {{ formatValue(record.get(key)) }}
              </div>
            </td>
          </tr>
        </tbody>
      </table>
      
      <!-- Pagination controls -->
      <div v-if="totalPages > 1" class="pagination-controls">
        <button 
          @click="changePage(1)" 
          class="pagination-button"
          :disabled="currentPage === 1"
          title="First page"
        >
          &laquo;
        </button>
        
        <button 
          @click="changePage(currentPage - 1)" 
          class="pagination-button"
          :disabled="currentPage === 1"
          title="Previous page"
        >
          &lsaquo;
        </button>
        
        <div class="page-indicator">
          <span v-if="showPageInput">
            <input 
              type="number" 
              v-model="pageInputValue" 
              class="page-input"
              @keyup.enter="goToPage"
              @blur="showPageInput = false"
              ref="pageInput"
              min="1"
              :max="totalPages"
            />
          </span>
          <span v-else @click="enablePageInput" class="page-number">
            {{ currentPage }}
          </span>
          <span class="page-divider">/</span>
          <span class="total-pages">{{ totalPages }}</span>
        </div>
        
        <button 
          @click="changePage(currentPage + 1)" 
          class="pagination-button"
          :disabled="currentPage === totalPages"
          title="Next page"
        >
          &rsaquo;
        </button>
        
        <button 
          @click="changePage(totalPages)" 
          class="pagination-button"
          :disabled="currentPage === totalPages"
          title="Last page"
        >
          &raquo;
        </button>
      </div>
    </div>
    
    <div v-else class="no-results">
      <div class="empty-state-message">
        <div class="empty-state-icon-container">
          <svg class="empty-state-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <h3 class="empty-state-title">No Results Found</h3>
        <p class="empty-state-description">
          Your query didn't return any results. Try modifying your query or check your data.
        </p>
      </div>
    </div>
  </div>
  
  <!-- Copy success notification -->
  <transition name="fade">
    <div v-if="showCopySuccessMessage" class="copy-success-notification">
      Copied to clipboard!
    </div>
  </transition>
</template>

<script setup>
import { ref, computed, nextTick, watch } from 'vue';

const props = defineProps({
  results: Object
});

// Pagination state
const pageSize = 40; // 40 records per page
const currentPage = ref(1);
const showPageInput = ref(false);
const pageInputValue = ref('1');
const pageInput = ref(null);
const showCopySuccessMessage = ref(false); // New state for notification

// Sorting state
const sortKey = ref(null);
const sortDirection = ref('asc'); // 'asc' or 'desc'

// Reset to page 1 and sorting when results change
watch(() => props.results, () => {
  currentPage.value = 1;
  showPageInput.value = false;
  sortKey.value = null; // Reset sort key
  sortDirection.value = 'asc'; // Reset sort direction
});

// Check if the result contains only paths
const isPathOnlyResult = computed(() => {
  if (!props.results || !props.results.records || props.results.records.length === 0) {
    return false;
  }
  
  // Check if the result is a single column named 'p'
  const keys = props.results.records[0].keys;
  if (keys.length !== 1 || keys[0] !== 'p') {
    return false;
  }
  
  // Check if that column contains a path
  const firstRecord = props.results.records[0];
  return isPath(firstRecord.get('p'));
});

// Computed properties for pagination
const totalPages = computed(() => {
  if (!props.results || !props.results.records) return 0;
  return Math.ceil(sortedRecords.value.length / pageSize);
});

// Computed property for sorted records
const sortedRecords = computed(() => {
  if (!props.results || !props.results.records) return [];
  if (!sortKey.value) return props.results.records; // No sorting if key is null

  const key = sortKey.value;
  const direction = sortDirection.value === 'asc' ? 1 : -1;

  // Create a copy before sorting to avoid mutating original data
  const recordsToSort = [...props.results.records];

  return recordsToSort.sort((a, b) => {
    let valA = a.get(key);
    let valB = b.get(key);

    // Handle potential Neo4j Integer (BigInt)
    if (typeof valA === 'bigint') valA = Number(valA);
    if (typeof valB === 'bigint') valB = Number(valB);

    // Use formatted value for comparison if it's an object/node/rel/path
    // This provides a basic sort but might not be perfect for all complex types
    if (typeof valA === 'object' && valA !== null) valA = formatValue(valA);
    if (typeof valB === 'object' && valB !== null) valB = formatValue(valB);
    
    // Basic comparison
    if (valA < valB) return -1 * direction;
    if (valA > valB) return 1 * direction;
    return 0;
  });
});

const paginatedRecords = computed(() => {
  // Paginate the *sorted* records
  const records = sortedRecords.value; 
  if (!records.length) return [];
  
  const startIndex = (currentPage.value - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  return records.slice(startIndex, endIndex);
});

// Methods for pagination
const changePage = (page) => {
  if (page < 1) page = 1;
  if (page > totalPages.value) page = totalPages.value;
  currentPage.value = page;
  // Scroll to top of results
  const resultsContainer = document.querySelector('.results-container');
  if (resultsContainer) {
    resultsContainer.scrollTop = 0;
  }
};

const enablePageInput = () => {
  showPageInput.value = true;
  pageInputValue.value = currentPage.value.toString();
  nextTick(() => {
    if (pageInput.value) {
      pageInput.value.focus();
      pageInput.value.select();
    }
  });
};

const goToPage = () => {
  let page = parseInt(pageInputValue.value);
  if (isNaN(page) || page < 1) page = 1;
  if (page > totalPages.value) page = totalPages.value;
  changePage(page);
  showPageInput.value = false;
};

// Method to handle sorting
const sortBy = (key) => {
  if (sortKey.value === key) {
    // Toggle direction if same key is clicked
    sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc';
  } else {
    // Set new key and default to ascending
    sortKey.value = key;
    sortDirection.value = 'asc';
  }
  // Go back to page 1 when sorting changes
  currentPage.value = 1; 
};

// Copy to clipboard function
const copyToClipboard = async (value) => {
  const textToCopy = formatValue(value);
  try {
    await navigator.clipboard.writeText(textToCopy);
    // Show success message
    showCopySuccessMessage.value = true; 
    // Hide after a delay
    setTimeout(() => {
      showCopySuccessMessage.value = false;
    }, 1500); // Hide after 1.5 seconds
    console.log('Copied to clipboard:', textToCopy); 
  } catch (err) {
    console.error('Failed to copy text: ', err);
    // Optional: Inform the user about the failure
  }
};

// Download CSV functionality
const downloadCSV = () => {
  if (!props.results || !props.results.records || props.results.records.length === 0) {
    return;
  }
  
  // Get headers from the first record
  const headers = props.results.records[0].keys;
  
  // Create CSV content
  let csvContent = headers.join(',') + '\n';
  
  // Process all records (not just current page)
  props.results.records.forEach(record => {
    const row = headers.map(key => {
      const value = record.get(key);
      return formatValueForCSV(value);
    });
    csvContent += row.join(',') + '\n';
  });
  
  // Create blob and download
  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  
  link.setAttribute('href', url);
  link.setAttribute('download', `neo4j-results-${timestamp}.csv`);
  link.style.visibility = 'hidden';
  
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
};

// Format value specifically for CSV to handle special characters and quoting
const formatValueForCSV = (value) => {
  if (value === null || value === undefined) {
    return '';
  }
  
  if (isNode(value)) {
    // For nodes, use label and ID
    const label = getNodeLabel(value);
    const id = value.identity;
    return `"${escapeCSV(`${label}:${id}`)}"`; 
  }
  
  if (isRelationship(value)) {
    // For relationships, use type and ID
    const type = getRelationshipType(value);
    const id = value.identity;
    return `"${escapeCSV(`${type}:${id}`)}"`; 
  }
  
  if (isPath(value)) {
    // For paths, provide the number of segments
    return `"Path with ${getPathLength(value)} segments"`;
  }
  
  if (typeof value === 'object') {
    // For objects, stringify with double quotes
    return `"${escapeCSV(JSON.stringify(value))}"`;
  }
  
  // For strings that might contain commas or quotes
  if (typeof value === 'string') {
    return `"${escapeCSV(value)}"`;
  }
  
  // For numbers and other primitive types
  return String(value);
};

// Escape quotes for CSV
const escapeCSV = (str) => {
  return str.replace(/"/g, '""');
};

// Check if a value is a Neo4j Node
const isNode = (value) => {
  return value && typeof value === 'object' && value.labels && Array.isArray(value.labels);
};

// Check if a value is a Neo4j Relationship
const isRelationship = (value) => {
  return value && typeof value === 'object' && value.type && typeof value.type === 'string';
};

// Check if a value is a Neo4j Path
const isPath = (value) => {
  return value && typeof value === 'object' && value.segments && Array.isArray(value.segments);
};

// Get the length of a path
const getPathLength = (path) => {
  if (!isPath(path)) return 0;
  return path.segments.length;
};

// Check if a value is a regular object (not a node or relationship)
const isObject = (value) => {
  return value && typeof value === 'object' && !isNode(value) && !isRelationship(value) && !isPath(value) && !Array.isArray(value);
};

// Get the label of a node
const getNodeLabel = (node) => {
  if (!isNode(node)) return '';
  return node.labels.join(', ');
};

// Get the properties of a node as a string
const getNodeProperties = (node) => {
  if (!isNode(node)) return '';
  return JSON.stringify(node.properties);
};

// Get the type of a relationship
const getRelationshipType = (rel) => {
  if (!isRelationship(rel)) return '';
  return rel.type;
};

// Get the properties of a relationship as a string
const getRelationshipProperties = (rel) => {
  if (!isRelationship(rel)) return '';
  return JSON.stringify(rel.properties);
};

// Format a value for display
const formatValue = (value) => {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'object') {
    if (Array.isArray(value)) {
      return JSON.stringify(value);
    }
    return JSON.stringify(value);
  }
  return String(value);
};
</script>

<style>
/* Component-specific styles */
.results-container {
  margin: 0;
  overflow-x: auto;
}

.results-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 1rem;
  border-bottom: 1px solid var(--color-border);
}

.pagination-info {
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.download-button {
  display: flex;
  align-items: center;
  padding: 0.25rem 0.5rem;
  background-color: var(--color-button-bg);
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  color: var(--color-text);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.download-button:hover {
  background-color: var(--color-button-hover);
}

.download-icon {
  margin-right: 0.25rem;
}

.results-table {
  width: 100%;
  border-collapse: collapse;
}

.table-header {
  background-color: var(--color-sidebar-bg);
  position: sticky;
  top: 0;
  z-index: 10;
}

.header-cell {
  padding: 0.75rem 1rem;
  font-size: 0.875rem;
  font-weight: 600;
  text-align: left;
  color: var(--color-text);
  border-bottom: 1px solid var(--color-border);
}

.data-cell {
  padding: 0.75rem 1rem;
  vertical-align: top;
  border-bottom: 1px solid var(--color-border-light);
  color: var(--color-text);
}

.clickable-cell {
  cursor: pointer;
}

.clickable-cell:hover {
  background-color: var(--color-button-hover); /* Subtle hover effect */
}

.node-label {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  background-color: var(--color-primary);
  color: var(--color-text-light);
  border-radius: 0.25rem;
  font-size: 0.75rem;
  font-weight: 500;
  margin-bottom: 0.5rem;
}

.rel-type {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  background-color: var(--color-accent);
  color: var(--color-text-light);
  border-radius: 0.25rem;
  font-size: 0.75rem;
  font-weight: 500;
  margin-bottom: 0.5rem;
}

.path-info {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  font-style: italic;
}

.props-container {
  max-height: 200px;
  overflow-y: auto;
  border-radius: 0.25rem;
  border: 1px solid var(--color-border);
}

.props-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.75rem;
}

.props-header {
  background-color: var(--color-sidebar-bg);
  position: sticky;
  top: 0;
  z-index: 5;
}

.prop-header-cell {
  padding: 0.5rem;
  font-weight: 500;
  text-align: left;
  color: var(--color-text);
  border-bottom: 1px solid var(--color-border);
}

.prop-name-cell {
  padding: 0.5rem;
  font-weight: 500;
  border-bottom: 1px solid var(--color-border-light);
  color: var(--color-text-muted);
  width: 40%;
}

.prop-value-cell {
  padding: 0.5rem;
  border-bottom: 1px solid var(--color-border-light);
  color: var(--color-text);
  word-break: break-all;
}

.no-results {
  padding: 2rem;
}

.empty-state-message {
  text-align: center;
  padding: 2rem;
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

.pagination-controls {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 1rem 0;
  gap: 0.5rem;
}

.pagination-button {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 1.75rem;
  width: 1.75rem;
  border-radius: 0.375rem;
  background-color: var(--color-button-bg);
  border: 1px solid var(--color-border);
  color: var(--color-text);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s;
}

.pagination-button:disabled {
  color: var(--color-text-muted);
  background-color: var(--color-button-hover);
  cursor: not-allowed;
}

.pagination-button:not(:disabled):hover {
  background-color: var(--color-button-hover);
}

.page-indicator {
  display: flex;
  align-items: center;
  min-width: 3rem;
  justify-content: center;
  font-size: 0.875rem;
  color: var(--color-text);
}

.page-input {
  width: 3rem;
  padding: 0.25rem;
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  text-align: center;
  background-color: var(--color-input-bg);
  color: var(--color-text);
}

.page-number {
  cursor: pointer;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
}

.page-number:hover {
  background-color: var(--color-button-hover);
}

.page-divider {
  margin: 0 0.25rem;
}

.copy-success-notification {
  position: fixed;
  bottom: 1rem;
  right: 1rem;
  background-color: var(--color-success);
  color: var(--color-text-light);
  padding: 0.5rem 1rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  font-weight: 500;
  z-index: 1000;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

/* Transition for fade in/out */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.5s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.sortable-header {
  cursor: pointer;
  position: relative; /* Needed for pseudo-element positioning */
  padding-right: 1.5rem; /* Space for sort arrow */
}

.sortable-header::after {
  content: '';
  position: absolute;
  right: 0.5rem;
  top: 50%;
  transform: translateY(-50%);
  width: 0;
  height: 0;
  border-left: 4px solid transparent;
  border-right: 4px solid transparent;
  opacity: 0.5; /* Dim arrow when not active */
}

.sortable-header.sorted-asc::after {
  border-bottom: 5px solid var(--color-text); /* Up arrow */
  opacity: 1;
}

.sortable-header.sorted-desc::after {
  border-top: 5px solid var(--color-text); /* Down arrow */
  opacity: 1;
}
</style> 