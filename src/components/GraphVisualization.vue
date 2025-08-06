<template>
  <div class="card">
    <h2 class="card-title">Graph Visualization</h2>
    
    <!-- Message when no graph-visualizable data is available -->
    <div v-if="showNoGraphDataMessage" class="no-graph-data">
      <div class="empty-state-message">
        <div class="empty-state-icon-container">
          <svg class="empty-state-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <h3 class="empty-state-title">No Visualizable Graph Data</h3>
        <p class="empty-state-description">
          The current query returns properties or scalar values that cannot be visualized as a graph.
          Try using queries that return full node objects (e.g., "RETURN n" instead of "RETURN n.name")
        </p>
      </div>
    </div>
    
    <!-- Graph in normal mode - show for all other cases including LiveSearch results -->
    <div v-else-if="!isFullscreen" class="graph-container-wrapper">
      <div ref="cyContainer" class="graph-container"></div>
      
      <!-- NodeDetails panel -->
      <NodeDetails 
        :visible="nodeDetailsVisible" 
        :nodeData="selectedNodeData" 
        @toggle="toggleNodeDetails" 
        @runQuery="handleNodeDetailsQuery" 
      />
      
      <!-- Controls container -->
      <div class="graph-controls">
        <ToggleButton 
          v-model="onlyBasicACLs" 
          label="Basic Expand" 
          title="Show only basic ACLs when expanding nodes"
          @update:modelValue="updateACLMode"
        />
        <ToggleButton 
          v-model="showNodeLabels" 
          label="Node Labels" 
          title="Toggle node labels visibility"
          @update:modelValue="updateLabelVisibility"
        />
        <ToggleButton 
          v-model="showEdgeLabels" 
          label="Edge Labels" 
          title="Toggle edge labels visibility"
          @update:modelValue="updateLabelVisibility"
        />
        <button
          @click="toggleFullscreen"
          class="fullscreen-button"
          title="Enter Fullscreen"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#4338ca" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="fullscreen-icon">
            <path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"/>
          </svg>
        </button>
      </div>
      
      <!-- Warning dialog for normal mode -->
      <WarningDialog
        :show="showWarningDialog && !isFullscreen"
        title="Large Graph Warning"
        :message="warningMessage"
        confirmText="Proceed Anyway"
        cancelText="Cancel"
        :limitText="`Draw Limited (${LARGE_GRAPH_THRESHOLD})`"
        :overlay-mode="true"
        @confirm="confirmLargeGraphVisualization"
        @cancel="cancelLargeGraphVisualization"
        @limit="drawLimitedGraph"
      />
    </div>
    
    <!-- Use the NodeContextMenu component in normal mode -->
    <NodeContextMenu 
      :visible="contextMenuVisible && !isFullscreen"
      :position="contextMenuPosition"
      :nodeData="selectedNodeData"
      @expand="expandNode"
    />
  </div>
  
  <!-- Teleport the graph container in fullscreen mode -->
  <teleport to="body" v-if="isFullscreen">
    <div class="fullscreen-graph-container">
      <div ref="fullscreenContainer" class="graph-container"></div>
      
      <!-- NodeDetails panel in fullscreen -->
      <NodeDetails 
        :visible="nodeDetailsVisible" 
        :nodeData="selectedNodeData" 
        @toggle="toggleNodeDetails" 
        @runQuery="handleNodeDetailsQuery" 
      />
      
      <!-- Controls container in fullscreen -->
      <div class="graph-controls">
        <ToggleButton 
          v-model="onlyBasicACLs" 
          label="Only basic ACLs" 
          title="Show only basic ACLs when expanding nodes"
          @update:modelValue="updateACLMode"
        />
        <ToggleButton 
          v-model="showNodeLabels" 
          label="Node Labels" 
          title="Toggle node labels visibility"
          @update:modelValue="updateLabelVisibility"
        />
        <ToggleButton 
          v-model="showEdgeLabels" 
          label="Edge Labels" 
          title="Toggle edge labels visibility"
          @update:modelValue="updateLabelVisibility"
        />
        <button
          @click="toggleFullscreen"
          class="fullscreen-button"
          title="Exit Fullscreen"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#4338ca" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="fullscreen-icon">
            <path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3"/>
          </svg>
        </button>
      </div>
      
      <!-- Warning dialog for fullscreen mode -->
      <WarningDialog
        :show="showWarningDialog && isFullscreen"
        title="Large Graph Warning"
        :message="warningMessage"
        confirmText="Proceed Anyway"
        cancelText="Cancel"
        :limitText="`Draw Limited (${LARGE_GRAPH_THRESHOLD})`"
        :overlay-mode="true"
        @confirm="confirmLargeGraphVisualization"
        @cancel="cancelLargeGraphVisualization"
        @limit="drawLimitedGraph"
      />
      
      <!-- Context menu in fullscreen mode -->
      <NodeContextMenu 
        :visible="contextMenuVisible"
        :position="contextMenuPosition"
        :nodeData="selectedNodeData"
        @expand="expandNode"
      />
    </div>
  </teleport>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { executeCypherQuery, processGraphData } from '../services/neo4jService';
