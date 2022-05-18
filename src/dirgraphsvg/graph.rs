use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashSet},
    rc::Rc,
};

use crate::dirgraphsvg::{
    edges::{EdgeType, SingleEdge},
    nodes::Node,
};

// use super::{util::point2d::Point2D, Margin};

#[derive(Debug)]
pub enum NodePlace {
    Node(String),
    MultipleNodes(Vec<String>),
}

impl NodePlace {
    pub(crate) fn get_max_width(&self, nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>) -> u32 {
        match self {
            NodePlace::Node(n) => nodes.get(n).unwrap().borrow().get_width(),
            NodePlace::MultipleNodes(np) => np
                .iter()
                .map(|n| nodes.get(n).unwrap().borrow().get_width())
                .max()
                .unwrap(),
        }
    }

    // pub(crate) fn set_position(
    //     &mut self,
    //     nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>,
    //     margin: &Margin,
    //     pos: Point2D,
    // ) {
    //     match self {
    //         NodePlace::Node(id) => {
    //             let mut n = nodes.get(id).unwrap().borrow_mut();
    //             n.set_position(&Point2D { x: pos.x, y: pos.y });
    //         }
    //         NodePlace::MultipleNodes(ids) => {
    //             let x_max = ids
    //                 .iter()
    //                 .map(|id| nodes.get(id).unwrap().borrow().get_width())
    //                 .max()
    //                 .unwrap();
    //             let mut y_n = pos.y;
    //             for id in ids {
    //                 let mut n = nodes.get(id).unwrap().borrow_mut();
    //                 let n_height = n.get_height();
    //                 n.set_position(&Point2D {
    //                     x: pos.x,
    //                     y: y_n + n_height / 2,
    //                 });
    //                 y_n += n_height + margin.top + margin.bottom;
    //             }
    //         }
    //     }
    // }

    pub(crate) fn get_x(&self, nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>) -> u32 {
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

struct NodeInfo<'a> {
    rank: Option<usize>,
    max_child_rank: Option<usize>,
    parents: BTreeSet<&'a str>,
    visited: bool,
}

struct NodeInfoMap<'a>(BTreeMap<String, NodeInfo<'a>>);

impl<'a> std::ops::DerefMut for NodeInfoMap<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> std::ops::Deref for NodeInfoMap<'a> {
    type Target = BTreeMap<String, NodeInfo<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> NodeInfoMap<'a> {
    ///
    ///
    ///
    ///
    ///
    fn new(
        nodes: &'a BTreeMap<String, Rc<RefCell<dyn Node>>>,
        edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
        root_nodes: &Vec<&'a str>,
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
                node_info.entry(target_edge.to_owned()).and_modify(|e| {
                    e.parents.insert(child);
                });
            }
        }

        for &root_node in root_nodes.iter() {
            let mut stack = Vec::new();
            node_info.set_rank(root_node, 0);
            stack.push(root_node);
            node_info.visit_node(root_node);
            while let Some(parent_id) = stack.pop() {
                let mut current_node = parent_id;
                let mut current_rank = node_info.get_rank(current_node).unwrap();
                while let Some(child_node) =
                    find_next_child_node(nodes, edges, &node_info, &current_node, false)
                {
                    node_info.set_rank(current_node, current_rank);
                    stack.push(current_node);
                    node_info.visit_node(child_node);
                    current_rank = dbg!(determine_child_rank(
                        nodes,
                        &node_info,
                        &child_node,
                        current_rank
                    ));
                    current_node = dbg!(&child_node);
                }
                node_info.set_max_child_rank(current_node, current_rank);
                // Record maximum depth
                for s in &stack {
                    node_info.set_max_child_rank(s, current_rank);
                }
                node_info.set_max_child_rank(parent_id, current_rank);
            }
        }
        node_info.unvisit_nodes();
        node_info
    }

    ///
    ///
    ///
    ///
    fn set_max_child_rank(&mut self, current_node: &str, current_rank: usize) {
        self.0.entry(current_node.to_owned()).and_modify(|v| {
            v.max_child_rank = std::cmp::max(v.max_child_rank, Some(current_rank + 1))
        });
    }

    ///
    /// 
    /// 
    /// 
    fn set_rank(&mut self, current_node: &str, current_rank: usize) {
        self.0.entry(current_node.to_owned()).and_modify(|v| {
            v.rank = Some(current_rank);
        });
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
}

