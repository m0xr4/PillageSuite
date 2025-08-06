// Neo4j Service - Centralized handling of Neo4j operations
import neo4j from 'neo4j-driver';

let driver = null;
let session = null;
// Store connection details in memory (not persisted to storage)
let connectionDetails = {
  uri: null,
  username: null,
  password: null
};

// Connect to Neo4j database
export const connectToNeo4j = async (uri, username, password) => {
  // Store connection details in memory for later use by other components
  connectionDetails = { uri, username, password };

  try {
    // Close any existing connection
    if (session) {
      await session.close();
    }
    if (driver) {
      await driver.close();
    }

    // Create a new driver instance
    driver = neo4j.driver(
      uri,
      neo4j.auth.basic(username, password),
      { disableLosslessIntegers: true }
    );

    // Verify the connection
    session = driver.session();
    const result = await session.run('RETURN 1 as n');
    
    if (result.records.length > 0) {
      return { success: true, driver, session };
    }
    
    return { success: false, error: 'Connection failed' };
  } catch (error) {
    console.error('Connection error:', error);
    return { success: false, error: error.message };
  }
};

// Store active query execution to allow termination
let activeQueryExecution = null;

// Execute a Cypher query and return the results
export const executeCypherQuery = async (cypher, customSession = null) => {
  if (!driver && !customSession) {
    throw new Error('No active Neo4j connection');
  }
  
  const sessionToUse = customSession || driver.session();
  
  try {
    // Store the active execution for potential termination
    activeQueryExecution = {
      session: sessionToUse,
      running: true
    };
    
    const result = await sessionToUse.run(cypher);
    activeQueryExecution.running = false;
    return result;
  } catch (error) {
    activeQueryExecution.running = false;
    console.error('Error executing Cypher query:', error);
    throw error;
  } finally {
    if (!customSession) {
      await sessionToUse.close();
    }
  }
};

// Terminate the currently running query
export const terminateQuery = async () => {
  if (activeQueryExecution && activeQueryExecution.running) {
    try {
      // Get the list of current transactions
      const statusSession = driver.session();
      const statusResult = await statusSession.run('SHOW TRANSACTIONS');
      
      // Find transactions that might be our currently running query
      // Filter out the SHOW TRANSACTIONS query itself
      const transactions = statusResult.records
        .filter(record => !record.get('currentQuery').includes('SHOW TRANSACTIONS'))
        .map(record => ({
          id: record.get('transactionId'),
          query: record.get('currentQuery'),
          elapsedTime: record.get('elapsedTime')
        }))
        .sort((a, b) => b.elapsedTime - a.elapsedTime); // Most recent first
      
      await statusSession.close();
      
      if (transactions.length > 0) {
        // Terminate the longest-running transaction (most likely our query)
        const txToTerminate = transactions[0].id;
        console.log(`Terminating transaction ${txToTerminate}`);
        
        const terminateSession = driver.session();
        await terminateSession.run(`TERMINATE TRANSACTIONS '${txToTerminate}'`);
        await terminateSession.close();
        
        activeQueryExecution.running = false;
        return { success: true };
      } else {
        return { success: false, error: 'No active transactions found to terminate' };
      }
    } catch (error) {
      console.error('Error terminating query:', error);
      return { success: false, error: error.message };
    }
  }
  return { success: false, error: 'No active query to terminate' };
};

