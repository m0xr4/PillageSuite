<template>
  <div class="credential-gathering">
    <!-- Sidebar Operations List -->
    <div v-if="!mainContentOnly" class="operations-list">
      <div 
        v-for="(operation, index) in operations" 
        :key="index"
        class="operation-item"
        :class="{ 'active': selectedOperation === index }"
        @click="$emit('select', index)"
      >
        <div class="operation-icon">
          <svg xmlns="http://www.w3.org/2000/svg" class="icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd" />
          </svg>
        </div>
        <span class="operation-name">{{ operation.name }}</span>
      </div>
    </div>

    <!-- Main Content Area -->
    <div v-if="mainContentOnly" class="operation-content">
      <div v-if="selectedOperation === null" class="empty-state">
        <div class="empty-state-icon-container">
          <svg xmlns="http://www.w3.org/2000/svg" class="empty-state-icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd" />
          </svg>
        </div>
        <h2 class="empty-state-title">Select a Credential Gathering Operation</h2>
        <p class="empty-state-description">Choose a credential gathering operation from the sidebar to configure and run it.</p>
      </div>

      <!-- Placeholder for future operations -->
      <div v-else class="operation-details">
        <div class="operation-header">
          <h2>{{ operations[selectedOperation]?.name || 'Credential Gathering' }}</h2>
          <p class="operation-description">{{ operations[selectedOperation]?.description || 'Credential gathering operations will be implemented here.' }}</p>
        </div>

        <div class="operation-config">
          <h3>Configuration</h3>
          <div class="config-form">
            <div class="config-item">
              <label>File list</label>
              <div class="input-group">
                <input 
                  type="text" 
                  v-model="config.file_list" 
                  placeholder="Select file containing file paths" 
                  :disabled="isRunning"
                >
                <button 
                  @click="selectTargetFile" 
                  class="file-button"
                  :disabled="isRunning"
                  title="Select file containing file paths"
                >
                  📁
                </button>
              </div>
              <small class="help-text">Select a file containing file paths (one per line)</small>
            </div>

            <div class="config-item">
              <label>String list</label>
              <div class="input-group">
                <input 
                  type="text" 
                  v-model="config.string_list" 
                  placeholder="Select file containing strings to search for" 
                  :disabled="isRunning"
                >
                <button 
                  @click="selectStringFile" 
                  class="file-button"
                  :disabled="isRunning"
                  title="Select file containing strings to search for"
                >
                  📁
                </button>
              </div>
              <small class="help-text">Select a file containing strings to search for (one per line)</small>
            </div>
            
            <div class="config-item">
              <label>
                <input 
                  type="checkbox" 
                  v-model="config.debug_mode"
                  :disabled="isRunning"
                >
                Debug Mode
              </label>
              <small class="help-text">Enable detailed logging output</small>
            </div>
          </div>
        </div>

        <!-- Todo: add function to create file list - this will run a cypher query and save the output in a text file in current directory-->
        <div class="operation-actions">
          <button 
            @click="runCreateFileList" 
            class="run-button"
            :disabled="isRunning"
            :class="{ 'running': isRunning }"
          >
            {{ isRunning ? 'Gathering...' : 'Create File List' }}
          </button>
          <div class="help-text" style="margin-top: 0.5rem;">
            Generates a text file of paths of files, potentially containing passwords, from Neo4j.
          </div>
          <div class="toggle-inline" style="margin-top: 0.5rem;">
            <span class="toggle-text">Edit builtin query</span>
            <button 
              class="toggle-switch-btn" 
              :class="{ on: editFileQueryEnabled }"
              @click="editFileQueryEnabled = !editFileQueryEnabled"
              :disabled="isRunning"
              title="Toggle query edit"
            >
              <span class="knob"></span>
            </button>
          </div>
          <div v-if="editFileQueryEnabled" class="config-item">
            <textarea
              v-model="fileQueryText"
              rows="3"
              :disabled="isRunning"
              class="query-textarea"
            ></textarea>
            <small class="help-text">Return the path column as <strong>path</strong> (e.g. RETURN n.full_path AS path).</small>
          </div>
        </div>

        <!-- Todo: add function to create string list - this will run a cypher query and save the output in a text file in current directory-->
        <div class="operation-actions">
          <button 
            @click="runCreateStringList" 
            class="run-button"
            :disabled="isRunning"
            :class="{ 'running': isRunning }"
          >
            {{ isRunning ? 'Gathering...' : 'Create String List' }}
          </button>
          <div class="help-text" style="margin-top: 0.5rem;">
            Builds a text file of usernames from Neo4j plus common keywords used during searches.
          </div>
          <div class="toggle-inline" style="margin-top: 0.5rem;">
            <span class="toggle-text">Edit builtin query</span>
            <button 
              class="toggle-switch-btn" 
              :class="{ on: editStringQueryEnabled }"
              @click="editStringQueryEnabled = !editStringQueryEnabled"
              :disabled="isRunning"
              title="Toggle query edit"
            >
              <span class="knob"></span>
            </button>
          </div>
          <div v-if="editStringQueryEnabled" class="config-item">
            <textarea
              v-model="stringQueryText"
              rows="3"
              :disabled="isRunning"
              class="query-textarea"
            ></textarea>
            <small class="help-text">Return the name column as <strong>name</strong> (e.g. RETURN n.sam_account_name AS name).</small>
          </div>
        </div>

        <div class="operation-actions">
          <button 
            @click="runCredentialGathering" 
            class="run-button"
            :disabled="isRunning || !config.file_list.trim() || !config.string_list.trim()"
            :class="{ 'running': isRunning }"
          >
            {{ isRunning ? 'Gathering...' : 'Run Credential Gathering' }}
          </button>
          <button
            v-if="isRunning"
            @click="abortCredentialGathering"
            class="abort-button"
            :disabled="abortRequested"
          >
            {{ abortRequested ? 'Aborting…' : 'Abort Gathering' }}
          </button>
        </div>

        <div class="operation-results">
          <h3>Results</h3>
          <div v-if="!hasResults && !isRunning" class="results-placeholder">
            Operation results will appear here
          </div>
          <div v-else class="results-content">
            <!-- Progress Bar -->
            <div v-if="isRunning && progress" class="progress-container">
              <div class="progress-info">
                <span class="progress-stage">{{ progress.stage.charAt(0).toUpperCase() + progress.stage.slice(1) }}</span>
                <span class="progress-message">{{ progress.message }}</span>
              </div>
              <div class="progress-bar">
                <div 
                  class="progress-fill" 
                  :style="{ width: progressPercentage + '%' }"
                ></div>
              </div>
              <div class="progress-text">
                {{ progress.current }}{{ progress.total ? '/' + progress.total : '' }}
              </div>
            </div>

            <!-- Log Output -->
            <div class="log-output" ref="logOutput">
              <div 
                v-for="(log, index) in logs" 
                :key="index"
                class="log-entry"
                :class="{ 'error': log.includes('Failed') || log.includes('Error') }"
              >
                {{ log }}
              </div>
            </div>

            <!-- Final Result -->
            <div v-if="result" class="final-result" :class="{ 'success': result.success, 'error': !result.success }">
              <h4>{{ result.success ? 'Success' : 'Error' }}</h4>
              <p>{{ result.message }}</p>
              <div v-if="result.success && result.output_file" class="output-info">
                <p><strong>Output file:</strong> {{ result.output_file }}</p>
                <p><strong>Total lines with hits:</strong> {{ result.total_entries }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { executeCypherQuery } from '../services/neo4jService'

const props = defineProps({
  mainContentOnly: {
    type: Boolean,
    default: false
  },
  selectedOperation: {
    type: Number,
    default: null
  }
});

const emit = defineEmits(['select']);

// Define available operations
const operations = [
  {
    name: 'Search Strings',
    description: 'Search for credentials in files'
  }
];

// Configuration state
const config = ref({
  file_list: '',
  string_list: '',
  debug_mode: false
});

// Query edit toggles and text
const editFileQueryEnabled = ref(false);
const editStringQueryEnabled = ref(false);
const fileQueryText = ref("MATCH (n:file) WHERE n.extension IN ['cmd', 'bat', 'ps1', 'vbs', 'hta', 'com', 'config', 'cfg', 'conf', 'ini', 'json', 'xml', 'yaml', 'yml', 'toml'] AND n.size < 5242880 RETURN n.full_path AS path");
const stringQueryText = ref('MATCH (n:User) WHERE size(n.sam_account_name) >= 3 RETURN n.sam_account_name AS name');

// Execution state
const isRunning = ref(false);
const abortRequested = ref(false);
const progress = ref(null);
const logs = ref([]);
const result = ref(null);
const hasResults = computed(() => logs.value.length > 0 || result.value !== null);

// Progress percentage computation
const progressPercentage = computed(() => {
  if (!progress.value || !progress.value.total) return 0;
  return Math.min(100, (progress.value.current / progress.value.total) * 100);
});

// Event listeners
let progressUnlisten = null;
let logUnlisten = null;

// File selection methods
const selectTargetFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Text files',
        extensions: ['txt', 'csv']
      }]
    });
    
    if (selected) {
      config.value.file_list = selected;
    }
  } catch (error) {
    console.error('Error selecting file:', error);
  }
};

const selectStringFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Text files',
        extensions: ['txt', 'csv']
      }]
    });
    
    if (selected) {
      config.value.string_list = selected;
    }
  } catch (error) {
    console.error('Error selecting file:', error);
  }
};

// Execution methods
const runCredentialGathering = async () => {
  if (isRunning.value) return;
  
  clearResults();
  isRunning.value = true;
  abortRequested.value = false;
  
  try {
    const cgConfig = {
      file_list: config.value.file_list,
      string_list: config.value.string_list,
      debug_mode: config.value.debug_mode
    };

    const response = await invoke('start_credential_gathering', { config: cgConfig });
    result.value = response;
    
  } catch (error) {
    console.error('Error running credential gathering:', error);
    result.value = {
      success: false,
      message: error.toString(),
      output_file: '',
      total_entries: 0,
      errors: [error.toString()]
    };
  } finally {
    isRunning.value = false;
  }
};

const abortCredentialGathering = async () => {
  if (!isRunning.value || abortRequested.value) return;
  abortRequested.value = true;
  try {
    await invoke('abort_credential_gathering');
  } catch (error) {
    console.error('Error aborting credential gathering:', error);
    abortRequested.value = false; // allow retry if it failed
  }
};

// Create File List from Neo4j query results
const runCreateFileList = async () => {
  if (isRunning.value) return;

  clearResults();
  isRunning.value = true;

  try {
    logs.value.push('Running Cypher to fetch file paths...');
    // Use label 'file' (as used by importer and index) and alias the return value
    const cypher = editFileQueryEnabled.value ? fileQueryText.value : "MATCH (n:file) WHERE n.extension IN ['cmd', 'bat', 'ps1', 'vbs', 'hta', 'com', 'config', 'cfg', 'conf', 'ini', 'json', 'xml', 'yaml', 'yml', 'toml'] AND n.size < 5242880 RETURN n.full_path AS path";
    const result = await executeCypherQuery(cypher);

    const paths = (result?.records || [])
      .map((r) => {
        try { return r.get('path'); } catch { return null; }
      })
      .filter((p) => typeof p === 'string' && p.length > 0);

    if (paths.length === 0) {
      logs.value.push('No matching file paths found.');
      return;
    }

    const pad = (n) => (n < 10 ? '0' + n : '' + n);
    const d = new Date();
    const ts = `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}_${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`;
    const filename = `.\\file_list_${ts}.txt`;

    await writeTextFile(filename, paths.join('\n') + '\n');
    logs.value.push(`Wrote ${paths.length} paths to ${filename}`);
  } catch (error) {
    console.error('Create File List error:', error);
    logs.value.push(`Error creating file list: ${error?.message || error}`);
  } finally {
    isRunning.value = false;
    scrollLogToBottom();
  }
};

