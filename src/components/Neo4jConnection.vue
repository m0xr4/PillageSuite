<template>
  <div class="card">
    <h2 class="card-title">Neo4j Connection</h2>
    <form @submit.prevent="connect" class="form-container">
      <div class="form-group">
        <label for="uri" class="form-label">Neo4j URI</label>
        <input
          id="uri"
          v-model="connectionDetails.uri"
          type="text"
          placeholder="bolt://localhost:7687"
          class="form-input"
          required
        />
      </div>
      <div class="form-group">
        <label for="username" class="form-label">Username</label>
        <input
          id="username"
          v-model="connectionDetails.username"
          type="text"
          placeholder="neo4j"
          class="form-input"
          required
        />
      </div>
      <div class="form-group">
        <label for="password" class="form-label">Password</label>
        <input
          id="password"
          v-model="connectionDetails.password"
          type="password"
          placeholder="password"
          class="form-input"
          required
        />
      </div>
      <div class="flex">
        <button
          type="submit"
          class="button-primary"
        >
          {{ isConnected ? 'Reconnect' : 'Connect' }}
        </button>
        <div v-if="connectionStatus" :class="connectionStatusClass">
          {{ connectionStatus }}
        </div>
      </div>
    </form>
  </div>
</template>

<script setup>
import { ref, reactive, computed } from 'vue';
import { connectToNeo4j } from '../services/neo4jService';

const connectionDetails = reactive({
  uri: 'bolt://localhost:7687',
  username: 'neo4j',
  password: ''
});

const isConnected = ref(false);
const connectionStatus = ref('');

const connectionStatusClass = computed(() => {
  return isConnected.value 
    ? 'status-success' 
    : 'status-error';
});

const connect = async () => {
  try {
    const result = await connectToNeo4j(
      connectionDetails.uri,
      connectionDetails.username,
      connectionDetails.password
    );

    if (result.success) {
      isConnected.value = true;
      connectionStatus.value = 'Connected successfully';
      
      // Emit connection event with driver and session
      emit('connected', { 
        driver: result.driver, 
        session: result.session 
      });
    } else {
      isConnected.value = false;
      connectionStatus.value = `Connection failed: ${result.error}`;
      
      // Emit connection error
      emit('connectionError', new Error(result.error));
    }
  } catch (error) {
    console.error('Connection error:', error);
    isConnected.value = false;
    connectionStatus.value = `Connection failed: ${error.message}`;
    
    // Emit connection error
    emit('connectionError', error);
  }
};

// Define emits
const emit = defineEmits(['connected', 'connectionError']);
</script>

<style scoped>
/* Using established styles from styles.css with additional padding */
.card {
  max-width: 500px;
  margin: 0 auto;
}

.form-container {
  padding: 16px 24px;
}
</style> 