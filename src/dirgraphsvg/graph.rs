use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::dirgraphsvg::{
    edges::{EdgeType, SingleEdge},
    nodes::Node,
};

use super::{util::point2d::Point2D, Margin};

#[derive(Debug)]
pub enum NodePlace {
    Node(String),
    MultipleNodes(Vec<String>),
}

impl NodePlace {
    ///
    /// Get (maximum) width of NodePlace
    ///
    /// Panics if a node in NodePlace does not exist or if NodePlace with multiple nodes is empty.
    ///
    ///
    pub(crate) fn get_max_width(&self, nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>) -> i32 {
        match self {
            NodePlace::Node(n) => nodes.get(n).unwrap().borrow().get_width(),
            NodePlace::MultipleNodes(np) => np
                .iter()
                .map(|n| nodes.get(n).unwrap().borrow().get_width())
                .max()
                .unwrap(),
        }
    }

    ///
    /// Position point to the center of the node
    ///
    ///
    pub(crate) fn set_position(
        &self,
        nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>,
        margin: &Margin,
        pos: Point2D,
    ) {
        // Unwraps are ok, since NodePlace are only created from existing nodes
        match self {
            NodePlace::Node(id) => {
                let mut n = nodes.get(id).unwrap().borrow_mut();
                n.set_position(&pos);
            }
            NodePlace::MultipleNodes(ids) => {
                let max_h = ids
                    .iter()
                    .map(|id| nodes.get(id).unwrap().borrow().get_height())
                    .sum::<i32>()
                    + (margin.top + margin.bottom) * (ids.len() - 1) as i32;
                let mut y_n = pos.y - max_h / 2;
                for id in ids {
                    let mut n = nodes.get(id).unwrap().borrow_mut();
                    let h = n.get_height();
                    n.set_position(&Point2D {
                        x: pos.x,
                        y: y_n + h / 2,
                    });
                    y_n += h + margin.top + margin.bottom;
                }
            }
        }
    }

    ///
    /// Get x value of NodePlace
    ///
    /// MultipleNodes have the same x, thus, just the value of the first node is used.
    /// MultipleNodes are never empty.
    ///
    ///
    pub(crate) fn get_x(&self, nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>) -> i32 {
        // Unwraps are ok, since NodePlace are only created from existing nodes
        match self {
            NodePlace::Node(n) => {
                let n = nodes.get(n).unwrap().borrow();
                n.get_position().x
            }
            NodePlace::MultipleNodes(np) => {
                let n = nodes.get(np.first().unwrap()).unwrap().borrow();
                n.get_position().x
            }
        }
    }
}

#[derive(Debug)]
struct NodeInfo<'a> {
    rank: Option<usize>,
    max_child_rank: Option<usize>,
    parents: BTreeSet<&'a str>,
    visited: bool,
}

#[derive(Debug)]
struct NodeInfoMap<'a>(BTreeMap<String, NodeInfo<'a>>);

