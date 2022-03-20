use std::{
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

pub(crate) fn rank_nodes(
    nodes: &mut BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
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
        let vertical_rank = ranks.entry(0).or_insert(BTreeMap::new());
        vertical_rank.insert(horiz_rank, NodePlace::Node(n.to_owned()));
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
                // Add invisible nodes on the vertical ranks if at least one rank is skipped
                for i in current_rank + 1..child_rank {
                    let inv = InvisibleNode::from(nodes.get(&child_node).unwrap());
                    let inv_id = inv.get_id().to_owned();
                    nodes.insert(inv.get_id().to_owned(), Rc::new(RefCell::new(inv)));
                    // TODO Replace edge with two edges and new edge type
                    let vertical_rank = ranks.entry(i).or_insert(BTreeMap::new());
                    // TODO Position of invisible node is wrong.
                    vertical_rank.insert(vertical_rank.len(), NodePlace::Node(inv_id.to_owned()));
                }
                // TODO If more than one incoming edge, the lowest rank should move unforced elements down

                let cur_h_rank = dbg!(ranks.get(&current_rank).unwrap().len());
                let vertical_rank = ranks.entry(child_rank).or_insert(BTreeMap::new());
                for i in vertical_rank.len()..cur_h_rank {
                    // let inv = Invisible { width: 50, height: 50, x: 0, y: 0, id: format!("inv{}",i) };
                    // nodes.insert(inv.get_id().to_owned(), Rc::new(RefCell::new(inv)));
                    // vertical_rank.insert(i, NodePlace::Node(format!("inv{}",i)));
                }
                vertical_rank.insert(vertical_rank.len(), NodePlace::Node(child_node.to_owned()));
                visited_nodes.insert(child_node.to_owned());
                current_node = child_node;
                current_rank = child_rank;
            }
        }
    }
    dbg!(&ranks);
    add_in_context_nodes(edges, &mut ranks);
    dbg!(ranks)
}

fn _count_crossings_same_rank(
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    rank_nodes: &[String],
) -> usize {
    let mut sum = 0usize;
    for (i, rn) in rank_nodes.iter().enumerate() {
        if let Some(edges) = edges.get(rn) {
            sum += edges
                .iter()
                .filter(|(id, _)| {
                    if let Some(x) = rank_nodes.iter().position(|x| x == id) {
                        if (x as i64 - i as i64).abs() > 1 {
                            return true;
                        }
                    }
                    false
                })
                .count()
        }
    }
    sum
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
