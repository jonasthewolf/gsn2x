use std::cmp::min;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

pub trait DirectedGraphNodeType<'a> {
    // FIXME: is that really needed?
    fn is_final_node(&self) -> bool;
    fn get_forced_level(&self) -> Option<usize>;
    fn get_horizontal_index(&self, current_index: usize) -> Option<usize>;
}

pub trait DirectedGraphEdgeType<'a> {
    fn is_primary_child_edge(&self) -> bool;
    fn is_secondary_child_edge(&self) -> bool;
}

///
///
///
///
///
pub(super) struct DirectedGraph<'a, NodeType, EdgeType>
where
    NodeType: DirectedGraphNodeType<'a> + Sized,
    EdgeType: DirectedGraphEdgeType<'a> + Copy + Debug,
{
    nodes: &'a BTreeMap<String, NodeType>,
    edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
    root_nodes: Vec<&'a str>,
    parent_edges: BTreeMap<&'a str, Vec<(&'a str, EdgeType)>>,
}

impl<'a, NodeType, EdgeType> DirectedGraph<'a, NodeType, EdgeType>
where
    NodeType: DirectedGraphNodeType<'a> + Sized,
    EdgeType: DirectedGraphEdgeType<'a> + Copy + Debug,
{
    ///
    ///
    /// This graph object should know as little as possible about its nodes.
    ///
    pub fn new(
        nodes: &'a BTreeMap<String, NodeType>,
        edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
    ) -> Self {
        // Initialize node_info map
        let mut node_info = DirectedGraph {
            nodes,
            edges,
            root_nodes: vec![],
            parent_edges: BTreeMap::new(),
        };
        node_info.calculate_parent_edge_map();
        node_info.find_root_nodes();

        node_info
    }

    ///
    /// `edges` is a map of parent node id to a vector of edges (child, type)
    /// The returned parent map is a map of child node it to a vector of edges (parent, type)
    ///
    ///
    fn calculate_parent_edge_map(&mut self) {
        for (child, target_edges) in self.edges {
            for (target_edge, target_type) in target_edges {
                self.parent_edges
                    .entry(target_edge)
                    .or_default()
                    .push((child, *target_type));
            }
        }
    }

    ///
    /// Get a map of all inverse edges, i.e. children to their parents.
    ///
    ///
    pub fn get_parent_edges(&'a self) -> BTreeMap<&'a str, Vec<(&'a str, EdgeType)>> {
        self.parent_edges.to_owned()
    }

    ///
    /// Get the nodes of the graph.
    ///
    pub fn get_nodes(&'a self) -> &'a BTreeMap<String, NodeType> {
        self.nodes
    }

    ///
    /// Get the edges of the graph.
    ///
    pub fn get_edges(&'a self) -> &'a BTreeMap<String, Vec<(String, EdgeType)>> {
        self.edges
    }

    ///
    /// Identify the nodes with no incoming edges.
    /// They are called root nodes.
    ///
    ///
    fn find_root_nodes(&mut self) {
        // Copy IDs
        let mut n_ids: BTreeSet<&String> = self.nodes.keys().collect();
        // Find root nodes
        for t_edges in self.edges.values() {
            for (target, _) in t_edges {
                n_ids.remove(target);
            }
        }
        self.root_nodes = if n_ids.is_empty() {
            // No root nodes are found.
            // This can actually only happen in architecture view.
            // Take the first node and start from there.
            vec![self.nodes.iter().next().unwrap().0]
        } else {
            n_ids.iter().map(|rn| rn.as_str()).collect()
        };
    }

    ///
    /// Return the nodes that have no incoming nodes in the current graph.
    ///
    pub fn get_root_nodes(&'a self) -> Vec<&'a str> {
        self.root_nodes.to_vec()
    }

    ///
    /// Get the first cycle in the graph.
    /// If no cycle is found None is returned.
    /// TODO Can be simplified. Depth is not important.
    ///
    pub fn get_first_cycle(&'a self) -> Option<(&'a str, Vec<&'a str>)> {
        let mut stack = Vec::new();
        let mut ancestors = Vec::new();

        for &root in &self.root_nodes {
            stack.push((root, 0));
        }
        let mut depth = 0;
        while let Some((p_id, rdepth)) = stack.pop() {
            // Jump back to current ancestor
            if rdepth < depth {
                // It is not sufficient to pop here, since one could skip levels when cleaning up.
                ancestors.resize(rdepth, "");
                depth = rdepth;
            }
            // Increase depth if current node has children that are not Solutions
            if self
                .get_real_children(p_id)
                .iter()
                .filter(|&&x| !self.nodes.get(x).unwrap().is_final_node())
                .count()
                > 0
            {
                depth += 1;
                ancestors.push(p_id);
            }
            // unwrap is ok, since all references have been checked already
            for &child_node in self.get_real_children(p_id).iter() {
                if !self.nodes.get(child_node).unwrap().is_final_node() {
                    if ancestors.contains(&child_node) {
                        let mut reported_ancestors = Vec::from(
                            ancestors.rsplit(|&x| x == child_node).next().unwrap(), // unwrap is ok, since it is checked above that `ancestors` contains `child_node`
                        );
                        // Add nodes for reporting the found cycle
                        reported_ancestors.insert(0, child_node);
                        reported_ancestors.push(child_node);
                        return Some((p_id, reported_ancestors));
                    }
                    stack.push((&child_node, depth));
                }
            }
        }
        None
    }

    ///
    /// Get a list of unreachable nodes in the graph.
    /// TODO can be simplified
    ///
    pub fn get_unreachable_nodes(&'a self) -> Vec<&'a str> {
        let mut visited: BTreeSet<&str> = BTreeSet::new();
        let mut stack = Vec::new();
        let mut ancestors = Vec::new();

        for &root in &self.root_nodes {
            visited.insert(root);
            stack.push((root, 0));
        }
        let mut depth = 0;
        while let Some((p_id, rdepth)) = stack.pop() {
            // Jump back to current ancestor
            if rdepth < depth {
                // It is not sufficient to pop here, since one could skip levels when cleaning up.
                ancestors.resize(rdepth, "");
                depth = rdepth;
            }
            // Increase depth if current node has children that are not Solutions
            if self
                .get_real_children(p_id)
                .iter()
                .filter(|&&x| !self.nodes.get(x).unwrap().is_final_node())
                .count()
                > 0
            {
                depth += 1;
                ancestors.push(p_id);
            }
            // Remember the incontext elements for the reachability analysis below.
            self.get_same_rank_children(p_id).iter().for_each(|x| {
                visited.insert(x);
            });
            // unwrap is ok, since all references have been checked already
            for &child_node in self.get_real_children(p_id).iter() {
                // Remember the solutions for reachability analysis.
                visited.insert(child_node);
                if !self.nodes.get(child_node).unwrap().is_final_node() {
                    stack.push((&child_node, depth));
                }
            }
        }

        let node_keys: BTreeSet<&str> = BTreeSet::from_iter(self.nodes.keys().map(|k| k.as_str()));
        let unvisited: BTreeSet<&str> = node_keys.difference(&visited).copied().collect();

        unvisited.into_iter().collect()
    }

    ///
    /// Get children of `node` that are typically placed on the *same* rank.
    ///
    ///
    fn get_same_rank_children(&self, node: &str) -> Vec<&str> {
        self.edges
            .get(node)
            .iter()
            .flat_map(|&x| x)
            .filter_map(|(target, edge_type)| {
                if edge_type.is_secondary_child_edge() {
                    Some(target.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    ///
    /// Get children of `node´ that are typically placed on the *next* rank.
    ///
    ///
    pub fn get_real_children(&self, node: &str) -> Vec<&str> {
        self.edges
            .get(node)
            .iter()
            .flat_map(|&x| x)
            .filter_map(|(target, edge_type)| {
                if edge_type.is_primary_child_edge() {
                    Some(target.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    ///
    /// Rank all nodes
    ///
    /// The return value are the IDs of the ranked nodes in
    /// the following order: vertical, horizontal, cell
    ///
    pub fn rank_nodes(&self) -> Vec<Vec<Vec<&str>>> {
        let mut ranks = Vec::new();
        let mut visited = Vec::new();

        let mut carried_nodes;
        let mut forced_levels = self.get_forced_levels();

        let mut current_rank_nodes = self.root_nodes.to_vec();

        loop {
            // For each rank
            let mut next_rank_nodes: Vec<&str> = Vec::new();

            // Postpone ranking of forced nodes
            (current_rank_nodes, carried_nodes) =
                current_rank_nodes
                    .into_iter()
                    .partition(|n| match forced_levels.get_mut(n) {
                        Some(0) => true,
                        Some(forced_level) if *forced_level > 0 => {
                            // Reduce forced level
                            *forced_level -= 1;
                            false
                        }
                        Some(_) => unreachable!(),
                        None => true,
                    });
            // Visit current nodes
            current_rank_nodes.iter().for_each(|n| visited.push(*n));
            // Sort nodes lexicographically
            current_rank_nodes.sort();
            // Apply horizontal index movement
            for idx in 0..current_rank_nodes.len() {
                if let Some(new_idx) = self
                    .nodes
                    .get(current_rank_nodes.get(idx).unwrap().to_owned())
                    .unwrap()
                    .get_horizontal_index(idx)
                {
                    // FIXME Some checks are missing
                    let child = current_rank_nodes.remove(idx);
                    current_rank_nodes.insert(new_idx, child);
                }
            }

            for parent_node in current_rank_nodes.iter() {
                // Get children of current parent
                let (mut children, mut child_carried_nodes) =
                    self.get_next_rank_children_of_parent(parent_node, &visited);
                // Append all children to next rank
                next_rank_nodes.append(&mut children);
                carried_nodes.append(&mut child_carried_nodes);
            }

            if current_rank_nodes.is_empty() {
                break;
            } else {
                let current_rank = self.add_same_rank_nodes(current_rank_nodes);
                ranks.push(current_rank);
                current_rank_nodes = next_rank_nodes;
                current_rank_nodes.append(&mut carried_nodes);
            }
        }
        ranks
    }

    ///
    ///
    ///
    ///
    ///
    fn add_same_rank_nodes<'b>(&'b self, current_rank_nodes: Vec<&'b str>) -> Vec<Vec<&str>> {
        let mut current_rank: Vec<Vec<&str>> =
            current_rank_nodes.iter().map(|&n| vec![n]).collect();
        // Add inContext nodes
        for index in 0..current_rank_nodes.len() {
            let same_rank_parent = current_rank_nodes.get(index).unwrap().to_owned();
            let (left, right): (Vec<_>, Vec<_>) = self
                .get_same_rank_children(same_rank_parent)
                .into_iter()
                .enumerate()
                .partition(|(idx, _)| idx % 2 == 0);
            let mut parent_index = current_rank
                .iter()
                .position(|x| x.contains(&same_rank_parent))
                .unwrap();
            if !left.is_empty() {
                current_rank.insert(parent_index, left.iter().map(|(_, x)| *x).collect());
                parent_index += 1;
            }
            if !right.is_empty() {
                current_rank.insert(
                    min(parent_index + 1, current_rank.len()),
                    right.iter().map(|(_, x)| *x).collect(),
                );
            }
        }
        current_rank
    }

    ///
    /// Get next rank children of `parent_node`` that are not `visited` yet.
    /// (Children scheduled for ranking, Children that are not yet ready)
    ///
    fn get_next_rank_children_of_parent(
        &self,
        parent_node: &str,
        visited: &[&str],
    ) -> (Vec<&str>, Vec<&str>) {
        self.edges
            .get(parent_node)
            .iter()
            .flat_map(|&x| x)
            .filter_map(|(target, edge_type)| {
                // Find next rank children
                if edge_type.is_primary_child_edge() {
                    Some(target.as_str())
                } else {
                    None
                }
            })
            .filter(|n| !visited.contains(n)) // Remove already visited nodes again
            .partition(|n| {
                // Remove nodes that have not all parents visited yet
                self.parent_edges
                    .get(n)
                    .unwrap()
                    .iter()
                    .filter(|(_, et)| et.is_primary_child_edge())
                    .all(|(p, _)| visited.contains(p))
            })
    }

    ///
    /// Collect a map of node IDs and their forced vertical rank increment.
    /// A node is not added if no forced level is set.
    ///
    fn get_forced_levels(&self) -> BTreeMap<&str, usize> {
        self.nodes
            .iter()
            .filter_map(|(id, n)| {
                n.get_forced_level()
                    .map(|forced_level| (id.as_str(), forced_level))
            })
            .collect()
    }
}

///
/// Debug display of DirectedGraph
///
///
impl<'a, NodeType, EdgeType> std::fmt::Debug for DirectedGraph<'a, NodeType, EdgeType>
where
    NodeType: DirectedGraphNodeType<'a> + Sized,
    EdgeType: DirectedGraphEdgeType<'a> + Copy + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ranks = self.rank_nodes();

        for rank in ranks {
            let max_lines = rank.iter().map(|x| x.len()).max().unwrap_or(1);
            for idx in 0..max_lines {
                let line = rank
                    .iter()
                    .filter_map(|x| x.get(idx))
                    .map(|&x| x.to_owned())
                    .collect::<Vec<String>>()
                    .join(" | ");
                f.write_fmt(format_args!("{line}\n"))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {}
