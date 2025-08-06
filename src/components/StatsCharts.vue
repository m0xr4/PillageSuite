<template>
  <div class="stats-charts">
    <div v-if="!isConnected" class="connection-required">
      <svg xmlns="http://www.w3.org/2000/svg" class="info-icon" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2h-1V9a1 1 0 00-1-1z" clip-rule="evenodd" />
      </svg>
      <span>Connect to a Neo4j database to view statistics</span>
    </div>
    
    <div v-else>
      <!-- Refresh Button -->
      <div class="refresh-button-container">
        <button class="refresh-button" @click="handleRefresh" :disabled="isRefreshing">
          <svg xmlns="http://www.w3.org/2000/svg" class="refresh-icon" :class="{ 'rotating': isRefreshing }" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" />
          </svg>
          <span>Refresh Stats</span>
        </button>
      </div>

      <!-- Chart Visualizations -->
      <div class="charts-container">
        <div class="chart-wrapper">
          <h3>Share Access Permissions</h3>
          <canvas ref="sharePermissionsChart"></canvas>
        </div>
        <div class="chart-wrapper">
          <h3>Relationship Types</h3>
          <canvas ref="relationshipChart"></canvas>
        </div>
      </div>
      
      <div class="charts-container">
        <div class="chart-wrapper">
          <h3>Top File Extensions</h3>
          <canvas ref="fileExtensionsChart"></canvas>
        </div>
        <div class="chart-wrapper">
          <h3>Last Modify Distribution</h3>
          <canvas ref="lastModifyChart"></canvas>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue';
import { executeCypherQuery } from '../services/neo4jService';
import Chart from 'chart.js/auto';

const props = defineProps({
  session: Object,
  isConnected: Boolean
});

const emit = defineEmits(['refresh']);

// Add isRefreshing state
const isRefreshing = ref(false);

// References for chart canvases
const sharePermissionsChart = ref(null);
const relationshipChart = ref(null);
const fileExtensionsChart = ref(null);
const lastModifyChart = ref(null);

// Charts objects
let sharePermChart = null;
let relChart = null;
let fileExtChart = null;
let modifyChart = null;

// Colors for charts
const chartColors = [
  'rgba(54, 162, 235, 0.7)',
  'rgba(255, 99, 132, 0.7)',
  'rgba(75, 192, 192, 0.7)',
  'rgba(255, 159, 64, 0.7)',
  'rgba(153, 102, 255, 0.7)',
  'rgba(255, 206, 86, 0.7)',
  'rgba(231, 233, 237, 0.7)',
  'rgba(97, 97, 97, 0.7)'
];

// Chart data
const sharePermissionsData = ref({
  labels: [],
  counts: []
});

const relationshipData = ref({
  labels: [],
  counts: []
});

const fileExtensionsData = ref({
  labels: [],
  counts: []
});

const lastModifyData = ref({
  labels: [],
  datasets: [
    {
      label: 'Files Modified',
      data: [],
      backgroundColor: chartColors[0],
      borderWidth: 1
    }
  ]
});

// Fetch chart data
const fetchChartData = async () => {
  try {
    if (!props.isConnected || !props.session) return;
    
    // Get Share Permissions data
    const sharePermResult = await executeCypherQuery(
      `MATCH (s:share)
       OPTIONAL MATCH (i:Identity)-[r]-(s)
       WHERE i.name IN ["Everyone", "Authenticated Users", "BUILTIN\\\\Users"] 
         AND type(r) IN ['FullControl', 'ReadData/ListDirectory', 'WriteData/AddFile']
       WITH s, count(i) > 0 AS hasInsecurePath
       WITH hasInsecurePath, count(s) AS shareCount
       RETURN hasInsecurePath, shareCount`,
      props.session
    );
    
    // Process Share Permissions data
    if (sharePermResult && sharePermResult.records.length > 0) {
      let insecureCount = 0;
      let secureCount = 0;
      sharePermResult.records.forEach(record => {
        if (record.get('hasInsecurePath')) {
          insecureCount = record.get('shareCount');
        } else {
          secureCount = record.get('shareCount');
        }
      });
      sharePermissionsData.value.labels = ["Access for all Domain Users", "Access restricted"];
      sharePermissionsData.value.counts = [insecureCount, secureCount];
      updateSharePermissionsChart();
    }
    
    // Get relationship type distribution
    const relResult = await executeCypherQuery(
      'MATCH ()-[r]->() RETURN type(r) as type, count(r) as count ORDER BY count DESC LIMIT 8',
      props.session
    );
    
    if (relResult && relResult.records.length > 0) {
      relationshipData.value.labels = relResult.records.map(record => record.get('type'));
      relationshipData.value.counts = relResult.records.map(record => record.get('count'));
      updateRelationshipChart();
    }
    
    // Get top file extensions
    const fileExtResult = await executeCypherQuery(
      `MATCH (f:file) 
       WHERE f.extension IS NOT NULL AND f.extension <> ""
       RETURN f.extension as extension, count(f) as count 
       ORDER BY count DESC 
       LIMIT 10`,
      props.session
    );
    
    if (fileExtResult && fileExtResult.records.length > 0) {
      fileExtensionsData.value.labels = fileExtResult.records.map(record => record.get('extension'));
      fileExtensionsData.value.counts = fileExtResult.records.map(record => record.get('count'));
      updateFileExtensionsChart();
    }
    
    // Get Last Modify Distribution data
    const lastModifyResult = await executeCypherQuery(
      `MATCH (f:file)
       WHERE f.modified IS NOT NULL AND f.modified <> "" AND size(f.modified) >= 4
       WITH toInteger(substring(f.modified, 0, 4)) AS year, count(f) AS count
       WHERE count >= 5
       RETURN year, count
       ORDER BY year ASC`,
      props.session
    );

    if (lastModifyResult && lastModifyResult.records.length > 0) {
      lastModifyData.value.labels = lastModifyResult.records.map(record => record.get('year'));
      lastModifyData.value.datasets[0].data = lastModifyResult.records.map(record => record.get('count'));
      updateLastModifyChart();
    }
    
  } catch (error) {
    console.error('Error fetching chart data:', error);
  }
};