// Create String List from Neo4j query results + predefined placeholders
const runCreateStringList = async () => {
  if (isRunning.value) return;

  clearResults();
  isRunning.value = true;

  try {
    logs.value.push('Running Cypher to fetch Strings...');
    const cypher = editStringQueryEnabled.value ? stringQueryText.value : 'MATCH (n:User) WHERE size(n.sam_account_name) >= 3 RETURN n.sam_account_name AS name';
    const result = await executeCypherQuery(cypher);

    const names = (result?.records || [])
      .map((r) => {
        try { return r.get('name'); } catch { return null; }
      })
      .filter((v) => typeof v === 'string' && v.length > 0);

    // strings to include
    const placeholders = ['passw', 'key', 'net use', 'runas /user:'];
    const lines = [...placeholders, ...names];

    if (lines.length === 0) {
      logs.value.push('No strings to write.');
      return;
    }

    const pad = (n) => (n < 10 ? '0' + n : '' + n);
    const d = new Date();
    const ts = `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}_${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`;
    const filename = `.\\string_list_${ts}.txt`;

    await writeTextFile(filename, lines.join('\n') + '\n');
    logs.value.push(`Wrote ${lines.length} strings to ${filename}`);
  } catch (error) {
    console.error('Create String List error:', error);
    logs.value.push(`Error creating string list: ${error?.message || error}`);
  } finally {
    isRunning.value = false;
    scrollLogToBottom();
  }
};