// Process Neo4j results for graph visualization
export const processGraphData = (results) => {
  if (!results || !results.records) {
    console.log("No records to process in processGraphData");
    return { nodes: [], edges: [], hasGraphData: false };
  }
  
  console.log("Processing records for visualization:", results.records.length);
  if (results.records.length > 0) {
    console.log("First record keys:", results.records[0].keys);
  }
  
  const nodes = new Map();
  const edges = new Map();
  
  // Track if there's any valid graph-visualizable data
  let hasGraphData = false;
  
  // Process records to extract nodes and relationships
  results.records.forEach((record) => {
    // Debug the record content
    console.log("Processing record with keys:", record.keys);
    
    record.keys.forEach(key => {
      const value = record.get(key);
      
      // Debug the value for this key
      console.log(`Examining key ${key}:`, value ? (typeof value === 'object' ? 'object' : value) : 'null/undefined');
      
      // Process paths
      if (value && typeof value === 'object' && value.segments && Array.isArray(value.segments)) {
        hasGraphData = true;
        value.segments.forEach((segment) => {
          // Process start node
          const startNode = segment.start;
          const startNodeId = startNode.identity.toString();
          
          if (!nodes.has(startNodeId)) {
            const nodeLabel = startNode.properties.name || 
                            startNode.properties.title || 
                            (startNode.labels && startNode.labels[0]) || 
                            'Node_' + startNodeId;
            
            nodes.set(startNodeId, {
              id: startNodeId,
              label: nodeLabel,
              properties: startNode.properties,
              labels: startNode.labels
            });
          }
          
          // Process end node
          const endNode = segment.end;
          const endNodeId = endNode.identity.toString();
          
          if (!nodes.has(endNodeId)) {
            const nodeLabel = endNode.properties.name || 
                            endNode.properties.title || 
                            (endNode.labels && endNode.labels[0]) || 
                            'Node_' + endNodeId;
            
            nodes.set(endNodeId, {
              id: endNodeId,
              label: nodeLabel,
              properties: endNode.properties,
              labels: endNode.labels
            });
          }
          
          // Process relationship
          const rel = segment.relationship;
          const edgeId = rel.identity.toString();
          
          if (!edges.has(edgeId)) {
            const relStartId = rel.start.toString();
            const relEndId = rel.end.toString();
            
            edges.set(edgeId, {
              id: 'e' + edgeId,
              source: relStartId,
              target: relEndId,
              label: rel.type || 'RELATED_TO',
              properties: rel.properties
            });
          }
        });
      }
      // Handle individual nodes
      else if (value && typeof value === 'object' && value.labels && Array.isArray(value.labels)) {
        hasGraphData = true;
        const nodeId = value.identity.toString();
        
        if (!nodes.has(nodeId)) {
          // Enhanced node label selection for better display, especially for LiveSearch results
          const nodeName = value.properties.name || '';
          const nodeTitle = value.properties.title || '';
          const nodeLabel = nodeName || nodeTitle || (value.labels && value.labels[0]) || 'Node_' + nodeId;
          
          // Add a node with more styling data for better visualization
          nodes.set(nodeId, {
            id: nodeId,
            label: nodeLabel,
            properties: value.properties,
            labels: value.labels,
            // Add metadata for styling in the graph
            data: {
              type: value.labels && value.labels[0],
              search: record.keys[0] === 'n' && record.keys.length === 1, // Mark nodes from search
              name: nodeName,
              title: nodeTitle,
              id: nodeId
            }
          });
        }
        
        // If we have multiple nodes in the result but no relationships (like in the relationship query issue)
        // Try to see if other keys in the record contain relationships or target nodes
        if (record.keys.length > 1) {
          for (const otherKey of record.keys) {
            if (otherKey !== key) {
              const otherValue = record.get(otherKey);
              
              // Check if the other value is a relationship
              if (otherValue && typeof otherValue === 'object' && otherValue.type && typeof otherValue.type === 'string') {
                // We have a relationship
                const edgeId = otherValue.identity.toString();
                
                if (!edges.has(edgeId)) {
                  edges.set(edgeId, {
                    id: 'e' + edgeId,
                    source: otherValue.start.toString(),
                    target: otherValue.end.toString(),
                    label: otherValue.type || 'RELATED_TO',
                    properties: otherValue.properties
                  });
                }
              }
              // Check if the other value is a node (could be target node)
              else if (otherValue && typeof otherValue === 'object' && otherValue.labels && Array.isArray(otherValue.labels)) {
                const targetNodeId = otherValue.identity.toString();
                
                if (!nodes.has(targetNodeId)) {
                  const nodeLabel = otherValue.properties.name || 
                                  otherValue.properties.title || 
                                  (otherValue.labels && otherValue.labels[0]) || 
                                  'Node_' + targetNodeId;
                  
                  nodes.set(targetNodeId, {
                    id: targetNodeId,
                    label: nodeLabel,
                    properties: otherValue.properties,
                    labels: otherValue.labels
                  });
                }
              }
            }
          }
        }
      }
      // Handle individual relationships
      else if (value && typeof value === 'object' && value.type && typeof value.type === 'string') {
        hasGraphData = true;
        const edgeId = value.identity.toString();
        
        if (!edges.has(edgeId)) {
          edges.set(edgeId, {
            id: 'e' + edgeId,
            source: value.start.toString(),
            target: value.end.toString(),
            label: value.type || 'RELATED_TO',
            properties: value.properties
          });
        }
      }
      // Check for special cases - the key can give us a hint about the node
      else if (key.includes('.') && key.split('.').length === 2) {
        // This might be a node property (e.g., "n.name")
        // We can't visualize this directly, but we don't need to do anything
        // The visualization will be empty in this case
      }
    });
  });

  // If we have only one node (like in LiveSearch results), ensure hasGraphData is true
  if (nodes.size === 1 && edges.size === 0) {
    hasGraphData = true;
  }
  
  // Log graph visualization data for debugging
  console.log(`Graph data processed: ${nodes.size} nodes, ${edges.size} edges, hasGraphData: ${hasGraphData}`);
  
  // If we're returning primitive values only, the graph will be empty
  return {
    nodes: Array.from(nodes.values()),
    edges: Array.from(edges.values()),
    // Add flag to indicate if we have graph-visualizable data
    hasGraphData: hasGraphData
  };
};

// Get driver instance
export const getDriver = () => driver;

// Get connection details - only returns data if there's an active connection
export const getConnectionDetails = () => {
  if (!driver || !connectionDetails.uri) {
    return null;
  }
  return { ...connectionDetails };
};

// Close all connections
export const closeConnections = async () => {
  if (session) {
    await session.close();
    session = null;
  }
  
  if (driver) {
    await driver.close();
    driver = null;
  }
  
  // Clear connection details
  connectionDetails = {
    uri: null,
    username: null,
    password: null
  };
}; 