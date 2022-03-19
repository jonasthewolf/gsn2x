use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{
    edges::EdgeType,
    nodes::{invisible_node::Invisible, Node},
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
    let root_nodes: Vec<_> = n_ids.iter().map(|x| x.to_owned()).collect();
    // Perform depth first searcb for SupportedBy child nodes.
    for (horiz_rank, n) in root_nodes.into_iter().enumerate() {
        visited_nodes.insert(n.to_owned());
        let v_r = ranks.entry(0).or_insert(BTreeMap::new());
        v_r.insert(horiz_rank, NodePlace::Node(n.to_owned()));
        let mut loc_stack = Vec::new();
        let mut cur_node = n;
        let mut cur_rank;
        loc_stack.push((cur_node.to_owned(), 0));
        while let Some((x, p_r)) = loc_stack.pop() {
            cur_node = x.to_owned();
            cur_rank = p_r;
            while let Some((c, c_r)) =
                find_next_child_node(nodes, edges, cur_rank, &visited_nodes, &cur_node)
            {
                loc_stack.push((cur_node.to_owned(), cur_rank));
                // Add invisible nodes
                for i in p_r + 1..c_r {
                    let inv = Invisible::from(nodes.get(&c).unwrap());
                    let inv_id = inv.get_id().to_owned();
                    nodes.insert(inv.get_id().to_owned(), Rc::new(RefCell::new(inv)));
                    // TODO Replace edge with two edges and new edge type
                    let v_r = ranks.entry(i).or_insert(BTreeMap::new());
                    // TODO Position of invisible node is wrong.
                    v_r.insert(v_r.len(), NodePlace::Node(inv_id.to_owned()));
                }
                // TODO If more than one incoming edge, the lowest rank should move unforced elements down
                let v_r = ranks.entry(c_r).or_insert(BTreeMap::new());
                v_r.insert(v_r.len(), NodePlace::Node(c.to_owned()));
                visited_nodes.insert(c.to_owned());
                cur_node = c;
                cur_rank = c_r;
            }
        }
    }
    dbg!(&ranks);
    add_in_context_nodes(edges, &mut ranks);
    dbg!(ranks)
}

fn _swap_same_rank(
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    rank_nodes: &mut Vec<String>,
) {
    let mut s: Option<(usize, usize)> = None;
    for (i, rn) in rank_nodes.iter().enumerate() {
        if let Some(edges) = edges.get(rn) {
            for (id, _) in edges {
                if let Some(x) = rank_nodes.iter().position(|x| x == id) {
                    if (x as i64 - i as i64).abs() > 1 {
                        s = Some((x, (x as i64 - i as i64).abs() as usize / 2));
                    }
                }
            }
        }
    }
    if let Some((x, y)) = s {
        rank_nodes.swap(x, y);
    }
}

fn _count_crossings_same_rank(
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    rank_nodes: &Vec<String>,
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
    for p in edges.get(current) {
        if let Some(opt_child) = p
            .iter()
            .filter_map(|(id, et)| match et {
                EdgeType::SupportedBy => Some(id.to_owned()),
                _ => None,
            })
            .filter(|id| !visited_nodes.contains(id))
            .next()
        {
            let node = nodes.get(&opt_child).unwrap();
            let r = node
                .borrow()
                .get_forced_level()
                .or_else(|| Some(rank + 1))
                .unwrap();
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
                        let (left, right): (Vec<(usize, String)>, Vec<(usize, String)>) = target
                            .iter()
                            .filter_map(|(tn, et)| match et {
                                EdgeType::InContextOf => Some(tn.to_owned()),
                                _ => None,
                            })
                            .collect::<Vec<String>>()
                            .into_iter()
                            .enumerate()
                            .partition(|(i, _)| i % 2 == 0);
                        match &left.len() {
                            1 => new_rank.push(NodePlace::Node(left.get(0).unwrap().1.to_owned())),
                            2.. => new_rank.push(NodePlace::MultipleNodes(
                                left.iter().map(|(_, x)| x.to_owned()).collect(),
                            )),
                            _ => (),
                        }
                        new_rank.push(NodePlace::Node(n.to_owned()));
                        match &right.len() {
                            1 => new_rank.push(NodePlace::Node(right.get(0).unwrap().1.to_owned())),
                            2.. => new_rank.push(NodePlace::MultipleNodes(
                                right.iter().map(|(_, x)| x.to_owned()).collect(),
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
