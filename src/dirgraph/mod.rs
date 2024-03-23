use std::cmp::min;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

///
/// Trait to be implemented for nodes of `DirectedGraph`.
///
pub trait DirectedGraphNodeType<'a> {
    ///
    /// Get the forced vertical rank increment, if any.
    ///
    /// Returns `None` if there is no forced index.
    ///
    fn get_forced_level(&self) -> Option<usize>;

    ///
    /// Get the forced horizontal index, if any.
    /// `current_index` gives in the current index of the node.
    ///
    /// Returns `None` if there is no forced index.
    ///
    fn get_horizontal_index(&self, current_index: usize) -> Option<usize>;
}

///
/// Trait to be implemented for edges of `DirectedGraph`.
///
pub trait DirectedGraphEdgeType<'a> {
    ///
    /// Returns true if the edge points to a primary (i.e. **next** rank) child.
    /// Otherwise false.
    ///
    fn is_primary_child_edge(&self) -> bool;

    ///
    /// Returns true if the edge points to a secondary (i.e. **same** rank) child.
    /// Otherwise false.
    ///
    fn is_secondary_child_edge(&self) -> bool;
}

///
/// The structure to rank nodes on a graph.
/// The graph is described by nodes and the edges between them.
/// Nodes and edges must be
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
    edge_decorators: BTreeMap<(String, String), Vec<String>>,
}