///
///
///
///
///
pub(crate) fn rank_nodes<'a>(
    nodes: &'a mut BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &'a mut BTreeMap<String, Vec<(String, EdgeType)>>,
    allow_cycle: bool,
) -> BTreeMap<usize, BTreeMap<usize, NodePlace>> {
    let mut ranks = BTreeMap::new();
    let mut rank_map = BTreeMap::<String, usize>::new();

    // Copy IDs
    let mut n_ids: BTreeSet<String> = nodes
        .iter()
        // Filter nodes with forced level larger than 0
        .filter(|(_, node)| !matches!(node.borrow().get_forced_level(), Some(x) if x != 0))
        .map(|(id, _)| id.to_owned())
        .collect();
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

    let mut node_info = NodeInfoMap::new(nodes, edges, &root_nodes);

    // let max_depths = get_max_depths(nodes, edges, &root_nodes, edge_map, &rank_map);

    // Perform depth first search for SupportedBy child nodes.
    for (horiz_rank, n) in root_nodes.into_iter().enumerate() {
        node_info.visit_node(n);
        rank_map.insert(n.to_owned(), 0);
        {
            let vertical_rank = ranks.entry(0).or_insert(BTreeMap::new());
            vertical_rank.insert(horiz_rank, NodePlace::Node(n.to_owned()));
        }
        let mut stack = Vec::new();
        let mut current_node = n;
        let mut current_rank;
        stack.push((current_node, 0));
        while let Some((p_id, p_r)) = stack.pop() {
            current_node = p_id;
            current_rank = p_r;
            while let Some(child_node) =
                find_next_child_node(nodes, edges, &node_info, &current_node, allow_cycle)
            {
                stack.push((current_node, current_rank));

                let child_rank = determine_child_rank(nodes, &node_info, &child_node, current_rank);

                let vertical_rank = ranks.entry(child_rank).or_insert(BTreeMap::new());
                vertical_rank.insert(vertical_rank.len(), NodePlace::Node(child_node.to_owned()));
                node_info.visit_node(child_node);
                rank_map.insert(child_node.to_owned(), child_rank);
                current_node = &child_node.as_ref();
                current_rank = child_rank;
            }
        }
    }
    add_in_context_nodes(edges, &mut ranks);
    ranks
}

///
///
///
///
fn determine_child_rank(
    nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>,
    node_info: &BTreeMap<String, NodeInfo>,
    child_node: &str,
    current_rank: usize,
) -> usize {
    let node = nodes.get(child_node).unwrap();
    // If one parent is on the same rank, put the child one rank further down.
    let max_parent_rank = node_info
        .get(child_node)
        .unwrap()
        .parents
        .iter()
        .map(|p| {
            node_info
                .get(p.to_owned())
                .map(|p| p.rank)
                .unwrap() // All nodes exist in node_info map
                .unwrap_or(current_rank)
        })
        .max()
        .unwrap();

    let r = node
        .borrow()
        .get_forced_level()
        .unwrap_or_else(|| std::cmp::max(current_rank + 1, max_parent_rank + 1));
    r
}


