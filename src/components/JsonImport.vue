<template>
  <div class="card">
    <h2 class="card-title">Data Import</h2>
    
    <div class="import-container">
      <div class="form-group">
        <label class="form-label">Select Import File</label>
        <div class="file-selection">
          <button 
            @click="openFileDialog" 
            class="button-secondary"
            :disabled="!isConnected || isLoading"
          >
            Browse...
          </button>
          <span class="file-path" v-if="selectedFilePath">{{ selectedFilePath }}</span>
          <span class="file-path placeholder" v-else>No file selected</span>
        </div>
      </div>
      
      <div v-if="selectedFile" class="file-info">
        <div class="file-details">
          <span class="file-name">Selected file: {{ selectedFile.name }}</span>
          <span class="file-size">{{ formatFileSize(selectedFile.size) }}</span>
        </div>
      </div>
      
      <div class="form-group">
        <label class="form-label">Import Mode</label>
        <div class="import-mode-options">
          <label class="custom-radio" :class="{ 'disabled': isLoading }">
            <input 
              type="radio" 
              id="mode-domain" 
              value="domain" 
              v-model="importMode"
              :disabled="isLoading"
            />
            <span class="radio-control"></span>
            <span class="radio-label">Import Domain Info</span>
          </label>
          
          <label class="custom-radio" :class="{ 'disabled': isLoading }">
            <input 
              type="radio" 
              id="mode-shares" 
              value="shares" 
              v-model="importMode"
              :disabled="isLoading"
            />
            <span class="radio-control"></span>
            <span class="radio-label">Import SharesIndex</span>
          </label>
          
          <div v-if="importMode === 'shares'" class="share-options">
            <label class="custom-checkbox">
              <input 
                type="checkbox" 
                v-model="isInitialShareImport"
                :disabled="isLoading"
              />
              <span class="checkbox-control"></span>
              <span class="checkbox-label">Initial Import Mode</span>
            </label>
          </div>
        </div>
      </div>
      
      <div v-if="isLoading" class="progress-container">
        <div class="progress-bar">
          <div 
            class="progress-fill" 
            :style="{ width: `${progressPercentage}%` }"
          ></div>
        </div>
        <div class="progress-stats">
          <div class="progress-stat-line">{{ batchInfo }}</div>
          <div class="progress-stat-line">{{ nodeInfo }}</div>
        </div>
      </div>
      
      <div class="action-bar">
        <button
          @click="importJson"
          class="button-primary"
          :disabled="!isConnected || !selectedFilePath || isLoading"
        >
          {{ isLoading ? 'Importing...' : 'Start Import' }}
        </button>
        <div v-if="importStatus" :class="['status-message', importStatusClass]">
          {{ importStatus }}
        </div>
      </div>
      
      <div class="builtin-action">
        <button
          @click="createBuiltinIdentities"
          class="button-builtin"
          :disabled="!isConnected || isLoading"
        >
          Create Builtin Identities
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';

// Import Tauri APIs properly
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { getConnectionDetails as getNeo4jConnectionDetails, executeCypherQuery } from '../services/neo4jService';

// Setup progress listener
onMounted(async () => {
  try {
    const unlisten = await listen('import-progress', (event) => {
      console.debug('Received progress event:', event.payload);
      // Ensure we're getting the correct data structure
      if (event.payload) {
        progress.value = {
          nodesImported: event.payload.nodes_imported || 0,
          batchesProcessed: event.payload.batches_processed || 0,
          totalBatches: event.payload.total_batches || 0,
          percentageComplete: event.payload.percentage_complete || 0,
          elapsedSeconds: event.payload.elapsed_seconds || 0
        };
        console.debug('Updated progress:', progress.value);
      }
    });
    
    onUnmounted(() => {
      if (unlisten) unlisten();
    });
  } catch (err) {
    console.warn('Error setting up event listener:', err);
  }
});

const props = defineProps({
  session: Object,
  isConnected: Boolean
});

const selectedFile = ref(null);
const selectedFilePath = ref('');
const isLoading = ref(false);
const importStatus = ref('');
const importStatusClass = ref('status-neutral');
const importMode = ref('domain');
const isInitialShareImport = ref(true);

// Progress tracking with explicit type
const progress = ref({
  nodesImported: 0,
  batchesProcessed: 0,
  totalBatches: 0,
  percentageComplete: 0,
  elapsedSeconds: 0
});