impl<'a, NodeType, EdgeType> DirectedGraph<'a, NodeType, EdgeType>
where
    NodeType: DirectedGraphNodeType<'a> + Sized,
    EdgeType: DirectedGraphEdgeType<'a> + Copy + Debug,
{
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
            edge_decorators: BTreeMap::new(),
        };
        node_info.calculate_parent_edge_map();
        node_info.find_root_nodes();

        node_info
    }

    pub fn add_edge_decorators(
        &mut self,
        edge_decorators: BTreeMap<(String, String), Vec<String>>,
    ) {
        self.edge_decorators = edge_decorators;
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
    /// Get the nodes of the graph.
    ///
    pub fn get_nodes(&self) -> &'a BTreeMap<String, NodeType> {
        self.nodes
    }

    ///
    /// Get the edges of the graph.
    ///
    pub fn get_edges(&self) -> &'a BTreeMap<String, Vec<(String, EdgeType)>> {
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
            // Unwrap is ok, since there is at least one node
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
    ///
    pub fn get_first_cycle(&'a self) -> Option<(&'a str, Vec<&'a str>)> {
        let mut stack = self.root_nodes.iter().map(|&n| (n, 0)).collect::<Vec<_>>();
        let mut ancestors = Vec::new();
        let mut depth = 0;
        'cycle_found: {
            while let Some((p_id, rdepth)) = stack.pop() {
                // Jump back to current ancestor
                if rdepth < depth {
                    ancestors.truncate(rdepth);
                    depth = rdepth;
                }
                // Remember the current node if it has no other real children
                if self
                    .get_real_children(p_id)
                    .iter()
                    .filter(|&&x| !self.get_real_children(x).is_empty())
                    .count()
                    > 0
                {
                    depth += 1;
                    ancestors.push(p_id);
                }
                for &child_node in self.get_real_children(p_id).iter() {
                    if !self.get_real_children(child_node).is_empty() {
                        if ancestors.contains(&child_node) {
                            let mut reported_ancestors = Vec::from(
                                ancestors.rsplit(|&x| x == child_node).next().unwrap(), // unwrap is ok, since it is checked above that `ancestors` contains `child_node`
                            );
                            // Add nodes for reporting the found cycle
                            reported_ancestors.insert(0, child_node);
                            reported_ancestors.push(child_node);
                            break 'cycle_found Some((p_id, reported_ancestors));
                        }
                        stack.push((&child_node, depth));
                    }
                }
            }
            None
        }
    }

    ///
    /// Get a list of unreachable nodes in the graph.
    ///
    pub fn get_unreachable_nodes(&'a self) -> Vec<&'a str> {
        let mut ranks = Vec::new();
        // Using BTreeSet here to ensure that reporting is always in the same order.
        let mut visited = BTreeSet::new();

        let mut current_rank_nodes = self.root_nodes.to_vec();

        loop {
            // For each rank
            let mut next_rank_nodes: Vec<&str> = Vec::new();
            // Visit current nodes
            current_rank_nodes.iter().for_each(|n| {
                visited.insert(*n);
            });
            // Find children for next rank
            for parent_node in current_rank_nodes.iter() {
                // Get children of current parent
                let (mut children, mut child_carried_nodes) =
                    self.get_next_rank_children_of_parent(parent_node, &visited);
                // Append all children to next rank
                next_rank_nodes.append(&mut children);
                next_rank_nodes.append(&mut child_carried_nodes);
            }

            if current_rank_nodes.is_empty() {
                break;
            } else {
                let current_rank = self.add_same_rank_nodes(current_rank_nodes, &mut visited);
                ranks.push(current_rank);
                current_rank_nodes = next_rank_nodes
                    .into_iter()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
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
    /// Get parents of `node´ that are typically placed on the *next* rank.
    ///
    ///
    pub fn get_real_parents(&self, node: &str) -> Vec<&str> {
        self.parent_edges
            .get(node)
            .iter()
            .flat_map(|&x| x)
            .filter_map(|(target, edge_type)| {
                if edge_type.is_primary_child_edge() {
                    Some(*target)
                } else {
                    None
                }
            })
            .collect()
    }

    ///
    /// Get parents of `node´ that are typically placed on the *same* rank.
    ///
    ///
    pub fn get_same_ranks_parents(&self, node: &str) -> Vec<&str> {
        self.parent_edges
            .get(node)
            .iter()
            .flat_map(|&x| x)
            .filter_map(|(target, edge_type)| {
                if edge_type.is_secondary_child_edge() {
                    Some(*target)
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
        let mut visited = BTreeSet::new();

        let mut carried_nodes;
        let mut forced_levels = self.get_forced_levels();

        let mut current_rank_nodes = self.root_nodes.to_vec();

        loop {
            // For each rank
            let mut next_rank_nodes: Vec<&str> = Vec::new();
            // Postpone ranking of forced nodes
            (current_rank_nodes, carried_nodes) =
                current_rank_nodes
                    .iter()
                    .partition(|&n| match forced_levels.get_mut(n) {
                        Some(forced_level) if *forced_level > 0 => {
                            // Reduce forced level
                            *forced_level -= 1;
                            false
                        }
                        // See if parents are not the same rank
                        _ => self
                            .parent_edges
                            .get(n)
                            .into_iter()
                            .flatten()
                            .filter(|(_, et)| et.is_primary_child_edge())
                            .all(|(p, _)| !current_rank_nodes.contains(p)),
                    });
            // Visit current nodes
            current_rank_nodes.iter().for_each(|n| {
                visited.insert(*n);
            });
            // Sort nodes lexicographically
            current_rank_nodes.sort();
            // Apply horizontal index movement
            self.reorder_horizontally(&mut current_rank_nodes);
            // Find children for next rank
            for parent_node in current_rank_nodes.iter() {
                // Get children of current parent
                let (mut children, mut child_carried_nodes) =
                    self.get_next_rank_children_of_parent(parent_node, &visited);
                // Append all children to next rank
                next_rank_nodes.append(&mut children);
                carried_nodes.append(&mut child_carried_nodes);
            }

            if current_rank_nodes.is_empty() && carried_nodes.is_empty() {
                break;
            } else {
                let current_rank = self.add_same_rank_nodes(current_rank_nodes, &mut visited);
                if !current_rank.is_empty() {
                    ranks.push(current_rank);
                }
                current_rank_nodes = next_rank_nodes
                    .into_iter()
                    .chain(carried_nodes)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
            }
        }
        ranks
    }

    ///
    ///
    ///
    ///
    fn reorder_horizontally(&self, current_rank_nodes: &mut Vec<&str>) {
        let current_rank_nodes_len = current_rank_nodes.len();
        let reordered_nodes = current_rank_nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                self.nodes
                    .get(*node)
                    .and_then(|n| n.get_horizontal_index(idx))
                    .map(|_| *node)
            })
            .collect::<Vec<_>>();
        for next_reorder in reordered_nodes {
            let cur_pos = current_rank_nodes
                .iter()
                .position(|n| *n == next_reorder)
                .unwrap(); // unwrap ok, since nodes exist.
            let new_pos = self
                .nodes
                .get(next_reorder)
                .unwrap() // unwrap ok, since nodes exist.
                .get_horizontal_index(cur_pos)
                .unwrap();
            let tmp = current_rank_nodes.remove(cur_pos);
            if new_pos > current_rank_nodes_len - 1 {
                current_rank_nodes.push(tmp);
            } else {
                current_rank_nodes.insert(new_pos, tmp);
            }
        }
    }

    ///
    /// Add same rank nodes
    ///
    ///
    fn add_same_rank_nodes<'b>(
        &'b self,
        current_rank_nodes: Vec<&'b str>,
        visited: &mut BTreeSet<&'b str>,
    ) -> Vec<Vec<&str>> {
        let mut current_rank: Vec<Vec<&str>> =
            current_rank_nodes.iter().map(|&n| vec![n]).collect();
        // Add inContext nodes
        for index in 0..current_rank_nodes.len() {
            let same_rank_parent = current_rank_nodes.get(index).unwrap().to_owned(); // unwrap ok, since nodes exist.
            let (left, right): (Vec<_>, Vec<_>) = self
                .get_same_rank_children(same_rank_parent)
                .into_iter()
                .filter(|n| !visited.contains(n))
                .enumerate()
                .partition(|(idx, same_rank_child)| {
                    if let Some(forced_index) = self
                        .get_nodes()
                        .get(same_rank_child.to_owned())
                        .unwrap() // unwrap ok, since nodes exist.
                        .get_horizontal_index(*idx)
                    {
                        0 == forced_index
                    } else {
                        // If a parent is already in the rank, put the same_rank_child to the left
                        self.get_same_ranks_parents(same_rank_child)
                            .iter()
                            .any(|p| current_rank_nodes[0..index].contains(p))
                            || idx % 2 != 0
                    }
                });
            let mut parent_index = current_rank
                .iter()
                .position(|x| x.contains(&same_rank_parent))
                .unwrap(); // unwrap ok, since nodes exist.
            if !left.is_empty() {
                let left_vec = left.iter().map(|(_, x)| *x).collect::<Vec<_>>();
                left_vec.iter().for_each(|n| {
                    visited.insert(n);
                });
                current_rank.insert(parent_index, left_vec);
                parent_index += 1;
            }
            if !right.is_empty() {
                let right_vec = right.iter().map(|(_, x)| *x).collect::<Vec<_>>();
                right_vec.iter().for_each(|n| {
                    visited.insert(n);
                });
                current_rank.insert(min(parent_index + 1, current_rank.len()), right_vec);
            }
        }
        current_rank
    }

    ///
    /// Get the children on the next rank of the given parent.
    /// This is used during ranking.
    ///
    fn get_next_rank_children_of_parent(
        &self,
        parent_node: &str,
        visited: &BTreeSet<&str>,
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
                self.parent_edges
                    .get(n)
                    .unwrap() // unwrap ok, since nodes exist.
                    .iter()
                    .filter(|(_, et)| et.is_primary_child_edge())
                    .all(|(p, _)| visited.contains(p))
            })
    }

    ///
    /// Collect a map of node IDs and their forced vertical rank increment.
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
mod test {
    use std::collections::BTreeMap;

    use crate::{
        dirgraph::{DirectedGraphEdgeType, DirectedGraphNodeType},
        dirgraphsvg::edges::{self, EdgeType},
    };

    use super::DirectedGraph;

    struct NT;
    struct ET;
    impl DirectedGraphNodeType<'_> for NT {
        fn get_forced_level(&self) -> Option<usize> {
            None
        }
        fn get_horizontal_index(&self, _current_index: usize) -> Option<usize> {
            None
        }
    }
    impl DirectedGraphEdgeType<'_> for ET {
        fn is_primary_child_edge(&self) -> bool {
            true
        }
        fn is_secondary_child_edge(&self) -> bool {
            false
        }
    }

    #[test]
    fn missing_edges() {
        let et = ET {};
        assert!(et.is_primary_child_edge());
        assert!(!et.is_secondary_child_edge());
    }

    #[test]
    fn debug_dirgraph() {
        let nodes = BTreeMap::from([("a".to_owned(), NT {}), ("b".to_owned(), NT {})]);
        let edges = BTreeMap::from([(
            "a".to_owned(),
            vec![(
                "b".to_owned(),
                EdgeType::OneWay(edges::SingleEdge::SupportedBy),
            )],
        )]);
        let dg = DirectedGraph::new(&nodes, &edges);
        let dbg = format!("{:?}", dg);
        assert_eq!(dbg, "a\nb\n");
    }

    #[test]
    fn no_roots() {
        let nodes = BTreeMap::from([("a".to_owned(), NT {}), ("b".to_owned(), NT {})]);
        let edges = BTreeMap::from([
            (
                "a".to_owned(),
                vec![(
                    "b".to_owned(),
                    EdgeType::OneWay(edges::SingleEdge::SupportedBy),
                )],
            ),
            (
                "b".to_owned(),
                vec![(
                    "a".to_owned(),
                    EdgeType::OneWay(edges::SingleEdge::SupportedBy),
                )],
            ),
        ]);
        let dg = DirectedGraph::new(&nodes, &edges);
        let dbg = format!("{:?}", dg);
        assert_eq!(dbg, "a\nb\n");
    }
}