import { 
  initializeGraph, 
  updateGraph, 
  generateNodeExpansionQuery, 
  limitGraphData,
  processAndUpdateGraph
} from '../services/graphVisualizationService';
import NodeContextMenu from './NodeContextMenu.vue';
import ToggleButton from './ToggleButton.vue';
import WarningDialog from './WarningDialog.vue';
import NodeDetails from './NodeDetails.vue';

const props = defineProps({
  results: Object
});

const cyContainer = ref(null);
const fullscreenContainer = ref(null);
let cy = null;
const isFullscreen = ref(false);

// Track if the current results have graph-visualizable data
const hasGraphData = ref(true);

// Computed property to determine if the "no graph data" message should be shown
const showNoGraphDataMessage = computed(() => {
  // Don't show message for LiveSearch results or forced visualization, but DO show for saved queries if they return no graph data
  const isSpecialResult = props.results && 
                         props.results.summary && 
                         (props.results.summary.isLiveSearchResult || 
                          props.results.summary.forceVisualization);
  
  // Always check if we have any elements in the graph
  const hasGraphElements = cy && cy.elements().length > 0;
  
  // Show message when:
  // 1. We have results
  // 2. hasGraphData is false
  // 3. The graph is empty
  // 4. It's not a LiveSearch or forced visualization result
  return props.results && 
         props.results.records && 
         props.results.records.length > 0 && 
         !hasGraphData.value &&
         !isSpecialResult &&
         !hasGraphElements;
});

// Label visibility states
const showNodeLabels = ref(true);
const showEdgeLabels = ref(true);
const onlyBasicACLs = ref(true); // Default to true

// Update ACL mode
const updateACLMode = () => {
  console.log(`ACL mode updated: Only basic ACLs = ${onlyBasicACLs.value}`);
};

// Context menu state
const contextMenuVisible = ref(false);
const contextMenuPosition = ref({ x: 0, y: 0 });
const selectedNodeData = ref(null);

// Node details panel state
const nodeDetailsVisible = ref(false);

// Toggle node details panel
const toggleNodeDetails = () => {
  nodeDetailsVisible.value = !nodeDetailsVisible.value;
};

// Large graph warning state
const showWarningDialog = ref(false);
const warningMessage = ref('');
const pendingGraphData = ref(null);
const isExpandOperation = ref(false);

// Threshold for large graph warning
const LARGE_GRAPH_THRESHOLD = 600;

// Update label visibility
const updateLabelVisibility = () => {
  if (!cy) return;
  
  // Update node labels visibility
  cy.style()
    .selector('node')
    .style({
      'label': showNodeLabels.value ? 'data(label)' : ''
    })
    .update();
  
  // Update edge labels visibility
  cy.style()
    .selector('edge')
    .style({
      'label': showEdgeLabels.value ? 'data(label)' : ''
    })
    .update();
};

