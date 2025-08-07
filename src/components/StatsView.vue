<template>
  <div class="stats-container">
    <div v-if="!isConnected" class="connection-required">
      <svg xmlns="http://www.w3.org/2000/svg" class="info-icon" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2h-1V9a1 1 0 00-1-1z" clip-rule="evenodd" />
      </svg>
      <span>Connect to a Neo4j database to view statistics</span>
    </div>
    
    <div v-else>
      <!-- Database Stats Summary Cards -->
      <div class="stats-summary">
        <div class="stats-card" v-for="(stat, index) in dbStats" :key="index">
          <div class="stats-icon" :style="{ backgroundColor: statColors[index % statColors.length] }">
            <svg xmlns="http://www.w3.org/2000/svg" class="icon" viewBox="0 0 20 20" fill="currentColor">
              <path v-if="stat.type === 'node'" d="M10 3.5a1.5 1.5 0 013 0V4a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-.5a1.5 1.5 0 000 3h.5a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-.5a1.5 1.5 0 00-3 0v.5a1 1 0 01-1 1H6a1 1 0 01-1-1v-3a1 1 0 00-1-1h-.5a1.5 1.5 0 010-3H4a1 1 0 001-1V6a1 1 0 011-1h3a1 1 0 001-1v-.5z" />
              <path v-else-if="stat.type === 'user'" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" />
              <path v-else-if="stat.type === 'identity'" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" />
              <path v-else-if="stat.type === 'group'" d="M13 6a3 3 0 11-6 0 3 3 0 016 0zM18 8a2 2 0 11-4 0 2 2 0 014 0zM14 15a4 4 0 00-8 0v3h8v-3zM6 8a2 2 0 11-4 0 2 2 0 014 0zM16 18v-3a5.972 5.972 0 00-.75-2.906A3.005 3.005 0 0119 15v3h-3zM4.75 12.094A5.973 5.973 0 004 15v3H1v-3a3 3 0 013.75-2.906z" />
              <path v-else-if="stat.type === 'share'" d="M15 8a3 3 0 10-2.977-2.63l-4.94 2.47a3 3 0 100 4.319l4.94 2.47a3 3 0 10.895-1.789l-4.94-2.47a3.027 3.027 0 000-.74l4.94-2.47C13.456 7.68 14.19 8 15 8z" />
              <path v-else-if="stat.type === 'computer'" d="M13 7H7v6h6V7z" />
              <path v-else-if="stat.type === 'file'" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" />
              <path v-else-if="stat.type === 'directory'" d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
              <path v-else d="M2 11a1 1 0 011-1h2a1 1 0 011 1v5a1 1 0 01-1 1H3a1 1 0 01-1-1v-5zM8 7a1 1 0 011-1h2a1 1 0 011 1v9a1 1 0 01-1 1H9a1 1 0 01-1-1V7zM14 4a1 1 0 011-1h2a1 1 0 011 1v12a1 1 0 01-1 1h-2a1 1 0 01-1-1V4z" />
            </svg>
          </div>
          <div class="stats-content">
            <div class="stats-value">{{ stat.value }}</div>
            <div class="stats-label">{{ stat.label }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue';
import { executeCypherQuery } from '../services/neo4jService';

const props = defineProps({
  session: Object,
  isConnected: Boolean
});

// Database stats
const dbStats = ref([
  { label: 'Total Nodes', value: 0, type: 'node' },
  { label: 'Users', value: 0, type: 'User' },
  { label: 'Groups', value: 0, type: 'Group' },
  { label: 'Shares', value: 0, type: 'share' },
  { label: 'Files', value: 0, type: 'file' },
  { label: 'Directories', value: 0, type: 'directory' },
  { label: 'Computers', value: 0, type: 'Computer' }
]);

// Colors for stats cards
const statColors = [
  'rgba(54, 162, 235, 0.9)',
  'rgba(255, 99, 132, 0.9)',
  'rgba(75, 192, 192, 0.9)',
  'rgba(153, 102, 255, 0.9)',
  'rgba(255, 159, 64, 0.9)'
];

// Fetch database statistics
const fetchStats = async () => {
  try {
    if (!props.isConnected || !props.session) return;
    
    // Get total node count
    const totalNodesResult = await executeCypherQuery(
      'MATCH (n) RETURN count(n) as count',
      props.session
    );
    
    if (totalNodesResult && totalNodesResult.records.length > 0) {
      dbStats.value[0].value = totalNodesResult.records[0].get('count');
    }
    
    // Get user count
    const usersResult = await executeCypherQuery(
      'MATCH (u:User) RETURN count(u) as count',
      props.session
    );
    
    if (usersResult && usersResult.records.length > 0) {
      dbStats.value[1].value = usersResult.records[0].get('count');
    }
    
    // Get group count
    const groupsResult = await executeCypherQuery(
      'MATCH (g:Group) RETURN count(g) as count',
      props.session
    );
    
    if (groupsResult && groupsResult.records.length > 0) {
      dbStats.value[2].value = groupsResult.records[0].get('count');
    }
    
    // Get share count
    const sharesResult = await executeCypherQuery(
      'MATCH (d:share) RETURN count(d) as count',
      props.session
    );
    
    if (sharesResult && sharesResult.records.length > 0) {
      dbStats.value[3].value = sharesResult.records[0].get('count');
    }
    
    // Get file count
    const filesResult = await executeCypherQuery(
      'MATCH (f:file) RETURN count(f) as count',
      props.session
    );
    
    if (filesResult && filesResult.records.length > 0) {
      dbStats.value[4].value = filesResult.records[0].get('count');
    }
    
    // Get directory count
    const directoriesResult = await executeCypherQuery(
      'MATCH (d:directory) RETURN count(d) as count',
      props.session
    );
    
    if (directoriesResult && directoriesResult.records.length > 0) {
      dbStats.value[5].value = directoriesResult.records[0].get('count');
    }
    
    // Get computer count
    const computersResult = await executeCypherQuery(
      'MATCH (c:Computer) RETURN count(c) as count',
      props.session
    );
    
    if (computersResult && computersResult.records.length > 0) {
      dbStats.value[6].value = computersResult.records[0].get('count');
    }
    
  } catch (error) {
    console.error('Error fetching database stats:', error);
  }
};

// Expose refresh method
defineExpose({
  refresh: fetchStats
});

// Watch for connection state changes
watch(() => props.isConnected, (newVal) => {
  if (newVal) {
    fetchStats();
  }
});

// Initialize stats when component is mounted
onMounted(() => {
  if (props.isConnected) {
    fetchStats();
  }
});
</script>

<style scoped>
.stats-container {
  width: 100%;
}

.connection-required {
  display: flex;
  align-items: center;
  padding: 1rem;
  background-color: rgba(var(--color-info-rgb), 0.1);
  border: 1px solid var(--color-info);
  border-radius: 0.375rem;
  color: var(--color-text);
}

.info-icon {
  height: 1.5rem;
  width: 1.5rem;
  margin-right: 0.5rem;
  color: var(--color-info);
}

.stats-summary {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.stats-card {
  display: flex;
  background-color: var(--color-card-bg);
  border-radius: 0.5rem;
  overflow: hidden;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.stats-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.5rem;
  padding: 0.75rem;
  color: white;
}

.stats-icon .icon {
  height: 1.5rem;
  width: 1.5rem;
}

.stats-content {
  flex: 1;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.stats-value {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-heading);
}

.stats-label {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  margin-top: 0.25rem;
}
</style> 