// Utility methods
const clearResults = () => {
  progress.value = null;
  logs.value = [];
  result.value = null;
};

const scrollLogToBottom = () => {
  nextTick(() => {
    const logOutput = document.querySelector('.log-output');
    if (logOutput) {
      logOutput.scrollTop = logOutput.scrollHeight;
    }
  });
};

// Lifecycle
onMounted(async () => {
  progressUnlisten = await listen('credential-gathering-progress', (event) => {
    progress.value = event.payload;
  });
  logUnlisten = await listen('credential-gathering-log', (event) => {
    logs.value.push(event.payload);
    scrollLogToBottom();
  });
});

onUnmounted(() => {
  if (progressUnlisten) progressUnlisten();
  if (logUnlisten) logUnlisten();
});
</script>

<style scoped>
.credential-gathering {
  height: 100%;
  background-color: var(--color-content-bg);
}

.operations-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.operation-item {
  display: flex;
  align-items: center;
  padding: 0.75rem;
  border-radius: 0.375rem;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-text);
  background-color: var(--color-card-bg);
}

.operation-item:hover {
  background-color: var(--color-button-hover);
}

.operation-item.active {
  background-color: var(--color-primary);
  color: white;
}

.operation-icon {
  display: flex;
  align-items: center;
  margin-right: 0.75rem;
}

.operation-icon .icon {
  width: 1.25rem;
  height: 1.25rem;
}

.operation-name {
  font-size: 0.875rem;
  font-weight: 500;
}

.operation-content {
  padding: 1.5rem;
  height: 100%;
  overflow-y: auto;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 2rem;
  text-align: center;
}