// Toggle fullscreen mode
const toggleFullscreen = () => {
  isFullscreen.value = !isFullscreen.value;
  
  // Wait for the DOM to update, then reinitialize/transfer the graph
  setTimeout(() => {
    if (cy) {
      // Transfer graph to the appropriate container
      transferGraph(isFullscreen.value ? fullscreenContainer.value : cyContainer.value);
    }
  }, 100);
  
  // Hide any visible context menu when toggling fullscreen
  contextMenuVisible.value = false;
};

// Helper method to transfer graph between containers
const transferGraph = (targetContainer) => {
  const positions = {};
  const data = {};
  
  // Save positions and data of all elements
  cy.elements().forEach(ele => {
    if (ele.isNode()) {
      positions[ele.id()] = ele.position();
      data[ele.id()] = ele.data();
    } else if (ele.isEdge()) {
      data[ele.id()] = ele.data();
    }
  });
  
  // Destroy existing graph
  cy.destroy();
  
  // Initialize new graph in target container
  cy = initializeGraph(targetContainer);
  
  // Restore all nodes with their positions
  Object.keys(positions).forEach(nodeId => {
    cy.add({
      group: 'nodes',
      data: data[nodeId],
      position: positions[nodeId]
    });
  });
  
  // Restore all edges
  cy.elements().forEach(ele => {
    if (ele.isNode() && data[ele.id()]) {
      Object.keys(data).forEach(id => {
        const edgeData = data[id];
        if (edgeData.source === ele.id() && !cy.getElementById(id).length) {
          cy.add({
            group: 'edges',
            data: edgeData
          });
        }
      });
    }
  });
  
  // Apply label visibility
  updateLabelVisibility();
  
  // Apply layout and fit
  cy.fit();
  cy.center();
  
  // Set up graph events for the new graph instance
  setupGraphEvents();
};

// Handle escape key to exit fullscreen
const handleKeyDown = (event) => {
  if (event.key === 'Escape' && isFullscreen.value) {
    toggleFullscreen();
  }
};

// Draw graph with limited elements
const drawLimitedGraph = () => {
  if (pendingGraphData.value) {
    console.log("Generating limited graph data...");
    const limitedGraph = limitGraphData(pendingGraphData.value, LARGE_GRAPH_THRESHOLD, isExpandOperation.value, selectedNodeData.value);
    console.log(`Limited graph has ${limitedGraph.nodes.length} nodes and ${limitedGraph.edges.length} edges`);
    
    // Clear existing graph if not an expansion
    if (!isExpandOperation.value) {
      cy.elements().remove();
    }
    
    // Update graph with limited data
    updateGraph(cy, limitedGraph, isExpandOperation.value);
    updateLabelVisibility();
    cy.fit();
    cy.center();
    
    // Reset states
    showWarningDialog.value = false;
    pendingGraphData.value = null;
  }
};

// Watch for changes in results prop
watch(() => props.results, (newResults) => {
  if (newResults && cy) {
    // Check if this is a LiveSearch result or saved query
    const isLiveSearchResult = newResults.summary && (newResults.summary.isLiveSearchResult || newResults.summary.forceVisualization);
    const isSavedQuery = newResults.summary && newResults.summary.fromSavedQuery;
    
    // LiveSearch results always need visualization, saved queries depend on their content
    if (isLiveSearchResult || isSavedQuery) {
      console.log(`Received ${isLiveSearchResult ? 'LiveSearch result' : 'saved query'} - completely resetting graph`);
      
      // Completely reinitialize the graph to avoid state issues
      if (cy) {
        cy.destroy();
        cy = null;
      }
      
      // Reinitialize the graph container
      cy = initializeGraph(cyContainer.value);
      setupGraphEvents();
      
      // Force hasGraphData to true ONLY for LiveSearch results
      // For saved queries, let the normal process determine if there's visualizable data
      if (isLiveSearchResult) {
        hasGraphData.value = true;
      } else {
        // For saved queries, start with true but let processResults determine the actual value
        hasGraphData.value = true;
      }
      
      showWarningDialog.value = false;
      pendingGraphData.value = null;
      
      // Process the results after reinitialization
      processResults(newResults);
    } else {
      console.log("Processing regular query results for visualization");
      
      // For regular queries, reset hasGraphData flag to ensure proper evaluation
      hasGraphData.value = true;
      
      // Process the results
      processResults(newResults);
    }
    
    // Hide node details panel when results change
    nodeDetailsVisible.value = false;
  }
}, { deep: true });