// Make computed properties more robust
const progressPercentage = computed(() => {
  const percentage = progress.value.percentageComplete || 0;
  console.debug('Progress percentage:', percentage);
  return percentage;
});

const batchInfo = computed(() => {
  const info = `Batch ${progress.value.batchesProcessed || 0}/${progress.value.totalBatches || 0}`;
  console.debug('Batch info:', info);
  return info;
});

const nodeInfo = computed(() => {
  const info = `${progress.value.nodesImported || 0} nodes imported`;
  console.debug('Node info:', info);
  return info;
});

// Open file dialog
const openFileDialog = async () => {
  try {
    // Allow different file types depending on import mode
    const filters = importMode.value === 'domain' 
      ? [{ name: 'ZIP Archive', extensions: ['zip'] }] 
      : [{ name: 'JSON', extensions: ['json','jsonl'] }];
    
    const filePath = await open({
      multiple: false,
      filters: filters
    });
    
    if (!filePath) return;
    
    selectedFilePath.value = filePath;
    
    // Just get basic file info from the path
    try {
      const filename = filePath.split('/').pop().split('\\').pop();
      const metadata = await invoke("get_file_info", { filePath });
      
      selectedFile.value = {
        name: filename,
        size: metadata.size || 0,
      };
      
      importStatus.value = '';
    } catch (error) {
      console.error('File info error:', error);
      importStatus.value = 'Could not read file information';
      importStatusClass.value = 'status-error';
    }
  } catch (error) {
    console.error('File dialog error:', error);
    importStatus.value = `Error: ${error.message}`;
    importStatusClass.value = 'status-error';
  }
};

