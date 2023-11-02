use std::collections::{BTreeMap, BTreeSet};

pub trait DirectedGraphNodeType<'a> {
    fn is_final_node(&'a self) -> bool;
}

pub trait DirectedGraphEdgeType<'a> {
    fn is_primary_child_edge(&'a self) -> bool;
    fn is_secondary_child_edge(&'a self) -> bool;
}

pub(super) struct DirectedGraph<'a, NodeType, EdgeType>
where
    NodeType: DirectedGraphNodeType<'a> + Sized,
    EdgeType: DirectedGraphEdgeType<'a> + Copy,
{
    nodes: &'a BTreeMap<String, NodeType>,
    edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
    // Map of node to forced rank
    forced_levels: BTreeMap<&'a str, usize>,
    root_nodes: Vec<&'a str>,
    parent_edges: BTreeMap<&'a str, Vec<(&'a str, EdgeType)>>,
}

impl<'a, NodeType, EdgeType> DirectedGraph<'a, NodeType, EdgeType>
where
    NodeType: DirectedGraphNodeType<'a> + Sized,
    EdgeType: DirectedGraphEdgeType<'a> + Copy,
{
    ///
    ///
    ///
    /// This graph object should know as little as possible about its nodes.
    /// Thus, forced_levels should be provided externally.
    ///
    pub fn new(
        nodes: &'a BTreeMap<String, NodeType>,
        edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
        forced_levels: &BTreeMap<&str, Vec<&'a str>>,
    ) -> Self {
        // Initialize node_info map
        let mut node_info = DirectedGraph {
            nodes,
            edges,
            forced_levels: BTreeMap::new(),
            root_nodes: vec![],
            parent_edges: BTreeMap::new(),
        };
        node_info.calculate_parent_edge_map();
        node_info.find_root_nodes();
        node_info.calculate_forced_levels(forced_levels);

        // TODO REIMPLEMENT
        // loop {
        //     let changed = node_info.find_ranks(nodes, edges, &node_info.root_nodes);
        //     node_info.constrain_by_forced_levels(nodes, edges, forced_levels);
        //     if !changed {
        //         break;
        //     }
        // }
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
                    .or_insert_with(Vec::new)
                    .push((child, *target_type));
            }
        }
    }

    ///
    ///
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
    /// FIXME: What if we have (a, b) and (b, c) and a and c are on different levels.
    ///
    ///
    fn calculate_forced_levels(&mut self, forced_levels: &BTreeMap<&str, Vec<&'a str>>) {
        for (_, ref nodes) in forced_levels {
            if let Some(max_depth) = nodes.iter().map(|&n| self.get_distance(n)).max() {
                nodes.iter().for_each(|n| {
                    self.forced_levels.insert(n.to_owned(), max_depth);
                });
            }
        }
    }

    ///
    ///
    ///
    pub fn get_root_nodes(&'a self) -> Vec<&'a str> {
        self.root_nodes.to_vec()
    }

    ///
    ///
    ///
    pub fn get_parent_edges(&'a self) -> BTreeMap<&'a str, Vec<(&'a str, EdgeType)>> {
        self.parent_edges.to_owned()
    }

    ///
    ///
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
    ///
    ///
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
        let unvisited: BTreeSet<&str> = node_keys.difference(&visited).map(|&v| v).collect();

        unvisited.into_iter().collect()
    }

    ///
    ///
    ///
    ///
    fn get_same_rank_children(&self, node: &str) -> Vec<&str> {
        self.edges
            .get(node)
            .iter()
            .flat_map(|&x| x)
            .into_iter()
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
    ///
    ///
    ///
    fn get_real_children(&self, node: &str) -> Vec<&str> {
        self.edges
            .get(node)
            .iter()
            .flat_map(|&x| x)
            .into_iter()
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
    ///
    /// Returns true if any rank has changed.
    ///
    // fn find_ranks(
    //     &mut self,
    //     nodes: &BTreeMap<String, Node>,
    //     edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    //     root_nodes: &[&str],
    // ) -> bool {
    //     let mut changed = false;
    // for &root_node in root_nodes.iter() {
    //     self.set_rank(root_node, 0);
    //     let mut stack = vec![root_node];
    //     self.visit_node(root_node);
    //     while let Some(parent_id) = stack.pop() {
    //         let mut current_node = parent_id;
    //         let mut current_rank = self.get_rank(current_node).unwrap();
    //         while let Some(child_node) =
    //             find_next_child_node(nodes, edges, &self.0, current_node, true)
    //         {
    //             self.set_rank(current_node, current_rank);
    //             stack.push(current_node);
    //             self.visit_node(child_node);
    //             current_rank = determine_child_rank(&mut self.0, child_node, current_rank);
    //             changed |= self.set_rank(child_node, current_rank);
    //             current_node = child_node;
    //         }
    //     }
    // }
    // self.unvisit_nodes();
    //     changed
    // }

    ///
    ///
    ///
    /// vertical, horizontal, cell
    ///
    pub(super) fn rank_nodes(&'a mut self, cycles_allowed: bool) -> Vec<Vec<Vec<&str>>> {
        let mut ranks = Vec::new();
        let mut vertical_index = 0;

        let mut next_rank_nodes = self.root_nodes.to_vec();

        loop {
            // FIXME root nodes that have forced levels
            next_rank_nodes = self
                .get_nodes_for_next_rank(&next_rank_nodes)
                .iter()
                .filter(|&n| {
                    if let Some(&forced_level) = self.forced_levels.get(n) {
                        forced_level > vertical_index
                    } else {
                        true
                    }
                })
                .map(|&n| n)
                .collect();
            if next_rank_nodes.is_empty() {
                break;
            } else {
                ranks.push(next_rank_nodes.iter().map(|&n| vec![n]).collect());
                vertical_index += 1;
            }
        }
        ranks
    }

    ///
    ///
    ///
    ///
    fn get_nodes_for_next_rank(&self, current_rank_nodes: &[&str]) -> Vec<&str> {
        current_rank_nodes
            .iter()
            .map(|&n| self.get_real_children(n))
            .flatten()
            .collect()
    }

    ///
    /// Get distance of `node` from any root_nodes
    ///
    ///
    fn get_distance(&self, node: &str) -> usize {
        let mut cur_rank = self.root_nodes.to_vec();
        let mut distance = 0;
        loop {
            if cur_rank.contains(&node) {
                break;
            }
            cur_rank = self.get_nodes_for_next_rank(&cur_rank);
            distance += 1;
        }
        distance
    }

    // fn int_rank_nodes(
    //     &mut self,
    //     cycles_allowed: bool,
    // ) -> BTreeMap<usize, BTreeMap<usize, Vec<&'a str>>> {
    //     let mut ranks = BTreeMap::new();
    //     for (horizontal_rank, &root_node) in root_nodes.iter().enumerate() {
    //         let mut stack = vec![root_node];
    //         self.visit_node(root_node);
    //         let vertical_rank = ranks
    //             .entry(self.get_rank(root_node).unwrap())
    //             .or_insert_with(BTreeMap::new);
    //         vertical_rank.insert(horizontal_rank, NodePlace::Node(root_node.to_owned()));
    //         while let Some(parent_id) = stack.pop() {
    //             let mut current_node = parent_id;
    //             while let Some(child_node) =
    //                 find_next_child_node(nodes, edges, &self.0, current_node, cycles_allowed)
    //             {
    //                 stack.push(current_node);
    //                 let vertical_rank =
    //                     ranks.entry(self.get_rank(child_node).unwrap()).or_default();
    //                 vertical_rank
    //                     .insert(vertical_rank.len(), NodePlace::Node(child_node.to_owned()));

    //                 self.visit_node(child_node);
    //                 current_node = child_node;
    //             }
    //         }
    //     }
    //     self.unvisit_nodes();
    //     ranks
    // }
}

///
///
///
///
// fn determine_child_rank(
//     node_info: &mut BTreeMap<String, NodeInfo>,
//     child_node: &str,
//     current_rank: usize,
// ) -> usize {
//     // If one parent is on the same rank, put the child one rank further down.
//     let max_parent_rank = node_info
//         .get(child_node)
//         .unwrap() // All nodes exist in node_info map
//         .parents
//         .iter()
//         .filter_map(
//             |p| node_info.get(p.to_owned()).map(|p| p.rank).unwrap(), // All nodes exist in node_info map
//         )
//         .max()
//         .map(|x| x + 1);

//     let r = [
//         Some(current_rank + 1),
//         max_parent_rank,
//         node_info.get(child_node).map(|ni| ni.rank).unwrap(), // All nodes exist in node_info map
//     ]
//     .iter()
//     .filter_map(|&x| x)
//     .max()
//     .unwrap(); // unwrap is ok, since r is never empty.

//     r
// }

///
///
/// Finds the next child node and returns it.
/// The rank could be constraint by the forced level.
///
// fn find_next_child_node<'a>(
//     nodes: &'a BTreeMap<String, Node>,
//     edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
//     node_info: &BTreeMap<String, NodeInfo>,
//     current: &str,
//     allow_unranked_parent: bool,
// ) -> Option<&'a str> {
//     if let Some(p) = edges.get(current) {
//         let mut x = p.to_vec();
//         x.sort_by_key(|a| {
//             (
//                 // Rank nodes earlier if they have multiple parents
//                 std::cmp::Reverse(
//                     edges
//                         .iter()
//                         // Get already ranked parents
//                         .filter_map(|(parent, children)| {
//                             node_info.get(parent).unwrap().rank.map(|_| children)
//                         })
//                         .flatten()
//                         // See if they have a SupportedBy relation to the current child
//                         .filter(|(child, et)| {
//                             child == &a.0 && et == &EdgeType::OneWay(SingleEdge::SupportedBy)
//                         })
//                         .count(),
//                 ),
//                 // Sort alphabetically next
//                 a.0.to_owned(),
//             )
//         });

//         if let Some(opt_child) = x
//             .iter()
//             .filter(|(id, _)| match allow_unranked_parent {
//                 false => count_unvisited_parents(node_info, id) == 0,
//                 true => true, // Architecture view may contain circles.
//                               // We have to skip this filter, otherwise we end in an endless loop.
//                               // We need to start ranking nodes even not all parents are drawn to break the circle.
//             })
//             .filter_map(|(id, et)| match et {
//                 EdgeType::OneWay(SingleEdge::SupportedBy)
//                 | EdgeType::OneWay(SingleEdge::Composite)
//                 | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
//                 | EdgeType::TwoWay((SingleEdge::SupportedBy, _))
//                 | EdgeType::TwoWay((_, SingleEdge::Composite))
//                 | EdgeType::TwoWay((SingleEdge::Composite, _)) => Some(id),
//                 _ => None,
//             })
//             .find(|id| !node_info.get(id.to_owned()).unwrap().visited)
//         {
//             return nodes.keys().find(|&x| x == opt_child).map(|x| x.as_str());
//             // return Some(opt_child);
//         }
//     }
//     None
// }

///
///
///
///
// fn count_unvisited_parents(node_info: &BTreeMap<String, NodeInfo>, current: &str) -> usize {
//     let mut unvisited_parents = 0usize;
//     // None can actually only happen for root nodes
//     if let Some(ni) = node_info.get(current) {
//         for parent in &ni.parents {
//             if !node_info.get(parent.to_owned()).unwrap().visited {
//                 unvisited_parents += 1;
//             }
//         }
//     }
//     unvisited_parents
// }

///
/// Until now, only supportedBy nodes are ranked.
/// Insert inContextOf nodes now.
/// At best very closely to the referencing node.
///
///
// fn add_in_context_nodes(
//     edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
//     ranks: &mut BTreeMap<usize, BTreeMap<usize, Vec<&str>>>,
// ) {
//     let mut visited_nodes: BTreeSet<String> = BTreeSet::new();
// for v_ranks in ranks.values_mut() {
//     let mut new_rank = Vec::new();
//     for n in v_ranks.values() {
//         match n {
//             NodePlace::Node(n) => {
//                 if let Some(target) = edges.get(n) {
//                     let mut i = 0;
//                     let (left, right): (Vec<String>, Vec<String>) = target
//                         .iter()
//                         .filter_map(|(tn, et)| match et {
//                             EdgeType::OneWay(SingleEdge::InContextOf) => Some(tn.to_owned()),
//                             _ => None,
//                         })
//                         .filter(|tn| !visited_nodes.contains(tn))
//                         .collect::<Vec<String>>()
//                         .into_iter()
//                         .partition(|x| {
//                             i += 1;
//                             if previous_node_with_connection(x, n, &new_rank, edges) {
//                                 // Make left/right distribution more even
//                                 if i % 2 == 1 {
//                                     i += 1;
//                                 }
//                                 true
//                             } else if edges
//                                 .values()
//                                 .flatten()
//                                 .filter(|(tn, et)| {
//                                     tn == x && *et == EdgeType::OneWay(SingleEdge::InContextOf)
//                                 })
//                                 .count()
//                                 > 1
//                             {
//                                 false
//                             } else {
//                                 i % 2 == 0
//                             }
//                         });
//                     // Visit nodes
//                     left.iter().for_each(|n| {
//                         visited_nodes.insert(n.to_owned());
//                     });
//                     right.iter().for_each(|n| {
//                         visited_nodes.insert(n.to_owned());
//                     });
//                     match &left.len() {
//                         0 => (),
//                         1 => new_rank.push(NodePlace::Node(left.get(0).unwrap().to_owned())),
//                         _ => new_rank.push(NodePlace::MultipleNodes(
//                             left.iter().map(|x| x.to_owned()).collect(),
//                         )),
//                     }
//                     new_rank.push(NodePlace::Node(n.to_owned()));
//                     match &right.len() {
//                         0 => (),
//                         1 => new_rank.push(NodePlace::Node(right.get(0).unwrap().to_owned())),
//                         _ => new_rank.push(NodePlace::MultipleNodes(
//                             right.iter().map(|x| x.to_owned()).collect(),
//                         )),
//                     }
//                 } else {
//                     new_rank.push(NodePlace::Node(n.to_owned()));
//                     visited_nodes.insert(n.to_owned());
//                 }
//             }
//             NodePlace::MultipleNodes(_) => (),
//         }
//     }
//     v_ranks.clear();
//     for (h, np) in new_rank.into_iter().enumerate() {
//         v_ranks.insert(h, np);
//     }
// }
// }

///
///
///
///
///
// fn previous_node_with_connection(
//     node: &str,
//     cur_node: &str,
//     cur_rank: &[Vec<&str>],
//     edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
// ) -> bool {
//     cur_rank.iter().any(|np| {
//         for &id in np {
//             if id != cur_node {
//                 if let Some(targets) = edges.get(id) {
//                     return targets.iter().any(|(tid, _)| tid == node);
//                 }
//             }
//         }
//         false
//     })
// }

#[cfg(test)]
mod test {}