// Clear graph state to avoid lingering issues
const clearGraphState = () => {
  if (cy) {
    clearGraph(cy);
    contextMenuVisible.value = false;
    selectedNodeData.value = null;
    showWarningDialog.value = false;
    pendingGraphData.value = null;
    nodeDetailsVisible.value = false;
  }
};

// Process query results with large graph safeguard
const processResults = (results, isExpansion = false) => {
  if (!results || !cy) return;
  
  // Check for special result types that should always be visualized
  const isLiveSearchResult = results.summary && (results.summary.isLiveSearchResult || results.summary.forceVisualization);
  const isSavedQuery = results.summary && results.summary.fromSavedQuery;
  
  // Only LiveSearch results should be forced to visualize when they have no graph data
  const forceVisualization = isLiveSearchResult;
  
  // Log info for debugging
  if (results.records && results.records.length > 0) {
    console.log(`Processing results with keys: ${results.records[0].keys.join(', ')}, isLiveSearch: ${isLiveSearchResult}, isSavedQuery: ${isSavedQuery}`);
  }
  
  // Special handling for LiveSearch results
  if (isLiveSearchResult && results.records?.length === 1 && results.records[0].keys?.includes('n')) {
    console.log("Processing LiveSearch result - forcing visualization");
    
    // Always set hasGraphData to true for LiveSearch results
    hasGraphData.value = true;
    
    // Clear existing graph
    cy.elements().remove();
    
    const node = results.records[0].get('n');
    if (node && typeof node === 'object') {
      console.log("LiveSearch node:", node);
      
      try {
        // Ensure we have minimum valid node data
        const nodeId = node.identity?.toString() || `node_${Date.now()}`;
        const nodeLabel = node.properties?.name || 
                        node.properties?.title || 
                        (node.labels && node.labels[0]) || 
                        'Node';
        
        // Add the node to the graph with special styling
        const nodeLabels = node.labels || [];
        cy.add({
          group: 'nodes',
          data: {
            id: nodeId,
            label: nodeLabel,
            properties: node.properties || {},
            labels: nodeLabels,
            labelString: nodeLabels.join(' '), // For cytoscape color selectors
            search: true  // Mark as search result for special styling
          },
          position: { 
            x: 300, 
            y: 200 
          }
        });
        
        // Apply styling and center the graph
        cy.center();
        cy.fit();
        updateLabelVisibility();
        console.log("LiveSearch node visualization complete");
      } catch (error) {
        console.error("Error visualizing LiveSearch node:", error);
        
        // Fallback to simpler node rendering if error occurs
        cy.add({
          group: 'nodes',
          data: {
            id: 'search_result',
            label: 'Search Result',
            search: true
          },
          position: { x: 300, y: 200 }
        });
        
        cy.center();
        cy.fit();
      }
      return;
    } else {
      console.warn("Invalid node data in LiveSearch result:", node);
    }
  }
  
  // First clear the existing graph to avoid state issues
  if (!isExpansion) {
    cy.elements().remove();
  }
  
  // Process data for visualization
  const graphData = processGraphData(results);
  hasGraphData.value = graphData.hasGraphData;
  console.log(`Processed graph data: ${graphData.nodes.length} nodes, ${graphData.edges.length} edges, hasGraphData: ${hasGraphData.value}`);
  
  // Special case - force visualization only for LiveSearch results
  if (forceVisualization) {
    hasGraphData.value = true;
  }
  
  // If there's no graph data, don't proceed with visualization
  if (!hasGraphData.value || (graphData.nodes.length === 0 && !forceVisualization)) {
    console.log("No visualizable graph data found in query results");
    return;
  }
  
  // Check if graph exceeds threshold
  if ((graphData.nodes.length + graphData.edges.length) > LARGE_GRAPH_THRESHOLD) {
    // Store data for large graph warning
    pendingGraphData.value = graphData;
    isExpandOperation.value = isExpansion;
    
    // Show warning dialog
    warningMessage.value = `This operation will visualize ${graphData.nodes.length} nodes and ${graphData.edges.length} edges (${graphData.nodes.length + graphData.edges.length} total elements), which may cause performance issues. Do you want to proceed?`;
    showWarningDialog.value = true;
    return;
  }
  
  // For small graphs, update directly
  updateGraph(cy, graphData, isExpansion);
  updateLabelVisibility();
  cy.fit();
};

