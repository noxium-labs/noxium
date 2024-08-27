use std::collections::{HashSet, HashMap};
use regex::Regex;

// Represents a node in the dependency graph
#[derive(Debug, Clone)]
struct Node {
    id: String,
    dependencies: HashSet<String>,
}

impl Node {
    // Creates a new node with the given ID
    fn new(id: &str) -> Self {
        Node {
            id: id.to_string(),
            dependencies: HashSet::new(),
        }
    }

    // Adds a dependency to this node
    fn add_dependency(&mut self, dependency: &str) {
        self.dependencies.insert(dependency.to_string());
    }
}

// Tree Shaker algorithm to remove unused nodes
fn tree_shaker(nodes: &HashMap<String, Node>, entry_points: &[&str]) -> HashSet<String> {
    let mut reachable = HashSet::new(); // Set to track reachable nodes
    let mut to_visit = entry_points.iter().map(|&id| id.to_string()).collect::<Vec<_>>(); // Nodes to visit

    while let Some(id) = to_visit.pop() {
        if reachable.insert(id.clone()) { // Mark the node as reachable
            if let Some(node) = nodes.get(&id) {
                for dep in &node.dependencies { // Check each dependency
                    if !reachable.contains(dep) { // If dependency is not already reachable
                        to_visit.push(dep.clone()); // Add to visit list
                    }
                }
            }
        }
    }

    reachable
}