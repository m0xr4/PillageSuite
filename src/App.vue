<script setup>
import { ref, onMounted, computed } from "vue";
import AppLayout from "./components/AppLayout.vue";
import Neo4jConnection from "./components/Neo4jConnection.vue";
import CypherQueryEditor from "./components/CypherQueryEditor.vue";
import QueryResults from "./components/QueryResults.vue";
import GraphVisualization from "./components/GraphVisualization.vue";
import LiveSearch from "./components/LiveSearch.vue";
import JsonImport from "./components/JsonImport.vue";
import TabbedSidebar from "./components/TabbedSidebar.vue";
import SavedQueries from "./components/SavedQueries.vue";
import StatsView from "./components/StatsView.vue";
import StatsCharts from "./components/StatsCharts.vue";
import ActiveIndex from "./components/ActiveIndex.vue";
import CredentialGathering from "./components/CredentialGathering.vue";
import AboutView from "./components/AboutView.vue";

// Neo4j connection state
const isConnected = ref(false);
const session = ref(null);
const driver = ref(null);

// Query results
const queryResults = ref(null);

// Current cypher query
const currentQuery = ref("");

// Active tab for sidebar
const activeTab = ref(0);

// Reference to the query editor component
const queryEditorRef = ref(null);

// Reference to the saved queries component
const savedQueriesRef = ref(null);

// Add refs for stats components
const statsViewRef = ref(null);
const statsChartsRef = ref(null);

// Selected operation for Active Index
const selectedOperation = ref(null);

// Handle operation selection
const handleOperationSelect = (index) => {
  selectedOperation.value = index;
};

// Selected operation for Credential Gathering
const selectedCredentialOperation = ref(null);

// Handle credential operation selection
const handleCredentialOperationSelect = (index) => {
  selectedCredentialOperation.value = index;
};

// Check if the result contains only paths
const isPathOnlyResult = computed(() => {
  if (!queryResults.value || !queryResults.value.records || queryResults.value.records.length === 0) {
    return false;
  }
  
  // Check if the result is a single column named 'p'
  const keys = queryResults.value.records[0].keys;
  if (keys.length !== 1 || keys[0] !== 'p') {
    return false;
  }
  
  // Check if that column contains a path
  const firstRecord = queryResults.value.records[0];
  const value = firstRecord.get('p');
  return value && typeof value === 'object' && value.segments && Array.isArray(value.segments);
});

// Handle Neo4j connection
const handleConnection = (connectionData) => {
  driver.value = connectionData.driver;
  session.value = connectionData.session;
  isConnected.value = true;
};

// Handle connection error
const handleConnectionError = (error) => {
  console.error("Connection error:", error);
  isConnected.value = false;
  session.value = null;
  driver.value = null;
};

// Handle query results
const handleQueryResult = (result) => {
  if (!result || !result.records) {
    console.error("Invalid query result received");
    return;
  }
  
  console.log(`Query executed: ${result.records.length} records returned`);
  queryResults.value = result;
};

// Handle query submission
const handleQuerySubmit = (query) => {
  currentQuery.value = query;
};

// Handle save query request
const handleSaveQuery = (query) => {
  currentQuery.value = query;
  // Show the save dialog and switch to the discovery tab where SavedQueries is located
  activeTab.value = 1; // Switch to discovery tab
  
  // Use nextTick to ensure component is mounted before accessing it
  setTimeout(() => {
    if (savedQueriesRef.value) {
      savedQueriesRef.value.showSaveDialog();
    }
  }, 100);
};

// Handle query from saved queries
const handleSavedQueryRun = async (query) => {
  console.log("Running saved query:", query);
  
  // Force a complete reset of all visualization state before running a saved query
  // This ensures the graph visualization resets even after viewing non-graph data
  queryResults.value = null;
  
  // Use a small delay to allow state to reset properly
  setTimeout(async () => {
    // Use the exposed method from the CypherQueryEditor component
    if (queryEditorRef.value) {
      await queryEditorRef.value.runExternalQuery(query);
    }
  }, 50);
};