// Handlers for warning dialog
const confirmLargeGraphVisualization = () => {
  if (pendingGraphData.value) {
    updateGraph(cy, pendingGraphData.value, isExpandOperation.value);
    // Apply label visibility settings after update
    updateLabelVisibility();
  }
  showWarningDialog.value = false;
  pendingGraphData.value = null;
};

const cancelLargeGraphVisualization = () => {
  showWarningDialog.value = false;
  // Don't clear pendingGraphData here - it's needed for the drawLimitedGraph function
};

// Function to expand node with a Cypher query
const expandNode = async (nodeData) => {
  console.log('Expanding node:', nodeData);
  
  // Hide the context menu
  contextMenuVisible.value = false;
  
  // Get the node ID and generate query
  const nodeId = nodeData.id;
  const cypher = generateNodeExpansionQuery(nodeId, onlyBasicACLs.value);
  
  try {
    // Execute the query
    console.log('Executing Cypher query:', cypher);
    const result = await executeCypherQuery(cypher);
    
    if (result && result.records) {
      console.log(`Found ${result.records.length} paths from node ${nodeId}`);
      // Process results
      processResults(result, true);
    } else {
      console.error('No results returned from expansion query');
    }
  } catch (error) {
    console.error('Error expanding node:', error);
  }
};

// Handle query from NodeDetails
const handleNodeDetailsQuery = (query, isExpansion = false) => {
  console.log(`Running query from NodeDetails (isExpansion: ${isExpansion})`, query);
  
  // Execute the query and process the results
  executeCypherQuery(query)
    .then(results => {
      // Set a flag to indicate this is from a saved query
      if (results && results.summary) {
        results.summary.fromSavedQuery = true;
      }
      
      // Process the results with expansion flag
      processResults(results, isExpansion);
    })
    .catch(error => {
      console.error("Error executing node details query:", error);
    });
};

// Set up graph events
const setupGraphEvents = () => {
  if (!cy) return;
  
  // Set up right-click (context menu) functionality
  cy.on('cxttap', 'node', function(event) {
    // Get the node that was right-clicked
    const node = event.target;
    const nodeData = node.data();
    
    // Store the selected node
    selectedNodeData.value = nodeData;
    
    // Calculate position for context menu
    const renderedPosition = event.renderedPosition;
    
    if (isFullscreen.value) {
      // In fullscreen mode, use the position directly
      contextMenuPosition.value = { 
        x: renderedPosition.x, 
        y: renderedPosition.y 
      };
    } else {
      // In normal mode, add container offset
      const containerRect = cyContainer.value.getBoundingClientRect();
      contextMenuPosition.value = { 
        x: containerRect.left + renderedPosition.x, 
        y: containerRect.top + renderedPosition.y 
      };
    }
    
    // Show the context menu
    contextMenuVisible.value = true;
  });
  
  // Add click handler for nodes - show details panel
  cy.on('tap', 'node', function(event) {
    // Get the node that was clicked
    const node = event.target;
    const nodeData = node.data();
    
    // Store the selected node data and show the details panel
    selectedNodeData.value = nodeData;
    nodeDetailsVisible.value = true;
    
    // Hide context menu if it's visible
    contextMenuVisible.value = false;
  });
  
  // Add background click handler to hide details panel when clicking on background
  cy.on('tap', function(event) {
    // Only hide when clicking directly on background (not on a node)
    if (event.target === cy) {
      // Don't hide details panel when clicking on background
      // We let the user control the panel visibility with the toggle button
    }
  });
};