///
///
/// Finds the next child node and returns it together with its rank.
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
        if !allow_unranked_parent {
            x.sort_by(|a, b| {
                node_info
                    .get(&b.0.to_owned())
                    .unwrap()
                    .max_child_rank
                    .cmp(&node_info.get(&a.0.to_owned()).unwrap().max_child_rank)
            });
        }
        if let Some(opt_child) = x
            .iter()
            .filter(|(id, _)| match allow_unranked_parent {
                false => count_unvisited_parents(&node_info, id) == 0,
                true => true, // Architecture view may contain circles.
                              // We have to skip this this filter, otherwise we end in an endless loop.
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
            return Some(nodes.keys().find(|&x| x == opt_child).unwrap());
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
                                if previous_node_with_connection(x, n, v_ranks, edges) {
                                    // Make left/rigth distribution more even
                                    if i % 2 == 1 {
                                        i += 1;
                                    }
                                    true
                                } else {
                                    i % 2 == 0
                                }
                            });
                        match &left.len() {
                            1 => new_rank.push(NodePlace::Node(left.get(0).unwrap().to_owned())),
                            2.. => new_rank.push(NodePlace::MultipleNodes(
                                left.iter().map(|x| x.to_owned()).collect(),
                            )),
                            _ => (),
                        }
                        new_rank.push(NodePlace::Node(n.to_owned()));
                        match &right.len() {
                            1 => new_rank.push(NodePlace::Node(right.get(0).unwrap().to_owned())),
                            2.. => new_rank.push(NodePlace::MultipleNodes(
                                right.iter().map(|x| x.to_owned()).collect(),
                            )),
                            _ => (),
                        }
                    } else {
                        new_rank.push(NodePlace::Node(n.to_owned()));
                    }
                }
                NodePlace::MultipleNodes(_) => (),
            }
        }
        v_ranks.clear();
        for (h, np) in new_rank.into_iter().enumerate() {
            // Only add visited nodes here, when changing vertical rank.
            // If done within one horizontal rank, the above logic will fail.
            match &np {
                NodePlace::Node(n) => {
                    visited_nodes.insert(n.to_owned());
                }
                NodePlace::MultipleNodes(ns) => ns.iter().for_each(|n| {
                    visited_nodes.insert(n.to_owned());
                }),
            }
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
    cur_rank: &BTreeMap<usize, NodePlace>,
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
) -> bool {
    cur_rank.values().any(|np| match np {
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
///
///
///
///
///
pub(crate) fn get_forced_levels<'a>(
    nodes: &'a BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
    levels: &BTreeMap<&'a str, Vec<&'a str>>,
) -> BTreeMap<&'a str, usize> {
    let mut forced_levels = BTreeMap::new();
    let depths = get_depths(nodes, edges);
    for nodes in levels.values() {
        if let Some(max_depth) = nodes.iter().filter_map(|n| depths.get(n)).max() {
            for node in nodes {
                forced_levels.insert(*node, *max_depth);
            }
        }
    }
    forced_levels
}

///
/// Calculate the depth of each node.
///
/// TODO depth of in_context_of nodes
///
/// There can be no forced levels yet.
///
fn get_depths<'a>(
    nodes: &'a BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
) -> BTreeMap<&'a str, usize> {
    let mut depths = BTreeMap::new();
    let mut current_nodes = get_root_nodes(nodes, edges);
    let mut visited_nodes = HashSet::new();

    current_nodes.iter().for_each(|&n| {
        visited_nodes.insert(n);
    });

    let mut depth = 0usize;
    while !current_nodes.is_empty() {
        let mut child_nodes = Vec::new();
        for cur_node in &current_nodes {
            depths.insert(*cur_node, depth);
            if let Some(children) = edges.get(*cur_node) {
                let mut c_nodes: Vec<&str> = children
                    .iter()
                    .filter_map(|(target, edge_type)| match edge_type {
                        EdgeType::OneWay(SingleEdge::SupportedBy)
                        | EdgeType::OneWay(SingleEdge::Composite)
                        | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                        | EdgeType::TwoWay((SingleEdge::SupportedBy, _))
                        | EdgeType::TwoWay((_, SingleEdge::Composite))
                        | EdgeType::TwoWay((SingleEdge::Composite, _)) => Some(target.as_str()),
                        _ => None,
                    })
                    .filter(|&n| !visited_nodes.contains(n))
                    .collect();
                c_nodes.iter().for_each(|&n| {
                    visited_nodes.insert(n);
                });
                child_nodes.append(&mut c_nodes);
            }
        }
        depth += 1;
        current_nodes.clear();
        current_nodes.append(&mut child_nodes);
    }
    depths
}

///
/// Get the root nodes of the given (nodes, edges) graph
///
/// Root nodes are considered those nodes that have no incoming edges.
///
/// Forced levels are not considered here, since they are set based on this evaluation.
///
fn get_root_nodes<'a>(
    nodes: &'a BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &'a BTreeMap<String, Vec<(String, EdgeType)>>,
) -> Vec<&'a str> {
    let mut root_nodes: Vec<&str> = nodes.keys().map(|n| n.as_str()).collect();
    for t_edges in edges.values() {
        for (target, _) in t_edges {
            root_nodes.retain(|id| id != target);
        }
    }
    root_nodes
}

///
///
///
///
///
pub fn calculate_parent_node_map(
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut parent_map = BTreeMap::new();
    for (child, target_edges) in edges {
        for (target_edge, _) in target_edges {
            parent_map
                .entry(target_edge.to_owned())
                .or_insert_with(BTreeSet::new)
                .insert(child.to_owned());
        }
    }
    parent_map
}
