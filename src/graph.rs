use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{edges::EdgeType, nodes::Node};

pub(crate) fn rank_nodes(
    nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
) -> Vec<Vec<String>> {
    let mut ranks = Vec::new();

    // Copy IDs
    let mut n_ids: BTreeSet<String> = nodes
        .iter()
        // Filter nodes with forced level
        .filter(|(_, node)| !matches!(node.borrow().get_forced_level(), Some(x) if x != 0))
        .map(|(id, _)| id.to_owned())
        .collect();
    // Find root nodes
    for t_edges in edges.values() {
        for (target, _) in t_edges {
            n_ids.remove(target);
        }
    }
    // Add inContextOf nodes again
    n_ids.append(&mut find_in_context_nodes(edges, &n_ids));
    let mut rank = 0;
    loop {
        let mut v: Vec<_> = n_ids.iter().map(|x| x.to_owned()).collect();
        if dbg!(count_crossings_same_rank(edges, &v)) > 0  {
            swap_same_rank(edges, &mut v);
        }
        ranks.insert(rank as usize, v);
        // Find children
        n_ids = find_child_nodes(nodes, edges, rank, &n_ids);
        if n_ids.is_empty() {
            break;
        }
        rank += 1;
    }
    ranks
}

fn swap_same_rank(
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    rank_nodes: &mut Vec<String>,
) {
    let mut s : Option<(usize, usize)> = None;
    for (i, rn) in rank_nodes.iter().enumerate() {
        if let Some(edges) = edges.get(rn) {
            for (id, _) in edges {
                if let Some(x) = rank_nodes.iter().position(|x| x == id) {
                    if (x as i64 - i as i64).abs() > 1 {
                        s = Some((x, (x as i64 -i as i64).abs()as usize/2 ));
                    }
                }
            }
        }
    }
    if let Some((x,y)) = s {
        rank_nodes.swap(x, y);
    }
}

fn count_crossings_same_rank(
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

fn find_child_nodes(
    nodes: &BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    rank: u32,
    parents: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut children = BTreeSet::new();
    for p in parents {
        // Direct children
        if let Some(es) = edges.get(p) {
            let mut targets = es
                .iter()
                .filter_map(|(id, et)| match et {
                    EdgeType::SupportedBy => Some(id.to_owned()),
                    _ => None,
                })
                .filter(
                    // Filter forced level nodes
                    |id| !matches!(nodes.get(id).unwrap().borrow().get_forced_level(), Some(x) if x != rank + 1)
                )
                .collect();
            children.append(&mut targets);
        }
    }
    children.append(&mut find_in_context_nodes(edges, &children));
    // Add forced level nodes
    for (id, n) in nodes.iter() {
        if n.borrow().get_forced_level() == Some(rank + 1) {
            children.insert(id.to_owned());
        }
    }
    children
}

fn find_in_context_nodes(
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    parents: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut additional_nodes = BTreeSet::<String>::new();
    for id in parents {
        if let Some(target) = edges.get(id) {
            let mut an = target
                .iter()
                .filter_map(|(tn, et)| match et {
                    EdgeType::InContextOf => Some(tn.to_owned()),
                    _ => None,
                })
                .collect::<BTreeSet<String>>();
            additional_nodes.append(&mut an);
        }
    }
    additional_nodes
}