// Set up document event listeners
const setupEventListeners = () => {
  // Set up escape key handler for fullscreen mode
  document.addEventListener('keydown', handleKeyDown);
  
  // Set up document click handler to close the context menu
  document.addEventListener('click', (e) => {
    if (contextMenuVisible.value && 
        !e.target.closest('.node-context-menu') && 
        !e.target.matches('node')) {
      contextMenuVisible.value = false;
    }
  });
  
  // Prevent default browser context menu
  document.addEventListener('contextmenu', (e) => {
    if (e.target.closest('.graph-container') || e.target.closest('.fullscreen-graph-container')) {
      e.preventDefault();
      return false;
    }
  });
};

// Remove document event listeners
const removeEventListeners = () => {
  document.removeEventListener('keydown', handleKeyDown);
  document.removeEventListener('click', () => {});
  document.removeEventListener('contextmenu', () => {});
};

onMounted(() => {
  // Initialize the graph in normal mode
  if (cyContainer.value) {
    cy = initializeGraph(cyContainer.value);
    
    // Set up graph events
    setupGraphEvents();
    
    
    // Apply initial label visibility
    updateLabelVisibility();
  }
  
  // Set up document-level event listeners
  setupEventListeners();
});

onUnmounted(() => {
  // Remove event listeners
  removeEventListeners();
  
  if (cy) {
    cy.destroy();
  }
});
</script>

<style>
/* Component-specific styles only */
.graph-container-wrapper {
  position: relative;
  width: 100%;
  height: 600px; /* Increased height for better visualization */
  border-radius: 8px;
  overflow: hidden;
  background-color: var(--color-graph-bg);
}

/* Fullscreen container styling */
.fullscreen-graph-container {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 9999;
  background-color: var(--color-graph-bg);
}

.graph-container {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

/* Controls container */
.graph-controls {
  position: absolute;
  bottom: 15px;
  right: 15px;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  z-index: 10000;
  gap: 8px;
}

.fullscreen-button {
  color: var(--color-text);
  background-color: var(--color-button-bg);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  width: 30px;
  height: 30px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.2s;
  box-shadow: 0 1px 3px var(--color-card-shadow);
}

.fullscreen-button:hover {
  background-color: var(--color-button-hover);
}

.fullscreen-icon {
  display: block;
  margin: auto;
}

/* No graph data message */
.no-graph-data {
  padding: 1rem;
  margin-top: 1rem;
  margin-bottom: 1rem;
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

/* Node context menu */
.node-context-menu {
  position: absolute;
  background-color: var(--color-card-bg);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  overflow: hidden;
  box-shadow: 0 2px 10px var(--color-card-shadow);
  z-index: 10001;
}

/* Loading indicator */
.loading-overlay {
  position: absolute;
  display: flex;
  justify-content: center;
  align-items: center;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(var(--color-bg-rgb, 255, 255, 255), 0.8);
  z-index: 10002;
}

.spinner {
  border: 3px solid var(--color-border-light);
  border-top: 3px solid var(--color-primary);
  border-radius: 50%;
  width: 30px;
  height: 30px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style> 