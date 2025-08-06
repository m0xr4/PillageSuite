import cytoscape from 'cytoscape';

// Initialize cytoscape graph
export const initializeGraph = (container) => {
  const cy = cytoscape({
    container: container,
    style: [
      {
        selector: 'node',
        style: {
          'label': 'data(label)',
          'text-wrap': 'wrap',
          'text-max-width': '100px',
          'font-size': '12px',
          'color': '#fff',
          'text-outline-width': 2,
          'text-outline-color': '#1a1a1a',
          'background-color': '#4338ca', // Default color
          'width': 40,
          'height': 40,
          'text-valign': 'center'
        }
      },
      // User nodes - Blue
      {
        selector: 'node[labelString *= "User"]',
        style: {
          'background-color': '#3b82f6', // Blue
          'width': 45,
          'height': 45
        }
      },
      // Group nodes - Green  
      {
        selector: 'node[labelString *= "Group"]',
        style: {
          'background-color': '#10b981', // Emerald green
          'width': 45,
          'height': 45
        }
      },
      // Share nodes - Purple
      {
        selector: 'node[labelString *= "share"]',
        style: {
          'background-color': '#8b5cf6', // Purple
          'width': 40,
          'height': 40
        }
      },
      // File nodes - Orange
      {
        selector: 'node[labelString *= "file"]',
        style: {
          'background-color': '#f59e0b', // Amber/Orange
          'width': 35,
          'height': 35
        }
      },
      // Directory nodes - Red
      {
        selector: 'node[labelString *= "directory"]',
        style: {
          'background-color': '#ef4444', // Red
          'width': 40,
          'height': 40
        }
      },
      {
        selector: 'edge',
        style: {
          'label': 'data(label)',
          'text-rotation': 'autorotate',
          'text-margin-y': -10,
          'text-background-color': '#fff',
          'text-background-opacity': 0.7,
          'text-background-padding': '2px',
          'width': 2,
          'line-color': '#666',
          'target-arrow-color': '#666',
          'target-arrow-shape': 'triangle',
          'curve-style': 'bezier'
        }
      },
      
    ],
    layout: {
      name: 'cose',
      idealEdgeLength: 100,
      nodeOverlap: 20,
      refresh: 20,
      fit: true,
      padding: 30,
      randomize: false,
      componentSpacing: 100,
      nodeRepulsion: 400000,
      edgeElasticity: 100,
      nestingFactor: 5,
      gravity: 80,
      numIter: 1000,
      initialTemp: 200,
      coolingFactor: 0.95,
      minTemp: 1.0
    }
  });
  
  return cy;
};

// Update the graph with processed data
export const updateGraph = (cy, graphData, isExpansion = false) => {
  if (!cy) return;
  
  console.log(`Updating graph with ${graphData.nodes.length} nodes and ${graphData.edges.length} edges`);
  
  // Clear existing graph if not an expansion operation
  if (!isExpansion) {
    clearGraph(cy);
  }
  
  if (graphData.nodes.length === 0) {
    console.log("No nodes to add to graph");
    return;
  }
  
  // Add nodes to the graph
  addNodesToGraph(cy, graphData.nodes);
  
  // Add edges to the graph
  addEdgesToGraph(cy, graphData.edges);
  
  // Apply layout only if not an expansion operation
  if (!isExpansion) {
    applyGraphLayout(cy);
  } else {
    // For expansion, just fit the graph
    cy.fit();
  }
  
  console.log("Graph update completed");
};

// Clear existing graph elements
export const clearGraph = (cy) => {
  try {
    cy.elements().remove();
  } catch (error) {
    console.error('Error clearing graph:', error);
    return null;
  }
};