impl<'a> NodeInfoMap<'a> {
    ///
    ///
    ///
    ///
    ///
    fn new(
        nodes: &'a BTreeMap<String, Rc<RefCell<dyn Node>>>,
        edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
        root_nodes: &[&'a str],
        forced_levels: &BTreeMap<&'a str, Vec<&'a str>>,
    ) -> Self {
        // Initialize node_info map
        let mut node_info = NodeInfoMap(
            nodes
                .keys()
                .map(|k| {
                    (
                        k.to_owned(),
                        NodeInfo {
                            rank: None,
                            max_child_rank: None,
                            visited: false,
                            parents: BTreeSet::new(),
                        },
                    )
                })
                .collect(),
        );
        // Look up parents for each node
        for (child, target_edges) in edges {
            for (target_edge, _) in target_edges {
                node_info.0.entry(target_edge.to_owned()).and_modify(|e| {
                    e.parents.insert(child);
                });
            }
        }

        loop {
            let changed = node_info.find_ranks(nodes, edges, root_nodes);
            node_info.constrain_by_forced_levels(nodes, edges, forced_levels);
            node_info.set_max_child_rank();
            if !changed {
                break;
            }
        }
        node_info
    }

    ///
    ///
    ///
    ///
    ///
    ///
    fn constrain_by_forced_levels(
        &mut self,
        nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>,
        edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
        forced_levels: &BTreeMap<&'a str, Vec<&'a str>>,
    ) {
        for forced_nodes in forced_levels.values() {
            if forced_nodes.iter().all(|&n| self.0.contains_key(n)) {
                if let Some(max_depth) = forced_nodes.iter().filter_map(|&n| self.get_rank(n)).max()
                {
                    for &node in forced_nodes {
                        let diff_rank = self.get_rank(node).unwrap().abs_diff(max_depth);
                        if diff_rank > 0 {
                            self.set_rank(node, max_depth);
                            let mut stack = vec![node];
                            self.visit_node(node);
                            while let Some(parent_id) = stack.pop() {
                                let mut current_node = parent_id;
                                while let Some(child_node) =
                                    find_next_child_node(nodes, edges, &self.0, current_node, true)
                                {
                                    stack.push(current_node);
                                    self.visit_node(child_node);
                                    let current_rank =
                                        self.get_rank(child_node).unwrap() + diff_rank;
                                    self.set_rank(child_node, current_rank);
                                    current_node = child_node;
                                }
                            }
                        }
                    }
                }
            }
        }
        self.unvisit_nodes();
    }

    ///
    ///
    ///
    ///
    fn set_max_child_rank(&mut self) {
        let mut max_map = BTreeMap::new();
        // Assign own rank as initial maximum child rank
        self.0
            .values_mut()
            .for_each(|ni| ni.max_child_rank = ni.rank);
        loop {
            let mut changed = false;
            for ni in self.0.values() {
                for &parent in &ni.parents {
                    max_map
                        .entry(parent)
                        .and_modify(|v| {
                            let v_prev = *v;
                            *v = std::cmp::max(*v, ni.max_child_rank);
                            if v_prev != *v {
                                changed = true;
                            }
                        })
                        .or_insert(ni.rank);
                }
            }
            max_map.iter_mut().for_each(|(&k, v)| {
                self.0.get_mut(k).unwrap().max_child_rank = *v;
            });
            if !changed {
                break;
            }
        }
    }

    ///
    ///
    /// Returns true if rank has changed
    ///
    fn set_rank(&mut self, current_node: &str, current_rank: usize) -> bool {
        let mut changed = false;
        self.0.entry(current_node.to_owned()).and_modify(|v| {
            if v.rank == Some(current_rank) {
                changed = false;
            } else {
                v.rank = Some(current_rank);
                changed = true;
            }
        });
        changed
    }

    ///
    ///
    ///
    ///
    fn get_rank(&self, current_node: &str) -> Option<usize> {
        let v = self.0.get(current_node).unwrap();
        v.rank
    }

    ///
    ///
    ///
    ///
    fn visit_node(&mut self, node: &str) {
        self.0
            .entry(node.to_owned())
            .and_modify(|n| n.visited = true);
    }

    ///
    ///
    ///
    ///
    fn unvisit_nodes(&mut self) {
        self.0.iter_mut().for_each(|(_, mut ni)| {
            ni.visited = false;
        });
    }

    ///
    ///
    /// Returns true if any rank has changed.
    ///
    fn find_ranks(
        &mut self,
        nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>,
        edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
        root_nodes: &[&str],
    ) -> bool {
        let mut changed = false;
        for &root_node in root_nodes.iter() {
            self.set_rank(root_node, 0);
            let mut stack = vec![root_node];
            self.visit_node(root_node);
            while let Some(parent_id) = stack.pop() {
                let mut current_node = parent_id;
                let mut current_rank = self.get_rank(current_node).unwrap();
                while let Some(child_node) =
                    find_next_child_node(nodes, edges, &self.0, current_node, true)
                {
                    self.set_rank(current_node, current_rank);
                    stack.push(current_node);
                    self.visit_node(child_node);
                    current_rank = determine_child_rank(&mut self.0, child_node, current_rank);
                    changed |= self.set_rank(child_node, current_rank);
                    current_node = child_node;
                }
            }
        }
        self.unvisit_nodes();
        changed
    }

    ///
    ///
    ///
    ///
    fn rank_nodes(
        &mut self,
        nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>,
        edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
        root_nodes: &[&str],
        cycles_allowed: bool,
    ) -> BTreeMap<usize, BTreeMap<usize, NodePlace>> {
        let mut ranks = BTreeMap::new();
        for (horiz_rank, &root_node) in root_nodes.iter().enumerate() {
            let mut stack = vec![root_node];
            self.visit_node(root_node);
            let vertical_rank = ranks
                .entry(self.get_rank(root_node).unwrap())
                .or_insert_with(BTreeMap::new);
            vertical_rank.insert(horiz_rank, NodePlace::Node(root_node.to_owned()));
            while let Some(parent_id) = stack.pop() {
                let mut current_node = parent_id;
                while let Some(child_node) =
                    find_next_child_node(nodes, edges, &self.0, current_node, cycles_allowed)
                {
                    stack.push(current_node);
                    let vertical_rank =
                        ranks.entry(self.get_rank(child_node).unwrap()).or_default();
                    vertical_rank
                        .insert(vertical_rank.len(), NodePlace::Node(child_node.to_owned()));

                    self.visit_node(child_node);
                    current_node = child_node;
                }
            }
        }
        self.unvisit_nodes();
        ranks
    }
}

///
///
///
///
///
pub(crate) fn rank_nodes<'a>(
    nodes: &'a mut BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &'a mut BTreeMap<String, Vec<(String, EdgeType)>>,
    forced_levels: &BTreeMap<&'a str, Vec<&'a str>>,
    cycles_allowed: bool,
) -> BTreeMap<usize, BTreeMap<usize, NodePlace>> {
    // Copy IDs
    let mut n_ids: BTreeSet<String> = nodes.keys().cloned().collect();
    // Find root nodes
    for t_edges in edges.values() {
        for (target, _) in t_edges {
            n_ids.remove(target);
        }
    }
    let root_nodes: Vec<&str> = if n_ids.is_empty() {
        // No root nodes are found.
        // This can actually only happen in architecture view.
        // Take the first node and start from there.
        vec![nodes.iter().next().unwrap().0]
    } else {
        n_ids.iter().map(|rn| rn.as_ref()).collect()
    };

    let mut node_info = NodeInfoMap::new(nodes, edges, &root_nodes, forced_levels);
    let mut ranks = node_info.rank_nodes(nodes, edges, &root_nodes, cycles_allowed);

    add_in_context_nodes(edges, &mut ranks);
    ranks
}