// Format file size
const formatFileSize = (bytes) => {
  if (bytes === 0) return '0 Bytes';
  
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// Get Neo4j connection details from the service
const getConnectionDetails = () => {
  if (!props.isConnected || !props.session) {
    throw new Error('No active Neo4j session');
  }
  
  const connectionInfo = getNeo4jConnectionDetails();
  
  if (!connectionInfo) {
    throw new Error('Neo4j connection details not available');
  }
  
  console.debug('Neo4j connection info:', { 
    uri: connectionInfo.uri,
    username: connectionInfo.username,
    hasPassword: !!connectionInfo.password
  });
  
  return connectionInfo;
};

// Create builtin identities directly in Neo4j
const createBuiltinIdentities = async () => {
  if (!props.isConnected) {
    importStatus.value = 'Not connected to Neo4j';
    importStatusClass.value = 'status-error';
    return;
  }
  
  try {
    isLoading.value = true;
    importStatus.value = 'Creating built-in identities...';
    importStatusClass.value = 'status-info';
    
    // The built-in identities query
    const builtinSidsQuery = `
    UNWIND [
      {sid: "S-1-5-18", name: "NT AUTHORITY\\SYSTEM"},
      {sid: "S-1-5-19", name: "NT AUTHORITY\\LOCAL SERVICE"},
      {sid: "S-1-5-20", name: "NT AUTHORITY\\NETWORK SERVICE"},
      {sid: "S-1-5-32-544", name: "BUILTIN\\Administrators"},
      {sid: "S-1-5-32-545", name: "BUILTIN\\Users"},
      {sid: "S-1-5-32-546", name: "BUILTIN\\Guests"},
      {sid: "S-1-5-32-547", name: "BUILTIN\\Power Users"},
      {sid: "S-1-5-32-548", name: "BUILTIN\\Account Operators"},
      {sid: "S-1-5-32-549", name: "BUILTIN\\Server Operators"},
      {sid: "S-1-5-32-550", name: "BUILTIN\\Print Operators"},
      {sid: "S-1-5-32-551", name: "BUILTIN\\Backup Operators"},
      {sid: "S-1-5-32-552", name: "BUILTIN\\Replicators"},
      {sid: "S-1-5-32-554", name: "BUILTIN\\Pre-Windows 2000 Compatible Access"},
      {sid: "S-1-5-32-555", name: "BUILTIN\\Remote Desktop Users"},
      {sid: "S-1-5-32-556", name: "BUILTIN\\Network Configuration Operators"},
      {sid: "S-1-5-32-557", name: "BUILTIN\\Incoming Forest Trust Builders"},
      {sid: "S-1-5-32-558", name: "BUILTIN\\Performance Monitor Users"},
      {sid: "S-1-5-32-559", name: "BUILTIN\\Performance Log Users"},
      {sid: "S-1-5-32-560", name: "BUILTIN\\Windows Authorization Access Group"},
      {sid: "S-1-5-32-561", name: "BUILTIN\\Terminal Server License Servers"},
      {sid: "S-1-5-32-562", name: "BUILTIN\\Distributed COM Users"},
      {sid: "S-1-5-32-568", name: "BUILTIN\\IIS_IUSRS"},
      {sid: "S-1-5-32-569", name: "BUILTIN\\Cryptographic Operators"},
      {sid: "S-1-5-32-573", name: "BUILTIN\\Event Log Readers"},
      {sid: "S-1-5-32-574", name: "BUILTIN\\Certificate Service DCOM Access"},
      {sid: "S-1-5-32-575", name: "BUILTIN\\RDS Remote Access Servers"},
      {sid: "S-1-5-32-576", name: "BUILTIN\\RDS Endpoint Servers"},
      {sid: "S-1-5-32-577", name: "BUILTIN\\RDS Management Servers"},
      {sid: "S-1-5-32-578", name: "BUILTIN\\Hyper-V Administrators"},
      {sid: "S-1-5-32-579", name: "BUILTIN\\Access Control Assistance Operators"},
      {sid: "S-1-5-32-580", name: "BUILTIN\\Remote Management Users"},
      {sid: "S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464", name: "NT SERVICE\\TrustedInstaller"},
      {sid: "S-1-5-32-583", name: "BUILTIN\\Device Owners"},
      {sid: "S-1-15-2-1", name: "ALL APPLICATION PACKAGES"},
      {sid: "S-1-15-2-2", name: "ALL RESTRICTED APPLICATION PACKAGES"},
      {sid: "S-1-1-0", name: "Everyone"},
      {sid: "S-1-5-11", name: "Authenticated Users"},
      {sid: "S-1-5-2", name: "NETWORK"},
      {sid: "S-1-5-4", name: "INTERACTIVE"},
      {sid: "S-1-5-6", name: "SERVICE"},
      {sid: "S-1-5-7", name: "ANONYMOUS"},
      {sid: "S-1-5-9", name: "ENTERPRISE DOMAIN CONTROLLERS"},
      {sid: "S-1-5-10", name: "Principal Self"},
      {sid: "S-1-3-0", name: "CREATOR OWNER"},
      {sid: "S-1-3-1", name: "CREATOR GROUP"},
      {sid: "S-1-5-32-553", name: "BUILTIN\\Backup Operators"}
    ] AS item
    MERGE (n:Identity {sid: item.sid})
    SET n.name = item.name
    RETURN count(n) AS count`;

    // Use the neo4j service to execute the query
    const startTime = performance.now();
    const result = await executeCypherQuery(builtinSidsQuery);
    const endTime = performance.now();
    
    console.debug('Built-in identities creation result:', result);
    
    // Extract the count from the result safely
    let count;
    if (result.records && result.records.length > 0) {
      const countValue = result.records[0].get('count');
      // Handle different possible return types
      if (typeof countValue === 'object' && countValue !== null && typeof countValue.toNumber === 'function') {
        count = countValue.toNumber();
      } else if (typeof countValue === 'number') {
        count = countValue;
      } else if (typeof countValue === 'string') {
        count = parseInt(countValue, 10);
      } else {
        // Fallback if we can't determine count
        count = 'unknown number of';
      }
    } else {
      count = 'unknown number of';
    }
    
    const elapsedSeconds = ((endTime - startTime) / 1000).toFixed(2);
    
    // Update status with results
    importStatus.value = `Successfully created ${count} built-in identities in ${elapsedSeconds} seconds`;
    importStatusClass.value = 'status-success';
    
    // Emit success event
    emit('importSuccess', {
      count: typeof count === 'number' ? count : 0,
      time: parseFloat(elapsedSeconds),
      mode: 'builtin'
    });
  } catch (error) {
    console.error('Built-in identities creation error:', error);
    importStatus.value = `Error creating built-in identities: ${error.message}`;
    importStatusClass.value = 'status-error';
    
    // Emit error event
    emit('importError', error);
  } finally {
    isLoading.value = false;
  }
};

// Import JSON data to Neo4j
const importJson = async () => {
  if (!props.isConnected || !props.session || !selectedFilePath.value) {
    importStatus.value = 'Not connected or no file selected';
    importStatusClass.value = 'status-error';
    return;
  }
  
  try {
    isLoading.value = true;
    importStatus.value = 'Processing file...';
    importStatusClass.value = 'status-info';
    
    // Reset progress
    progress.value = {
      nodesImported: 0,
      batchesProcessed: 0,
      totalBatches: 0,
      percentageComplete: 0,
      elapsedSeconds: 0
    };
    
    // Get connection details
    const connection = getConnectionDetails();
    
    // Use the frontend mode directly - backend now supports 'domain'
    const backendImportMode = importMode.value;
    
    // Log parameters being sent to Rust (without sensitive info)
    console.debug('Sending import request to Rust backend:', {
      filePath: selectedFilePath.value,
      neo4jUri: connection.uri,
      neo4jUser: connection.username,
      passwordProvided: !!connection.password,
      batchSize: 1000,
      importMode: importMode.value,
      backendImportMode: backendImportMode,
      isInitialShareImport: importMode.value === 'shares' ? isInitialShareImport.value : true
    });
    
    // Call Rust function with import mode
    const result = await invoke('import_json_to_neo4j', {
      filePath: selectedFilePath.value,
      neo4jUri: connection.uri,
      neo4jUser: connection.username,
      neo4jPassword: connection.password,
      batchSize: 1000,
      importMode: backendImportMode,
      isInitialShareImport: importMode.value === 'shares' ? isInitialShareImport.value : true
    });
    
    // Log the full result for debugging - also stringify to check raw JSON format
    console.debug('Import result from Rust backend:', result);
    console.debug('Result as JSON string:', JSON.stringify(result));
    
    // Update status with results
    const elapsedTime = result.elapsed_seconds !== undefined && result.elapsed_seconds !== null
      ? result.elapsed_seconds.toFixed(2)
      : '0.00';
    const nodesCount = result.nodes_imported || result.nodesImported || 0;
    
    console.debug(`Setting success message with ${nodesCount} nodes and ${elapsedTime} seconds`);
    
    importStatus.value = `Import finished!`;
    importStatusClass.value = 'status-success';
    
    // Emit success event
    emit('importSuccess', {
      count: result.nodes_imported || result.nodesImported || 0,
      time: result.elapsed_seconds || 0,
      mode: importMode.value
    });
  } catch (error) {
    console.error('Import error:', error);
    
    // Handle different types of errors more specifically
    let errorMessage = `Import failed: ${error}`;
    
    // Check for authentication errors
    if (error.toString().toLowerCase().includes('auth') || 
        error.toString().toLowerCase().includes('unauthorized') ||
        error.toString().toLowerCase().includes('password') ||
        error.toString().toLowerCase().includes('credentials')) {
      errorMessage = 'Neo4j authentication failed. Please check your connection and try again.';
    }
    // Check for connection errors
    else if (error.toString().toLowerCase().includes('connect') || 
             error.toString().toLowerCase().includes('network') ||
             error.toString().toLowerCase().includes('unreachable')) {
      errorMessage = 'Failed to connect to Neo4j database. Please check your connection settings.';
    }
    // Check for file errors
    else if (error.toString().toLowerCase().includes('file') || 
             error.toString().toLowerCase().includes('path') ||
             error.toString().toLowerCase().includes('not found')) {
      errorMessage = 'File error: The selected file could not be processed.';
    }
    
    importStatus.value = errorMessage;
    importStatusClass.value = 'status-error';
    
    // Emit error event
    emit('importError', error);
  } finally {
    isLoading.value = false;
  }
};

// Define emits
const emit = defineEmits(['importSuccess', 'importError']);
</script>

<style scoped>
.import-container {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.form-group {
  margin-bottom: 0.5rem;
}

.form-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-muted);
  margin-bottom: 0.25rem;
}