// Handle query from NodeDetails
const handleNodeDetailsQuery = async (query, isExpansion = false) => {
  console.log("Running NodeDetails query:", query);
  
  // Force a complete reset of all visualization state before running the query
  queryResults.value = null;
  
  // Use a small delay to allow state to reset properly
  setTimeout(async () => {
    // Use the exposed method from the CypherQueryEditor component
    if (queryEditorRef.value) {
      await queryEditorRef.value.runExternalQuery(query);
    }
  }, 50);
};

// Handle query error
const handleQueryError = (error) => {
  console.error("Query error:", error);
};

// Handle node selection from search
const handleNodeSelected = (node) => {
  console.log("Node selected from LiveSearch:", node);
  
  // Force a complete reset of all visualization state
  // First, set query results to null to trigger cleanup
  queryResults.value = null;
  
  // Use a longer delay to ensure complete state reset
  setTimeout(() => {
    // Create a mock result to display the selected node with special flags to ensure visualization
    queryResults.value = {
      records: [
        {
          keys: ["n"],
          // Ensure we return a properly formatted record with get function
          get: (key) => key === "n" ? node : null,
          _fields: [node]  // Add _fields property for compatibility with Neo4j records
        }
      ],
      // Add special metadata for LiveSearch results
      summary: {
        resultAvailableAfter: 0,
        resultConsumedAfter: 0,
        isLiveSearchResult: true,  // Flag to identify as LiveSearch result
        forceVisualization: true   // Additional flag to force visualization
      }
    };
    
    // Focus on graph visualization after the node is displayed
    setTimeout(() => {
      // Scroll to the graph visualization section
      const graphElement = document.querySelector('.graph-container-wrapper');
      if (graphElement) {
        graphElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    }, 200);
  }, 100);
};

// Handle JSON import success
const handleImportSuccess = (data) => {
  console.log(`Successfully imported ${data.count} results`);
};

// Handle JSON import error
const handleImportError = (error) => {
  console.error("Import error:", error);
};

// Handle tab change from dropdown
const handleTabChange = (tabIndex) => {
  activeTab.value = tabIndex;
};

// Handle stats refresh
const handleStatsRefresh = () => {
  if (statsViewRef.value) {
    statsViewRef.value.refresh();
  }
};
</script>

<template>
  <AppLayout :isConnected="isConnected" @tabChange="handleTabChange">
    <template #sidebar>
      <TabbedSidebar :activeTab="activeTab">
        <!-- Connection Tab -->
        <template #connection-tab>
          <div class="info-box">
            <svg xmlns="http://www.w3.org/2000/svg" class="info-icon" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2h-1V9a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
            <p>Instructions to run a neo4j Instance with docker, optionally in WSL can be found in the README.md</p>
          </div>
        </template>
        
        <!-- Discovery Tab -->
        <template #discovery-tab>
          <LiveSearch 
            :session="session" 
            :isConnected="isConnected"
            @nodeSelected="handleNodeSelected"
          />
          
          <SavedQueries
            ref="savedQueriesRef"
            :currentQuery="currentQuery"
            @runQuery="handleSavedQueryRun"
          />
        </template>
        
        <!-- Import Tab -->
        <template #import-tab>
          <JsonImport 
            :session="session" 
            :isConnected="isConnected"
            @importSuccess="handleImportSuccess"
            @importError="handleImportError"
          />
        </template>

        <!-- Stats Tab -->
        <template #stats-tab>
          <StatsView
            ref="statsViewRef"
            :session="session"
            :isConnected="isConnected"
          />
        </template>

        <!-- Active Index Tab -->
        <template #active-index-tab>
          <ActiveIndex 
            :mainContentOnly="false" 
            :selectedOperation="selectedOperation"
            @select="handleOperationSelect"
          />
        </template>

        <!-- Credential Gathering Tab -->
        <template #credential-gathering-tab>
          <CredentialGathering 
            :mainContentOnly="false" 
            :selectedOperation="selectedCredentialOperation"
            @select="handleCredentialOperationSelect"
          />
        </template>

        <!-- About Tab -->
        <template #about-tab>
          <div class="info-box">
            <svg xmlns="http://www.w3.org/2000/svg" class="info-icon" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2h-1V9a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
            <p>Information about the tool, its author and the GPLv3 license.</p>
          </div>
        </template>
      </TabbedSidebar>
    </template>
    
    <template #main>
      <!-- Show Neo4j Connection when Connection tab is active -->
      <Neo4jConnection 
        v-if="activeTab === 0"
        @connected="handleConnection" 
        @connectionError="handleConnectionError" 
      />
      
      <!-- Show StatsCharts when Stats tab is active -->
      <StatsCharts
        v-else-if="activeTab === 3"
        ref="statsChartsRef"
        :session="session"
        :isConnected="isConnected"
        @refresh="handleStatsRefresh"
      />
      
      <!-- Show ActiveIndex when Active Index tab is active -->
      <ActiveIndex
        v-else-if="activeTab === 4"
        :mainContentOnly="true"
        :selectedOperation="selectedOperation"
        @select="handleOperationSelect"
      />

      <!-- Show CredentialGathering when Credential Gathering tab is active -->
      <CredentialGathering
        v-else-if="activeTab === 5"
        :mainContentOnly="true"
        :selectedOperation="selectedCredentialOperation"
        @select="handleCredentialOperationSelect"
      />

      <!-- Show About when About tab is active -->
      <AboutView 
        v-else-if="activeTab === 6"
      />
      
      <!-- Show regular content for other tabs -->
      <div v-else>
        <CypherQueryEditor 
          ref="queryEditorRef"
          id="cypher-query-editor"
          :session="session" 
          :isConnected="isConnected"
          @queryResult="handleQueryResult"
          @queryError="handleQueryError"
          @querySubmit="handleQuerySubmit"
          @saveQuery="handleSaveQuery"
        />
        
        <!-- Show hint message when a path query is executed -->
        <div v-if="isPathOnlyResult" class="path-only-notice">
          <div class="info-message">
            <svg xmlns="http://www.w3.org/2000/svg" class="icon" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2h-1V9a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
            <span>Path query results are displayed in the graph visualization below</span>
          </div>
        </div>
        
        <QueryResults 
          :results="queryResults" 
        />
        
        <GraphVisualization 
          :results="queryResults" 
          @nodeDetailsQuery="handleNodeDetailsQuery"
        />
      </div>
    </template>
  </AppLayout>
</template>

<style>
/* App-specific styles only */
.debug-info {
  background-color: var(--color-code-bg);
  border: 1px solid var(--color-border);
  padding: 10px;
  margin-bottom: 20px;
  font-family: monospace;
  font-size: 12px;
  color: var(--color-code-text);
}

.path-only-notice {
  margin-bottom: 1rem;
}

.info-message {
  display: flex;
  align-items: center;
  padding: 0.75rem;
  background-color: rgba(var(--color-info-rgb), 0.1);
  border: 1px solid var(--color-info);
  border-radius: 0.375rem;
  color: var(--color-info);
  font-size: 0.875rem;
}

.info-message .icon {
  width: 1.25rem;
  height: 1.25rem;
  margin-right: 0.5rem;
  color: var(--color-info);
}

.no-results-message {
  text-align: center;
  padding: 2rem;
  background-color: var(--color-card-bg);
  border-radius: 0.375rem;
  border: 1px solid var(--color-border);
  color: var(--color-text-muted);
  font-size: 0.875rem;
  margin-bottom: 1rem;
}

.info-box {
  display: flex;
  align-items: flex-start;
  padding: 1rem;
  background-color: rgba(var(--color-info-rgb), 0.1);
  border: 1px solid var(--color-info);
  border-radius: 0.375rem;
  color: var(--color-info);
  font-size: 0.875rem;
  line-height: 1.5;
}

.info-box .info-icon {
  width: 1.25rem;
  height: 1.25rem;
  margin-right: 0.75rem;
  flex-shrink: 0;
  margin-top: 0.125rem;
}
</style>
