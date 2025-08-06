<template>
  <div class="active-index">
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
            <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4zm2 6a1 1 0 011-1h6a1 1 0 110 2H7a1 1 0 01-1-1zm1 3a1 1 0 100 2h6a1 1 0 100-2H7z" clip-rule="evenodd" />
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
            <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4zm2 6a1 1 0 011-1h6a1 1 0 110 2H7a1 1 0 01-1-1zm1 3a1 1 0 100 2h6a1 1 0 100-2H7z" clip-rule="evenodd" />
          </svg>
        </div>
        <h2 class="empty-state-title">Select an Operation</h2>
        <p class="empty-state-description">Choose an indexing operation from the sidebar to configure and run it.</p>
      </div>

      <!-- Share Enumeration Operation -->
      <div v-else-if="selectedOperation === 0" class="operation-details">
        <div class="operation-header">
          <h2>{{ operations[0].name }}</h2>
          <p class="operation-description">{{ operations[0].description }}</p>
        </div>

        <div class="operation-config">
          <h3>Configuration</h3>
          <div class="config-form">
            <div class="config-item">
              <label>Target Hosts</label>
              <div class="input-group">
                <input 
                  type="text" 
                  v-model="config.targets" 
                  placeholder="Enter comma-separated hostnames (e.g., DC01,FILE01,SHARE01)" 
                  :disabled="isRunning"
                >
                <button 
                  @click="selectHostsFile" 
                  class="file-button"
                  :disabled="isRunning"
                  title="Select file containing hostnames"
                >
                  📁
                </button>
              </div>
              <small class="help-text">Enter hostnames separated by commas, or select a file containing hostnames (one per line)</small>
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

        <div class="operation-actions">
          <button 
            @click="runShareEnumeration" 
            class="run-button"
            :disabled="isRunning || !config.targets.trim()"
            :class="{ 'running': isRunning }"
          >
            {{ isRunning ? 'Enumerating...' : 'Run Share Enumeration' }}
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
                <p><strong>Total entries:</strong> {{ result.total_entries }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- File Walking Operation -->
      <div v-else-if="selectedOperation === 1" class="operation-details">
        <div class="operation-header">
          <h2>{{ operations[1].name }}</h2>
          <p class="operation-description">{{ operations[1].description }}</p>
        </div>

        <div class="operation-config">
          <h3>Configuration</h3>
          <div class="config-form">
            <div class="config-item">
              <label>Target Hosts</label>
              <div class="input-group">
                <input 
                  type="text" 
                  v-model="config.targets" 
                  placeholder="Enter comma-separated hostnames (e.g., DC01,FILE01,SHARE01)" 
                  :disabled="isRunning"
                >
                <button 
                  @click="selectHostsFile" 
                  class="file-button"
                  :disabled="isRunning"
                  title="Select file containing hostnames"
                >
                  📁
                </button>
              </div>
              <small class="help-text">Enter hostnames separated by commas, or select a file containing hostnames (one per line)</small>
            </div>

            <div class="config-item">
              <label>Maximum Directory Depth</label>
              <input 
                type="number" 
                v-model.number="config.max_depth" 
                min="1" 
                max="10"
                :disabled="isRunning"
              >
              <small class="help-text">How deep to traverse directory structures (1-10)</small>
            </div>

            <div class="config-item">
              <label>Max Entries per Share</label>
              <input 
                type="number" 
                v-model.number="config.max_entries" 
                min="100" 
                max="50000"
                placeholder="5000"
                :disabled="isRunning"
              >
              <small class="help-text">Limit entries per share to prevent excessive data collection</small>
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

        <div class="operation-actions">
          <button 
            @click="runFullIndexing" 
            class="run-button"
            :disabled="isRunning || !config.targets.trim()"
            :class="{ 'running': isRunning }"
          >
            {{ isRunning ? 'Indexing...' : 'Run Full Indexing' }}
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
                <p><strong>Total entries:</strong> {{ result.total_entries }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- UNC Paths Operation -->
      <div v-else-if="selectedOperation === 2" class="operation-details">
        <div class="operation-header">
          <h2>{{ operations[2].name }}</h2>
          <p class="operation-description">{{ operations[2].description }}</p>
        </div>

        <div class="operation-config">
          <h3>Configuration</h3>
          <div class="config-form">
            <div class="config-item">
              <label>UNC Paths File</label>
              <div class="input-group">
                <input 
                  type="text" 
                  v-model="config.shares_file" 
                  placeholder="Select file containing UNC paths" 
                  readonly
                  :disabled="isRunning"
                >
                <button 
                  @click="selectUNCFile" 
                  class="file-button"
                  :disabled="isRunning"
                  title="Select file containing UNC paths"
                >
                  📁
                </button>
              </div>
              <small class="help-text">Select a file containing UNC paths (one per line, e.g., \\server\share)</small>
            </div>

            <div class="config-item">
              <label>Maximum Directory Depth</label>
              <input 
                type="number" 
                v-model.number="config.max_depth" 
                min="1" 
                max="10"
                :disabled="isRunning"
              >
              <small class="help-text">How deep to traverse directory structures (1-10)</small>
            </div>

            <div class="config-item">
              <label>Max Entries per Share</label>
              <input 
                type="number" 
                v-model.number="config.max_entries" 
                min="100" 
                max="50000"
                placeholder="5000"
                :disabled="isRunning"
              >
              <small class="help-text">Limit entries per share to prevent excessive data collection</small>
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

        <div class="operation-actions">
          <button 
            @click="runUNCIndexing" 
            class="run-button"
            :disabled="isRunning || !config.shares_file"
            :class="{ 'running': isRunning }"
          >
            {{ isRunning ? 'Indexing...' : 'Run UNC Indexing' }}
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
                <p><strong>Total entries:</strong> {{ result.total_entries }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Domain Enumeration Operation -->
      <div v-else-if="selectedOperation === 3" class="operation-details">
        <div class="operation-header">
          <h2>{{ operations[3].name }}</h2>
          <p class="operation-description">{{ operations[3].description }}</p>
        </div>

        <div class="operation-config">
          <h3>Configuration</h3>
          <div class="config-form">
            <div class="config-item">
              <label>Domain Controller</label>
              <input 
                type="text" 
                v-model="config.dc_hostname" 
                placeholder="Enter domain controller hostname or IP (e.g., dc01.domain.local)" 
                :disabled="isRunning"
              >
              <small class="help-text">Domain controller hostname or IP address</small>
            </div>

            <div class="config-item">
              <label>Base DN (optional)</label>
              <input 
                type="text" 
                v-model="config.base_dn" 
                placeholder="E.g., DC=domain,DC=local (will attempt auto-detection if empty)" 
                :disabled="isRunning"
              >
              <small class="help-text">Starting point for LDAP searches - will attempt auto-detection if left empty</small>
            </div>

            <div class="config-item">
              <label>Username</label>
              <input 
                type="text" 
                v-model="config.username" 
                placeholder="username@domain.local" 
                :disabled="isRunning"
              >
              <small class="help-text">Username with LDAP read permissions, UPN format accepted. Leave empty if current user tickets (kerberos) should be used.</small>
            </div>

            <div class="config-item">
              <label>Password</label>
              <input 
                type="password" 
                v-model="config.password" 
                placeholder="Password for authentication" 
                :disabled="isRunning"
              >
              <small class="help-text">Password for LDAP authentication. Leave empty if current user tickets (kerberos) should be used.</small>
            </div>

            <div class="config-item">
              <label>Enumeration Mode</label>
              <select 
                v-model="config.ldap_mode" 
                :disabled="isRunning"
              >
                <option value="all">All (computers, users, and groups)</option>
                <option value="computers">Computers only</option>
                <option value="users">Users only</option>
                <option value="groups">Groups only</option>
              </select>
              <small class="help-text">Select which directory objects to enumerate</small>
            </div>

            <div class="config-item">
              <label>
                <input 
                  type="checkbox" 
                  v-model="config.use_ldaps"
                  :disabled="isRunning"
                >
                Use LDAPS (Secure LDAP)
              </label>
              <small class="help-text">Use encrypted LDAPS connection (recommended)</small>
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

        <div class="operation-actions">
          <button 
            @click="runLdapEnumeration" 
            class="run-button"
            :disabled="isRunning || !config.dc_hostname.trim()"
            :class="{ 'running': isRunning }"
          >
            {{ isRunning ? 'Enumerating...' : 'Run Domain Enumeration' }}
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
              <div v-if="result.success && result.output_files" class="output-info">
                <p><strong>Output files:</strong></p>
                <ul>
                  <li v-for="(file, idx) in result.output_files" :key="idx">{{ file }}</li>
                </ul>
                <p><strong>Total entries:</strong> {{ result.total_entries }}</p>
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
    name: 'Enumerate Network Shares',
    description: 'Enumerate network shares on target hosts without walking directories'
  },
  {
    name: 'Enumerate and Index Network Shares',
    description: 'Enumerate shares and walk directory structures to collect file metadata'
  },
  {
    name: 'Index Network Shares from list',
    description: 'Walk predefined UNC paths from a file to collect file metadata'
  },
  {
    name: 'Enumerate Active Directory',
    description: 'Query Active Directory to enumerate computers, users, and groups'
  }
];

// Configuration state
const config = ref({
  targets: '',
  max_depth: 3,
  max_entries: 5000,
  debug_mode: false,
  share_enum_only: false,
  shares_file: null,
  // LDAP config
  dc_hostname: '',
  base_dn: '',
  username: '',
  password: '',
  use_ldaps: false,
  ldap_mode: 'all'
});

// Execution state
const isRunning = ref(false);
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
const selectHostsFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Text files',
        extensions: ['txt', 'csv']
      }]
    });
    
    if (selected) {
      config.value.targets = selected;
    }
  } catch (error) {
    console.error('Error selecting hosts file:', error);
  }
};

const selectUNCFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Text files',
        extensions: ['txt', 'csv']
      }]
    });
    
    if (selected) {
      config.value.shares_file = selected;
    }
  } catch (error) {
    console.error('Error selecting UNC file:', error);
  }
};

// Execution methods
const runShareEnumeration = async () => {
  if (isRunning.value) return;
  
  clearResults();
  isRunning.value = true;
  
  try {
    const indexConfig = {
      targets: config.value.targets,
      max_depth: config.value.max_depth,
      max_entries: config.value.max_entries || null,
      debug_mode: config.value.debug_mode,
      share_enum_only: true,
      shares_file: null
    };
    
    const response = await invoke('start_active_indexing', { config: indexConfig });
    result.value = response;
    
  } catch (error) {
    console.error('Error running share enumeration:', error);
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

const runFullIndexing = async () => {
  if (isRunning.value) return;
  
  clearResults();
  isRunning.value = true;
  
  try {
    const indexConfig = {
      targets: config.value.targets,
      max_depth: config.value.max_depth,
      max_entries: config.value.max_entries || null,
      debug_mode: config.value.debug_mode,
      share_enum_only: false,
      shares_file: null
    };
    
    const response = await invoke('start_active_indexing', { config: indexConfig });
    result.value = response;
    
  } catch (error) {
    console.error('Error running full indexing:', error);
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

const runUNCIndexing = async () => {
  if (isRunning.value) return;
  
  clearResults();
  isRunning.value = true;
  
  try {
    const indexConfig = {
      targets: '',
      max_depth: config.value.max_depth,
      max_entries: config.value.max_entries || null,
      debug_mode: config.value.debug_mode,
      share_enum_only: false,
      shares_file: config.value.shares_file
    };
    
    const response = await invoke('start_active_indexing', { config: indexConfig });
    result.value = response;
    
  } catch (error) {
    console.error('Error running UNC indexing:', error);
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

const runLdapEnumeration = async () => {
  if (isRunning.value) return;
  
  clearResults();
  isRunning.value = true;
  
  try {
    const ldapConfig = {
      dc_hostname: config.value.dc_hostname,
      base_dn: config.value.base_dn,
      username: config.value.username,
      password: config.value.password,
      use_ldaps: config.value.use_ldaps,
      mode: config.value.ldap_mode,
      debug_mode: config.value.debug_mode
    };
    
    const response = await invoke('start_ldap_enumeration', { config: ldapConfig });
    result.value = response;
    
  } catch (error) {
    console.error('Error running domain enumeration:', error);
    result.value = {
      success: false,
      message: error.toString(),
      output_files: [],
      total_entries: 0,
      errors: [error.toString()]
    };
  } finally {
    isRunning.value = false;
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
  // Listen for progress updates
  progressUnlisten = await listen('indexing-progress', (event) => {
    progress.value = event.payload;
  });
  
  // Listen for log messages
  logUnlisten = await listen('indexing-log', (event) => {
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
.active-index {
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