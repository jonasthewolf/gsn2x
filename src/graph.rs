use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{
    edges::EdgeType,
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
            NodePlace::MultipleNodes(np) => {
                let mut max = 0;
                for n in np {
                    let n = nodes.get(n).unwrap().borrow();
                    max = std::cmp::max(max, n.get_width());
                }
                max
            }
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

pub(crate) fn rank_nodes(
    nodes: &mut BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &mut BTreeMap<String, Vec<(String, EdgeType)>>,
) -> BTreeMap<usize, BTreeMap<usize, NodePlace>> {
    let mut ranks = BTreeMap::new();
    let mut visited_nodes: BTreeSet<String> = BTreeSet::new();

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
    let root_nodes = n_ids.iter().map(|x| x.to_owned());
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
            while let Some((child_node, child_rank)) =
                find_next_child_node(nodes, edges, current_rank, &visited_nodes, &current_node)
            {
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
                        .position(|e| e == &(child_node.to_owned(), EdgeType::SupportedBy))
                        .unwrap();
                    orig_edges.remove(orig_edge_id);
                    // Add two new edges.
                    orig_edges.push((cloned_node.to_owned(), EdgeType::Invisible));
                    let new_entry = edges.entry(cloned_node.to_owned()).or_insert(Vec::new());
                    new_entry.push((child_node.to_owned(), EdgeType::SupportedBy));
                }
                // TODO If more than one incoming edge, the lowest rank should move unforced elements down

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

fn find_next_child_node(
    nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    rank: usize,
    visited_nodes: &BTreeSet<String>,
    current: &str,
) -> Option<(String, usize)> {
    if let Some(p) = edges.get(current) {
        if let Some(opt_child) = p
            .iter()
            .filter_map(|(id, et)| match et {
                EdgeType::SupportedBy => Some(id.to_owned()),
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
                                EdgeType::InContextOf => Some(tn.to_owned()),
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