const updateSharePermissionsChart = () => {
  if (sharePermChart) {
    sharePermChart.destroy();
  }
  
  if (!sharePermissionsChart.value) return;
  
  const ctx = sharePermissionsChart.value.getContext('2d');
  sharePermChart = new Chart(ctx, {
    type: 'pie',
    data: {
      labels: sharePermissionsData.value.labels,
      datasets: [{
        data: sharePermissionsData.value.counts,
        backgroundColor: [chartColors[1], chartColors[0]],
        borderWidth: 1
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: {
          position: 'right',
        }
      }
    }
  });
};

const updateRelationshipChart = () => {
  if (relChart) {
    relChart.destroy();
  }
  
  if (!relationshipChart.value) return;
  
  const ctx = relationshipChart.value.getContext('2d');
  relChart = new Chart(ctx, {
    type: 'pie',
    data: {
      labels: relationshipData.value.labels,
      datasets: [{
        data: relationshipData.value.counts,
        backgroundColor: chartColors.slice(0, relationshipData.value.labels.length),
        borderWidth: 1
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: {
          position: 'right',
        }
      }
    }
  });
};

const updateFileExtensionsChart = () => {
  if (fileExtChart) {
    fileExtChart.destroy();
  }
  
  if (!fileExtensionsChart.value) return;
  
  const ctx = fileExtensionsChart.value.getContext('2d');
  fileExtChart = new Chart(ctx, {
    type: 'bar',
    data: {
      labels: fileExtensionsData.value.labels,
      datasets: [{
        label: 'File Count',
        data: fileExtensionsData.value.counts,
        backgroundColor: chartColors[0],
        borderWidth: 1
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      scales: {
        y: {
          beginAtZero: true
        }
      }
    }
  });
};

const updateLastModifyChart = () => {
  if (modifyChart) {
    modifyChart.destroy();
  }
  
  if (!lastModifyChart.value) return;
  
  const ctx = lastModifyChart.value.getContext('2d');
  modifyChart = new Chart(ctx, {
    type: 'bar',
    data: lastModifyData.value,
    options: {
      responsive: true,
      maintainAspectRatio: false,
      scales: {
        y: {
          beginAtZero: true
        }
      }
    }
  });
};

// Handle refresh button click
const handleRefresh = async () => {
  if (isRefreshing.value) return;
  
  isRefreshing.value = true;
  try {
    await fetchChartData();
    emit('refresh'); // Emit event to refresh stats in parent
  } finally {
    isRefreshing.value = false;
  }
};

// Watch for connection state changes
watch(() => props.isConnected, (newVal) => {
  if (newVal) {
    fetchChartData();
  }
});

// Initialize charts when component is mounted
onMounted(() => {
  if (props.isConnected) {
    fetchChartData();
  }
});
</script>

<style scoped>
.stats-charts {
  width: 100%;
  padding: 1rem;
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

.charts-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.5rem;
  margin-bottom: 1.5rem;
}

.chart-wrapper {
  background-color: var(--color-card-bg);
  border-radius: 0.5rem;
  padding: 1rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  height: 300px;
  position: relative;
}

h3 {
  font-size: 1.125rem;
  margin-top: 0;
  margin-bottom: 1rem;
  color: var(--color-heading);
}

canvas {
  display: block;
  width: 100%;
  height: calc(100% - 40px);
}

@media (max-width: 1024px) {
  .charts-container {
    grid-template-columns: 1fr;
  }
}

.refresh-button-container {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 1rem;
}

.refresh-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background-color: var(--color-primary);
  color: white;
  border: none;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

.refresh-button:hover {
  background-color: var(--color-primary-dark);
}

.refresh-button:disabled {
  background-color: var(--color-primary-light);
  cursor: not-allowed;
}

.refresh-icon {
  width: 1.25rem;
  height: 1.25rem;
  transition: transform 0.5s;
}

.refresh-icon.rotating {
  animation: rotate 0.5s linear infinite;
}

@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style> 