///
///
///
///
fn determine_child_rank(
    node_info: &mut BTreeMap<String, NodeInfo>,
    child_node: &str,
    current_rank: usize,
) -> usize {
    // If one parent is on the same rank, put the child one rank further down.
    let max_parent_rank = node_info
        .get(child_node)
        .unwrap() // All nodes exist in node_info map
        .parents
        .iter()
        .filter_map(
            |p| node_info.get(p.to_owned()).map(|p| p.rank).unwrap(), // All nodes exist in node_info map
        )
        .max()
        .map(|x| x + 1);

    let r = [
        Some(current_rank + 1),
        max_parent_rank,
        node_info.get(child_node).map(|ni| ni.rank).unwrap(), // All nodes exist in node_info map
    ]
    .iter()
    .filter_map(|&x| x)
    .max()
    .unwrap(); // unwrap is ok, since r is never empty.

    r
}

///
///
/// Finds the next child node and returns it.
/// The rank could be constraint by the forced level.
///
fn find_next_child_node<'a, 'b>(
    nodes: &'a BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
    node_info: &'b BTreeMap<String, NodeInfo>,
    current: &str,
    allow_unranked_parent: bool,
) -> Option<&'a str> {
    if let Some(p) = edges.get(current) {
        let mut x = p.to_vec();
        x.sort_by_key(|a| {
            (
                // Reverse since higher values should be ranked earlier
                std::cmp::Reverse(node_info.get(&a.0).unwrap().max_child_rank),
                // Rank nodes earlier if they have multiple parents
                std::cmp::Reverse(
                    edges
                        .iter()
                        // Get already ranked parents
                        .filter_map(|(parent, children)| {
                            node_info.get(parent).unwrap().rank.map(|_| children)
                        })
                        .flatten()
                        // See if they have a SupportedBy relation to the current child
                        .filter(|(child, et)| {
                            child == &a.0 && et == &EdgeType::OneWay(SingleEdge::SupportedBy)
                        })
                        .count(),
                ),
                // Sort alphabetically next
                a.0.to_owned(),
            )
        });

        if let Some(opt_child) = x
            .iter()
            .filter(|(id, _)| match allow_unranked_parent {
                false => count_unvisited_parents(node_info, id) == 0,
                true => true, // Architecture view may contain circles.
                              // We have to skip this filter, otherwise we end in an endless loop.
                              // We need to start ranking nodes even not all parents are drawn to break the circle.
            })
            .filter_map(|(id, et)| match et {
                EdgeType::OneWay(SingleEdge::SupportedBy)
                | EdgeType::OneWay(SingleEdge::Composite)
                | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                | EdgeType::TwoWay((SingleEdge::SupportedBy, _))
                | EdgeType::TwoWay((_, SingleEdge::Composite))
                | EdgeType::TwoWay((SingleEdge::Composite, _)) => Some(id),
                _ => None,
            })
            .find(|id| !node_info.get(id.to_owned()).unwrap().visited)
        {
            return nodes.keys().find(|&x| x == opt_child).map(|x| x.as_str());
            // return Some(opt_child);
        }
    }
    None
}