.file-selection {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.file-path {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-path.placeholder {
  color: var(--color-text-muted);
  opacity: 0.7;
}

.button-secondary {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 0.375rem;
  background-color: var(--color-button-bg);
  color: var(--color-primary);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.button-secondary:hover {
  background-color: var(--color-button-hover);
}

.file-info {
  margin-top: 0.5rem;
}

.file-details {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.file-name {
  font-size: 0.875rem;
  color: var(--color-text);
}

.file-size {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

/* New custom radio button styles */
.import-mode-options {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  margin-top: 0.5rem;
}

.custom-radio {
  position: relative;
  display: flex;
  align-items: center;
  padding: 0.5rem;
  border-radius: 0.375rem;
  background-color: var(--color-button-bg);
  cursor: pointer;
  transition: all 0.2s ease;
}

.custom-radio:hover {
  background-color: var(--color-button-hover);
}

.custom-radio input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.radio-control {
  position: relative;
  width: 1.25rem;
  height: 1.25rem;
  margin-right: 0.75rem;
  border-radius: 50%;
  background-color: var(--color-input-bg);
  border: 2px solid var(--color-input-border);
  display: inline-block;
  transition: all 0.2s ease;
}

.custom-radio input:checked + .radio-control {
  border-color: var(--color-primary);
  border-width: 2px;
}

.custom-radio input:checked + .radio-control:after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 0.6rem;
  height: 0.6rem;
  border-radius: 50%;
  background-color: var(--color-primary);
}

.radio-label {
  font-size: 0.875rem;
  color: var(--color-text);
  font-weight: 500;
}

.custom-radio.disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.progress-container {
  margin-top: 0.5rem;
}

.progress-bar {
  width: 100%;
  height: 0.5rem;
  background-color: var(--color-input-bg);
  border-radius: 0.25rem;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background-color: var(--color-primary);
  transition: width 0.3s ease;
}

.progress-stats {
  margin-top: 0.25rem;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  text-align: right;
}

.progress-stat-line {
  margin-bottom: 0.25rem;
}

.action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 0.75rem;
}

.builtin-action {
  margin-top: 1rem;
  display: flex;
  justify-content: flex-start;
}

.button-primary {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 0.375rem;
  background-color: var(--color-button-primary-bg);
  color: var(--color-text-light);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.button-primary:hover {
  background-color: var(--color-button-primary-hover);
}

.button-primary:disabled {
  background-color: var(--color-button-primary-bg);
  opacity: 0.6;
  cursor: not-allowed;
}

.button-builtin {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 0.375rem;
  background-color: var(--color-success);
  color: var(--color-text-light);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.button-builtin:hover {
  background-color: var(--color-success);
  filter: brightness(90%);
}

.button-builtin:disabled {
  background-color: var(--color-success);
  opacity: 0.6;
  cursor: not-allowed;
}

.status-neutral {
  color: var(--color-text-muted);
  font-weight: 500;
}

.status-info {
  color: var(--color-info);
  font-weight: 500;
}

.status-success {
  color: var(--color-success);
  font-weight: 500;
}

.status-error {
  color: var(--color-error);
  font-weight: 500;
}

.status-message {
  margin-left: 1rem;
}

.share-options {
  margin-left: 2rem;
  margin-top: 0.5rem;
}

.custom-checkbox {
  position: relative;
  display: flex;
  align-items: center;
  padding: 0.5rem;
  border-radius: 0.375rem;
  background-color: var(--color-button-bg);
  cursor: pointer;
  transition: all 0.2s ease;
}

.custom-checkbox:hover {
  background-color: var(--color-button-hover);
}

.custom-checkbox input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.checkbox-control {
  position: relative;
  width: 1.25rem;
  height: 1.25rem;
  margin-right: 0.75rem;
  border-radius: 0.25rem;
  background-color: var(--color-input-bg);
  border: 2px solid var(--color-input-border);
  display: inline-block;
  transition: all 0.2s ease;
}

.custom-checkbox input:checked + .checkbox-control {
  border-color: var(--color-primary);
  background-color: var(--color-primary);
}

.custom-checkbox input:checked + .checkbox-control:after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 0.6rem;
  height: 0.6rem;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 20 20' fill='white'%3E%3Cpath fill-rule='evenodd' d='M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z' clip-rule='evenodd'/%3E%3C/svg%3E");
  background-size: contain;
  background-repeat: no-repeat;
}

.checkbox-label {
  font-size: 0.875rem;
  color: var(--color-text);
  font-weight: 500;
}
</style> 