// Add nodes to the graph with random positions
export const addNodesToGraph = (cy, nodes) => {
  // Create a Set of existing node IDs
  const existingNodeIds = new Set();
  cy.nodes().forEach(node => {
    existingNodeIds.add(node.id());
  });
  
  // Add new nodes that don't already exist
  nodes.forEach(node => {
    if (!existingNodeIds.has(node.id)) {
      const nodeData = {
        id: node.id,
        label: node.label,
        properties: node.properties,
        labels: node.labels // Keep original array for NodeDetails component
      };
      
      // Add data attributes for styling if they exist
      if (node.data) {
        if (node.data.type) nodeData.type = node.data.type;
        if (node.data.search) nodeData.search = node.data.search;
        if (node.data.name) nodeData.name = node.data.name;
        if (node.data.title) nodeData.title = node.data.title;
      }
      
      // Create a labelString for cytoscape selectors while keeping original labels array
      if (node.labels && Array.isArray(node.labels)) {
        nodeData.labelString = node.labels.join(' '); // For cytoscape selectors
      }
      
      cy.add({
        group: 'nodes',
        data: nodeData,
        position: { 
          x: Math.random() * 500, 
          y: Math.random() * 500 
        }
      });
    }
  });
};

// Add edges to the graph
export const addEdgesToGraph = (cy, edges) => {
  // Prepare edges for cytoscape - only add edges where source and target exist
  const edgesToAdd = edges.filter(edge => 
    cy.getElementById(edge.source).length > 0 && 
    cy.getElementById(edge.target).length > 0
  ).map(edge => ({
    data: edge
  }));
  
  // Add edges to graph
  if (edgesToAdd.length > 0) {
    cy.add(edgesToAdd);
  }
};

// Apply layout and center the graph
export const applyGraphLayout = (cy) => {
  // Apply layout
  cy.layout({
    name: 'cose',
    idealEdgeLength: 150,
    nodeOverlap: 20,
    refresh: 20,
    fit: true,
    padding: 50,
    randomize: true,
    componentSpacing: 150,
    nodeRepulsion: 500000,
    edgeElasticity: 100,
    nestingFactor: 5,
    gravity: 150,
    numIter: 1000,
    initialTemp: 250,
    coolingFactor: 0.95,
    minTemp: 1.0
  }).run();
  
  // Center and fit graph
  setTimeout(() => {
    cy.fit();
    cy.center();
    cy.style().update();
  }, 500);
};


// Generate Cypher query for node expansion
export const generateNodeExpansionQuery = (nodeId, onlyBasicACLs = true) => {
  if (onlyBasicACLs) {
    // Filter relationships to only include basic ACL types
    return `
      MATCH p=(n)-[r]-(related)
      WHERE (id(n) = ${nodeId} OR n.id = "${nodeId}")
      AND type(r) IN ["FullControl", "Modify", "ReadAndExecute", "ReadAndWrite", "Read", "Write", "ReadData/ListDirectory", "MEMBER_OF"]
      RETURN p limit 250
    `;
  } else {
    // Return all relationship types
    return `
      MATCH p=(n)-[r]-(related)
      WHERE id(n) = ${nodeId} OR n.id = "${nodeId}"
      RETURN p limit 250
    `;
  }
};