.empty-state-icon-container {
  width: 4rem;
  height: 4rem;
  border-radius: 9999px;
  background-color: rgba(var(--color-info-rgb, 59, 130, 246), 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 1rem;
}

.empty-state-icon {
  width: 2.5rem;
  height: 2.5rem;
  color: var(--color-info);
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
}

.operation-details {
  max-width: 800px;
  margin: 0 auto;
}

.operation-header {
  margin-bottom: 2rem;
}

.operation-header h2 {
  color: var(--color-text);
  margin-bottom: 0.5rem;
}

.operation-description {
  color: var(--color-text-muted);
}

.operation-config {
  background-color: var(--color-card-bg);
  border-radius: 0.5rem;
  padding: 1.5rem;
  margin-bottom: 1.5rem;
}

.operation-config h3 {
  color: var(--color-text);
  margin-bottom: 1rem;
}

.config-form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.config-item {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.config-item label {
  color: var(--color-text);
  font-size: 0.875rem;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.input-group {
  display: flex;
  gap: 0.5rem;
}

.config-item input[type="text"],
.config-item input[type="password"],
.config-item input[type="number"] {
  flex: 1;
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  background-color: var(--color-input-bg);
  color: var(--color-text);
}

.config-item input[type="checkbox"] {
  margin: 0;
}

.config-item input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.file-button {
  padding: 0.5rem;
  background-color: var(--color-card-bg);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

.file-button:hover:not(:disabled) {
  background-color: var(--color-button-hover);
}

.file-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.help-text {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin-top: 0.25rem;
}

.query-textarea {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  background-color: var(--color-input-bg);
  color: var(--color-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.toggle-inline {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.toggle-text {
  color: var(--color-text);
  font-size: 0.875rem;
}

.toggle-switch-btn {
  position: relative;
  width: 38px;
  height: 23px;
  margin-bottom: 2px;
  border-radius: 9999px;
  border: 1px solid var(--color-border);
  background-color: var(--color-input-bg);
  cursor: pointer;
  transition: background-color 0.2s, border-color 0.2s;
}

.toggle-switch-btn .knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 9999px;
  background-color: var(--color-card-bg);
  transition: left 0.2s;
}

.toggle-switch-btn.on {
  background-color: var(--color-primary);
  border-color: var(--color-primary);
}

.toggle-switch-btn.on .knob {
  left: 18px;
  background-color: #fff;
}

.operation-actions {
  margin-bottom: 1.5rem;
  padding-left: 1.5rem;
}

.run-button {
  padding: 0.75rem 1.5rem;
  background-color: var(--color-primary);
  color: var(--color-text);
  border: none;
  border-radius: 0.375rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  min-width: 150px;
}

.run-button:hover:not(:disabled) {
  background-color: var(--color-primary-hover);
}

.run-button:disabled {
  background-color: var(--color-disabled);
  cursor: not-allowed;
}

.run-button.running {
  background-color: var(--color-warning);
}

.abort-button {
  margin-left: 0.75rem;
  padding: 0.75rem 1.5rem;
  background-color: var(--color-error);
  color: #fff;
  border: none;
  border-radius: 0.375rem;
  font-weight: 500;
}

.abort-button:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.operation-results {
  background-color: var(--color-card-bg);
  border-radius: 0.5rem;
  padding: 1.5rem;
}

.operation-results h3 {
  color: var(--color-text);
  margin-bottom: 1rem;
}

.results-placeholder {
  color: var(--color-text-muted);
  text-align: center;
  padding: 2rem;
  border: 1px dashed var(--color-border);
  border-radius: 0.375rem;
}

.progress-container {
  margin-bottom: 1rem;
  padding: 1rem;
  background-color: var(--color-content-bg);
  border-radius: 0.375rem;
  border: 1px solid var(--color-border);
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.progress-stage {
  font-weight: 600;
  color: var(--color-primary);
  text-transform: capitalize;
}

.progress-message {
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.progress-bar {
  height: 8px;
  background-color: var(--color-border);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 0.5rem;
}

.progress-fill {
  height: 100%;
  background-color: var(--color-primary);
  transition: width 0.3s ease;
}

.progress-text {
  text-align: center;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.log-output {
  max-height: 300px;
  overflow-y: auto;
  background-color: #1e1e1e;
  color: #d4d4d4;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  padding: 1rem;
  border-radius: 0.375rem;
  margin-bottom: 1rem;
}

.log-entry {
  margin-bottom: 0.25rem;
  word-wrap: break-word;
}

.log-entry.error {
  color: #f85149;
}

.final-result {
  padding: 1rem;
  border-radius: 0.375rem;
  border: 1px solid;
}

.final-result.success {
  background-color: rgba(34, 197, 94, 0.1);
  border-color: #22c55e;
  color: #15803d;
}

.final-result.error {
  background-color: rgba(239, 68, 68, 0.1);
  border-color: #ef4444;
  color: #dc2626;
}

.final-result h4 {
  margin: 0 0 0.5rem 0;
  font-size: 1rem;
}

.final-result p {
  margin: 0 0 1rem 0;
}

.output-info p {
  margin: 0.25rem 0;
  font-size: 0.875rem;
}
</style> 