///
///
///
///
fn count_unvisited_parents(node_info: &BTreeMap<String, NodeInfo>, current: &str) -> usize {
    let mut unvisited_parents = 0usize;
    // None can actually only happen for root nodes
    if let Some(ni) = node_info.get(current) {
        for parent in &ni.parents {
            if !node_info.get(parent.to_owned()).unwrap().visited {
                unvisited_parents += 1;
            }
        }
    }
    unvisited_parents
}

///
/// Until now, only supportedBy nodes are ranked.
/// Insert inContextOf nodes now.
/// At best very closely to the referencing node.
///
///
fn add_in_context_nodes(
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    ranks: &mut BTreeMap<usize, BTreeMap<usize, NodePlace>>,
) {
    let mut visited_nodes: BTreeSet<String> = BTreeSet::new();
    for v_ranks in ranks.values_mut() {
        let mut new_rank = Vec::new();
        for n in v_ranks.values() {
            match n {
                NodePlace::Node(n) => {
                    if let Some(target) = edges.get(n) {
                        let mut i = 0;
                        let (left, right): (Vec<String>, Vec<String>) = target
                            .iter()
                            .filter_map(|(tn, et)| match et {
                                EdgeType::OneWay(SingleEdge::InContextOf) => Some(tn.to_owned()),
                                _ => None,
                            })
                            .filter(|tn| !visited_nodes.contains(tn))
                            .collect::<Vec<String>>()
                            .into_iter()
                            .partition(|x| {
                                i += 1;
                                if previous_node_with_connection(x, n, &new_rank, edges) {
                                    // Make left/right distribution more even
                                    if i % 2 == 1 {
                                        i += 1;
                                    }
                                    true
                                } else {
                                    i % 2 == 0
                                }
                            });
                        // Visit nodes
                        left.iter().for_each(|n| {
                            visited_nodes.insert(n.to_owned());
                        });
                        right.iter().for_each(|n| {
                            visited_nodes.insert(n.to_owned());
                        });
                        match &left.len() {
                            0 => (),
                            1 => new_rank.push(NodePlace::Node(left.get(0).unwrap().to_owned())),
                            _ => new_rank.push(NodePlace::MultipleNodes(
                                left.iter().map(|x| x.to_owned()).collect(),
                            )),
                        }
                        new_rank.push(NodePlace::Node(n.to_owned()));
                        match &right.len() {
                            0 => (),
                            1 => new_rank.push(NodePlace::Node(right.get(0).unwrap().to_owned())),
                            _ => new_rank.push(NodePlace::MultipleNodes(
                                right.iter().map(|x| x.to_owned()).collect(),
                            )),
                        }
                    } else {
                        new_rank.push(NodePlace::Node(n.to_owned()));
                        visited_nodes.insert(n.to_owned());
                    }
                }
                NodePlace::MultipleNodes(_) => (),
            }
        }
        v_ranks.clear();
        for (h, np) in new_rank.into_iter().enumerate() {
            v_ranks.insert(h, np);
        }
    }
}

///
///
///
///
///
fn previous_node_with_connection(
    node: &str,
    cur_node: &str,
    cur_rank: &[NodePlace],
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
) -> bool {
    cur_rank.iter().any(|np| match np {
        NodePlace::Node(id) => {
            if id != cur_node {
                if let Some(targets) = edges.get(id) {
                    targets.iter().any(|(tid, _)| tid == node)
                } else {
                    false
                }
            } else {
                false
            }
        }
        NodePlace::MultipleNodes(mn) => {
            for id in mn {
                if id != cur_node {
                    if let Some(targets) = edges.get(id) {
                        return targets.iter().any(|(tid, _)| tid == node);
                    }
                }
            }
            false
        }
    })
}

///
/// `edges` is a map of parent node id to a vector of edges (child, type)
/// The returned parent map is a map of child node it to a vector of edges (parent, type)
///
///
pub fn calculate_parent_edge_map(
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
) -> BTreeMap<String, Vec<(String, EdgeType)>> {
    let mut parent_map = BTreeMap::new();
    for (child, target_edges) in edges {
        for (target_edge, target_type) in target_edges {
            parent_map
                .entry(target_edge.to_owned())
                .or_insert_with(Vec::new)
                .push((child.to_owned(), *target_type));
        }
    }
    parent_map
}