// Limit graph data to threshold
export const limitGraphData = (graphData, threshold, isExpandOperation = false, selectedNodeData = null) => {
  if (!graphData) return { nodes: [], edges: [] };
  
  const totalElements = graphData.nodes.length + graphData.edges.length;
  if (totalElements <= threshold) {
    return graphData; // No need to limit
  }
  
  console.log(`Limiting graph data from ${totalElements} to ${threshold} elements`);
  
  // When expanding a node, make sure to keep the source node and its immediate connections
  if (isExpandOperation && selectedNodeData) {
    const sourceNodeId = selectedNodeData.id;
    
    // Ensure the source node is kept
    const sourceNode = graphData.nodes.find(node => node.id === sourceNodeId);
    
    // Find all edges connected to the source node
    const connectedEdges = graphData.edges.filter(edge => 
      edge.source === sourceNodeId || edge.target === sourceNodeId
    );
    
    // Get IDs of nodes connected to the source node
    const connectedNodeIds = new Set();
    connectedEdges.forEach(edge => {
      if (edge.source === sourceNodeId) {
        connectedNodeIds.add(edge.target);
      } else if (edge.target === sourceNodeId) {
        connectedNodeIds.add(edge.source);
      }
    });
    
    // Prioritize keeping these connected nodes
    const prioritizedNodes = [sourceNode];
    const remainingNodes = [];
    
    graphData.nodes.forEach(node => {
      if (node.id !== sourceNodeId) {
        if (connectedNodeIds.has(node.id)) {
          prioritizedNodes.push(node);
        } else {
          remainingNodes.push(node);
        }
      }
    });
    
    // Calculate how many more nodes we can include
    const spaceForRemainingNodes = Math.max(0, threshold - prioritizedNodes.length - connectedEdges.length);
    const additionalNodes = remainingNodes.slice(0, spaceForRemainingNodes);
    
    // Combine prioritized and additional nodes
    const limitedNodes = [...prioritizedNodes, ...additionalNodes];
    
    // Get all valid edges (connecting nodes that are kept)
    const nodeIdSet = new Set(limitedNodes.map(n => n.id));
    
    // Keep all edges connected to the source node, then add other valid edges up to the limit
    const prioritizedEdges = graphData.edges.filter(edge => 
      edge.source === sourceNodeId || edge.target === sourceNodeId
    );
    
    const remainingEdges = graphData.edges.filter(edge => 
      edge.source !== sourceNodeId && 
      edge.target !== sourceNodeId &&
      nodeIdSet.has(edge.source) && 
      nodeIdSet.has(edge.target)
    );
    
    const spaceForRemainingEdges = Math.max(0, threshold - limitedNodes.length - prioritizedEdges.length);
    const limitedEdges = [
      ...prioritizedEdges,
      ...remainingEdges.slice(0, spaceForRemainingEdges)
    ];
    
    console.log(`Limited to ${limitedNodes.length} nodes and ${limitedEdges.length} edges`);
    return { nodes: limitedNodes, edges: limitedEdges };
  }
  
  // Standard limiting logic for non-expansion operations
  const limitedNodes = [...graphData.nodes];
  const limitedEdges = [...graphData.edges];
  
  // Calculate how many elements we need to remove
  let elementsToRemove = totalElements - threshold;
  
  // Remove edges first, but keep at least one edge per node
  if (elementsToRemove > 0 && limitedEdges.length > 0) {
    // Calculate how many edges we can safely remove
    const maxEdgesToRemove = Math.min(elementsToRemove, limitedEdges.length - limitedNodes.length);
    if (maxEdgesToRemove > 0) {
      limitedEdges.splice(-maxEdgesToRemove);
      elementsToRemove -= maxEdgesToRemove;
    }
  }
  
  // If we still need to remove elements, remove nodes
  if (elementsToRemove > 0 && limitedNodes.length > 0) {
    // Calculate how many nodes we can safely remove
    const maxNodesToRemove = Math.min(elementsToRemove, limitedNodes.length - 1);
    if (maxNodesToRemove > 0) {
      limitedNodes.splice(-maxNodesToRemove);
    }
  }
  
  // ALWAYS ensure we only keep edges that connect remaining nodes
  // This needs to happen regardless of whether we removed nodes or just edges
  const remainingNodeIds = new Set(limitedNodes.map(node => node.id));
  const validEdges = limitedEdges.filter(edge => 
    remainingNodeIds.has(edge.source) && remainingNodeIds.has(edge.target)
  );
  
  return { nodes: limitedNodes, edges: validEdges };
};

// Process query results and update graph with large graph safeguards
export const processAndUpdateGraph = (cy, results, threshold, isExpansion = false, selectedNodeData = null, callbacks = {}) => {
  if (!results || !cy) return;
  
  // Extract callback functions with defaults
  const { 
    onLargeGraph = () => {}, 
    onGraphUpdated = () => {},
    processGraphData = () => ({ nodes: [], edges: [] })
  } = callbacks;
  
  // Process graph data
  const graphData = processGraphData(results);
  const totalElements = graphData.nodes.length + graphData.edges.length;
  
  // Check if graph exceeds threshold
  if (totalElements > threshold) {
    // Invoke callback for large graph warning
    onLargeGraph(graphData, totalElements);
    return graphData;
  } else {
    // Safe to proceed, update graph
    updateGraph(cy, graphData, isExpansion);
    
    // Invoke callback after graph update
    onGraphUpdated();
    
    return graphData;
  }
}; 