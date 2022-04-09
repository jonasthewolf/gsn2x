use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashSet},
    rc::Rc,
};

use crate::{
    edges::{EdgeType, SingleEdge},
    nodes::{invisible_node::InvisibleNode, Node},
};

#[derive(Debug)]
pub enum NodePlace {
    Node(String),
    MultipleNodes(Vec<String>),
}

impl NodePlace {
    pub(crate) fn get_max_width(&self, nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>) -> u32 {
        match self {
            NodePlace::Node(n) => {
                let n = nodes.get(n).unwrap().borrow();
                n.get_width() / 2
            }
            NodePlace::MultipleNodes(np) => np
                .iter()
                .map(|n| nodes.get(n).unwrap().borrow().get_width())
                .max()
                .unwrap(),
        }
    }

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

///
///
///
///
///
pub(crate) fn rank_nodes<'a>(
    nodes: &'a mut BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &'a mut BTreeMap<String, Vec<(String, EdgeType)>>,
) -> BTreeMap<usize, BTreeMap<usize, NodePlace>> {
    let mut ranks = BTreeMap::new();
    let mut visited_nodes: BTreeSet<String> = BTreeSet::new();
    let edge_map = calculate_parent_node_map(edges);

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
    let root_nodes: Vec<String> = if n_ids.is_empty() {
        // No root nodes are found.
        // This can actually only happen in architecture view.
        // Take the first node and start from there.
        vec![nodes.iter().nth(0).unwrap().0.to_owned()]
    } else {
        n_ids.iter().map(|x| x.to_owned()).collect()
    };
    // Perform depth first search for SupportedBy child nodes.
    for (horiz_rank, n) in root_nodes.into_iter().enumerate() {
        visited_nodes.insert(n.to_owned());
        {
            let vertical_rank = ranks.entry(0).or_insert(BTreeMap::new());
            vertical_rank.insert(horiz_rank, NodePlace::Node(n.to_owned()));
        }
        let mut stack = Vec::new();
        let mut current_node = n;
        let mut current_rank;
        stack.push((current_node.to_owned(), 0));
        while let Some((p_id, p_r)) = stack.pop() {
            current_node = p_id.to_owned();
            current_rank = p_r;
            while let Some((child_node, child_rank)) = find_next_child_node(
                nodes,
                edges,
                current_rank,
                &visited_nodes,
                &current_node,
                &edge_map,
            ) {
                stack.push((current_node.to_owned(), current_rank));

                let mut last_h_rank = ranks
                    .get_mut(&current_rank)
                    .unwrap()
                    .iter()
                    .find(|(_, np)| match np {
                        NodePlace::Node(n) => n == &current_node,
                        NodePlace::MultipleNodes(_) => false,
                    })
                    .map(|(&hr, _)| hr)
                    .unwrap();
                // Add invisible nodes on the vertical ranks if at least one rank is skipped
                for i in current_rank + 1..child_rank {
                    let vertical_rank = ranks.entry(i).or_insert(BTreeMap::new());
                    let cloned_node =
                        clone_invisible_node(nodes, &NodePlace::Node(child_node.to_owned()));
                    last_h_rank = vertical_rank.len();
                    vertical_rank
                        .insert(vertical_rank.len(), NodePlace::Node(cloned_node.to_owned()));
                    // Remove original edge
                    let orig_edges = edges.get_mut(&current_node).unwrap();
                    let orig_edge_id = orig_edges
                        .iter()
                        .position(|e| {
                            e == &(
                                child_node.to_owned(),
                                EdgeType::OneWay(SingleEdge::SupportedBy),
                            )
                        })
                        .unwrap();
                    orig_edges.remove(orig_edge_id);
                    // Add two new edges.
                    orig_edges.push((cloned_node.to_owned(), EdgeType::Invisible));
                    let new_entry = edges.entry(cloned_node.to_owned()).or_insert(Vec::new());
                    new_entry.push((
                        child_node.to_owned(),
                        EdgeType::OneWay(SingleEdge::SupportedBy),
                    ));
                }

                // Move nodes to the right if child rank contains too few nodes
                let vertical_rank = ranks.entry(child_rank).or_insert(BTreeMap::new());
                for i in vertical_rank.len()..last_h_rank {
                    if let Some(b_node) = ranks.get(&(child_rank - 1)).unwrap().get(&i) {
                        let cloned_node = NodePlace::Node(clone_invisible_node(nodes, b_node));
                        let vertical_rank = ranks.entry(child_rank).or_insert(BTreeMap::new());
                        vertical_rank.insert(i, cloned_node);
                    }
                }
                let vertical_rank = ranks.entry(child_rank).or_insert(BTreeMap::new());
                vertical_rank.insert(vertical_rank.len(), NodePlace::Node(child_node.to_owned()));
                visited_nodes.insert(child_node.to_owned());
                current_node = child_node;
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
///
fn clone_invisible_node(
    nodes: &mut BTreeMap<String, Rc<RefCell<dyn Node>>>,
    origin: &NodePlace,
) -> String {
    match origin {
        NodePlace::Node(n) => {
            let mut invisible_node = InvisibleNode::from(nodes.get(n).unwrap());
            let mut invisible_node_id = invisible_node.get_id().to_owned();
            let mut i = 0;
            while nodes.contains_key(&invisible_node_id) {
                invisible_node_id = format!("{}{}", invisible_node.get_id(), i);
                i += 1;
            }
            invisible_node.set_id(&invisible_node_id);
            nodes.insert(
                invisible_node.get_id().to_owned(),
                Rc::new(RefCell::new(invisible_node)),
            );
            invisible_node_id
        }
        NodePlace::MultipleNodes(ns) => {
            let mut invisible_node = InvisibleNode::from(nodes.get(ns.get(0).unwrap()).unwrap());
            let (max_height, max_width) = ns
                .iter()
                .map(|n| {
                    let nb = nodes.get(n).unwrap().borrow();
                    (nb.get_width(), nb.get_height())
                })
                .max()
                .unwrap();
            invisible_node.borrow_mut().set_size(max_width, max_height);
            let invisible_node_id = ns.join("_");
            nodes.insert(
                invisible_node.get_id().to_owned(),
                Rc::new(RefCell::new(invisible_node)),
            );
            invisible_node_id
        }
    }
}

///
///
/// Finds the next child node and returns it together with its rank.
/// The rank could be constraint by the forced level.
///
fn find_next_child_node(
    nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    rank: usize,
    visited_nodes: &BTreeSet<String>,
    current: &str,
    edge_map: &BTreeMap<String, HashSet<String>>,
) -> Option<(String, usize)> {
    if let Some(p) = edges.get(current) {
        if let Some(opt_child) = p
            .iter()
            .filter(|(id, _)| count_unvisited_parents(edge_map, visited_nodes, id) == 0)
            .filter_map(|(id, et)| match et {
                EdgeType::OneWay(SingleEdge::SupportedBy)
                | EdgeType::OneWay(SingleEdge::Composite)
                | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                | EdgeType::TwoWay((SingleEdge::SupportedBy, _))
                | EdgeType::TwoWay((_, SingleEdge::Composite))
                | EdgeType::TwoWay((SingleEdge::Composite, _)) => Some(id.to_owned()),
                _ => None,
            })
            .find(|id| !visited_nodes.contains(id))
        {
            let node = nodes.get(&opt_child).unwrap();
            let r = node.borrow().get_forced_level().unwrap_or(rank + 1);
            return Some((opt_child, r));
        }
    }
    None
}

///
///
///
///
fn count_unvisited_parents(
    edge_map: &BTreeMap<String, HashSet<String>>,
    visited_nodes: &BTreeSet<String>,
    current: &str,
) -> usize {
    let mut unvisited_parents = 0usize;
    // None can actually only happen for root nodes
    if let Some(parents) = edge_map.get(current) {
        for parent in parents {
            if !visited_nodes.contains(parent) {
                unvisited_parents += 1;
            }
        }
    }
    unvisited_parents
}

///
///
///
///
///
///
fn add_in_context_nodes(
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    ranks: &mut BTreeMap<usize, BTreeMap<usize, NodePlace>>,
) {
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
                            .collect::<Vec<String>>()
                            .into_iter()
                            .partition(|_| {
                                i += 1;
                                i % 2 == 0
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
        for (h, n) in new_rank.into_iter().enumerate() {
            v_ranks.insert(h, n);
        }
    }
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
        let max_depth = nodes.iter().map(|n| depths.get(n).unwrap()).max().unwrap();
        for node in nodes {
            forced_levels.insert(*node, *max_depth);
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
                    .collect();
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
/// TODO check if forced levels are already set
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
fn calculate_parent_node_map(
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
) -> BTreeMap<String, HashSet<String>> {
    let mut parent_map = BTreeMap::new();
    for (child, target_edges) in edges {
        for (target_edge, _) in target_edges {
            parent_map
                .entry(target_edge.to_owned())
                .or_insert_with(HashSet::new)
                .insert(child.to_owned());
        }
    }
    parent_map
}
