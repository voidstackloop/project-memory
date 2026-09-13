use crate::types::Memory;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct MemoryGraph {
    adj: HashMap<String, Vec<String>>,
}

impl MemoryGraph {
    pub fn from_memories(memories: &[Memory]) -> Self {
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        
        for m in memories {
            adj.entry(m.id.clone()).or_default();
            for rid in &m.related_ids {
                adj.entry(m.id.clone()).or_default().push(rid.clone());
                adj.entry(rid.clone()).or_default().push(m.id.clone());
            }
        }
        
        Self { adj }
    }
    
    pub fn shortest_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        if from == to {
            return Some(vec![from.to_string()]);
        }
        
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, Vec<String>)> = VecDeque::new();
        
        visited.insert(from.to_string());
        queue.push_back((from.to_string(), vec![from.to_string()]));
        
        while let Some((current, path)) = queue.pop_front() {
            if let Some(neighbors) = self.adj.get(&current) {
                for neighbor in neighbors {
                    if neighbor == to {
                        let mut result = path;
                        result.push(neighbor.clone());
                        return Some(result);
                    }
                    
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        let mut new_path = path.clone();
                        new_path.push(neighbor.clone());
                        queue.push_back((neighbor.clone(), new_path));
                    }
                }
            }
        }
        
        None
    }
    
    pub fn connected_components(&self) -> Vec<Vec<String>> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut components = Vec::new();
        
        for node in self.adj.keys() {
            if !visited.contains(node) {
                let mut component = Vec::new();
                let mut queue = VecDeque::new();
                
                visited.insert(node.clone());
                queue.push_back(node.clone());
                component.push(node.clone());
                
                while let Some(current) = queue.pop_front() {
                    if let Some(neighbors) = self.adj.get(&current) {
                        for neighbor in neighbors {
                            if !visited.contains(neighbor) {
                                visited.insert(neighbor.clone());
                                queue.push_back(neighbor.clone());
                                component.push(neighbor.clone());
                            }
                        }
                    }
                }
                
                components.push(component);
            }
        }
        
        components.sort_by(|a, b| b.len().cmp(&a.len()));
        components
    }
    
    pub fn neighbors(&self, node: &str) -> Vec<String> {
        self.adj.get(node).cloned().unwrap_or_default()
    }
    
    pub fn degree(&self, node: &str) -> usize {
        self.adj.get(node).map_or(0, |n| n.len())
    }
    
    pub fn nodes(&self) -> Vec<String> {
        self.adj.keys().cloned().collect()
    }
    
    pub fn edges(&self) -> usize {
        self.adj.values().map(|n| n.len()).sum::<usize>() / 2
    }
    
    pub fn isolated_nodes(&self) -> Vec<String> {
        self.adj.iter()
            .filter(|(_, neighbors)| neighbors.is_empty())
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    pub fn hub_nodes(&self, min_degree: usize) -> Vec<(String, usize)> {
        let mut hubs: Vec<(String, usize)> = self.adj.iter()
            .filter(|(_, neighbors)| neighbors.len() >= min_degree)
            .map(|(id, neighbors)| (id.clone(), neighbors.len()))
            .collect();
        
        hubs.sort_by(|a, b| b.1.cmp(&a.1));
        hubs
    }
}

pub fn format_graph_stats(graph: &MemoryGraph) -> String {
    let mut output = String::new();
    
    output.push_str("Graph Statistics\n");
    output.push_str("================\n\n");
    output.push_str(&format!("Nodes: {}\n", graph.nodes().len()));
    output.push_str(&format!("Edges: {}\n", graph.edges()));
    output.push_str(&format!("Components: {}\n", graph.connected_components().len()));
    output.push_str(&format!("Isolated nodes: {}\n", graph.isolated_nodes().len()));
    
    let hubs = graph.hub_nodes(2);
    if !hubs.is_empty() {
        output.push_str("\nHub nodes (degree >= 2):\n");
        for (id, degree) in hubs.iter().take(5) {
            output.push_str(&format!("  {}: {} connections\n", &id[..8.min(id.len())], degree));
        }
    }
    
    output
}
