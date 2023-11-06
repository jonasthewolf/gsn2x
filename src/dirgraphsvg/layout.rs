use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::dirgraph::DirectedGraph;

use super::{edges::EdgeType, nodes::Node, util::point2d::Point2D};

pub struct Margin {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Default for Margin {
    fn default() -> Self {
        Self {
            top: 20,
            right: 20,
            bottom: 20,
            left: 20,
        }
    }
}

trait Cell {
    fn get_max_width(&self, nodes: &BTreeMap<String, RefCell<Node>>) -> i32;
    fn get_x(&self, nodes: &BTreeMap<String, RefCell<Node>>) -> i32;
    fn set_position(&self, nodes: &BTreeMap<String, RefCell<Node>>, margin: &Margin, pos: Point2D);
}

impl Cell for Vec<&str> {
    fn get_max_width(&self, nodes: &BTreeMap<String, RefCell<Node>>) -> i32 {
        self.iter()
            .map(|&n| nodes.get(n).unwrap().borrow().get_width())
            .max()
            .unwrap()
    }

    fn get_x(&self, nodes: &BTreeMap<String, RefCell<Node>>) -> i32 {
        let n = nodes.get(self.first().unwrap().to_owned()).unwrap();
        n.borrow().get_position().x
    }

    fn set_position(&self, nodes: &BTreeMap<String, RefCell<Node>>, margin: &Margin, pos: Point2D) {
        let max_h = self
            .iter()
            .map(|&id| nodes.get(id).unwrap().borrow().get_height())
            .sum::<i32>()
            + (margin.top + margin.bottom) * (self.len() - 1) as i32;
        let mut y_n = pos.y - max_h / 2;
        for &id in self {
            let n = nodes.get(id).unwrap();
            let h = n.borrow().get_height();
            n.borrow_mut().set_position(&Point2D {
                x: pos.x,
                y: y_n + h / 2,
            });
            y_n += h + margin.top + margin.bottom;
        }
    }
}

//     ///
//     /// Get x value of NodePlace
//     ///
//     /// MultipleNodes have the same x, thus, just the value of the first node is used.
//     /// MultipleNodes are never empty.
//     ///
//     ///
//     pub(crate) fn get_x(&self, nodes: &BTreeMap<String, Node>) -> i32 {
//         // Unwraps are ok, since NodePlace are only created from existing nodes
//         let n = nodes.get(self.0.first().unwrap()).unwrap();
//         n.get_position().x
//     }
// }

///
/// Iteratively move nodes horizontally until no movement detected
///  
///
pub(super) fn layout_nodes(
    graph: &DirectedGraph<'_, RefCell<Node>, EdgeType>,
    ranks: &[Vec<Vec<&str>>],
    margin: &Margin,
) -> (i32, i32) {
    let mut first_run = true;
    let nodes = graph.get_nodes();
    // This number should be safe that it yields a final, good looking image
    // let limit = nodes.len() * nodes.len() * 2;
    // TODO Restore old limit
    let limit = 5;
    for limiter in 1..=limit {
        let mut changed = false;
        let mut y = margin.top;
        for v_rank in ranks.iter() {
            let mut x = margin.left;
            let dy_max = get_max_height(nodes, margin, v_rank);
            y += dy_max / 2;
            for cell in v_rank.iter() {
                let w = cell.get_max_width(nodes);
                let old_x = cell.get_x(nodes);
                x = std::cmp::max(x + w / 2, old_x);
                if !first_run {
                    if let Some(new_x) = has_node_to_be_moved(graph, cell, margin) {
                        if new_x > x {
                            x = std::cmp::max(x, new_x);
                            // eprintln!("Changed {:?} {} {} {}", &np, x, old_x, new_x);
                            changed = true;
                        }
                    }
                }
                cell.set_position(nodes, margin, Point2D { x, y });
                x += w / 2 + margin.left + margin.right;
            }
            y += margin.bottom + dy_max / 2 + margin.top;
        }
        if !(first_run || changed) {
            break;
        }
        first_run = false;
        if changed && limiter == limit {
            eprintln!("Rendering a diagram took too many iterations ({limiter}). See README.md for hints how to solve this situation.");
        }
    }
    calculate_size_of_document(nodes, ranks, margin)
}

///
///  Calculate size of document
///
///
fn calculate_size_of_document(
    nodes: &BTreeMap<String, RefCell<Node>>,
    ranks: &[Vec<Vec<&str>>],
    margin: &Margin,
) -> (i32, i32) {
    let width = ranks
        .iter()
        .map(|rank| {
            let n = rank.last().unwrap();
            n.get_x(nodes) + n.get_max_width(nodes)
        })
        .max()
        .unwrap_or(0);
    let height = ranks
        .iter()
        .map(|rank| margin.top + get_max_height(nodes, margin, rank) + margin.bottom)
        .sum();
    (width, height)
}

///
/// Get the maximum height of a rank
///
///
fn get_max_height(
    nodes: &BTreeMap<String, RefCell<Node>>,
    margin: &Margin,
    rank: &[Vec<&str>],
) -> i32 {
    rank.iter()
        .map(|id| {
            id.iter()
                .map(|&x| nodes.get(x).unwrap().borrow().get_height())
                .sum::<i32>()
                + (margin.top + margin.bottom) * (id.len() - 1) as i32
        })
        .max()
        .unwrap()
}

///
///
///
///
fn get_center(nodes: &BTreeMap<String, RefCell<Node>>, set: &[&str]) -> i32 {
    let x_values: Vec<_> = set
        .iter()
        .map(|&node| nodes.get(node).unwrap().borrow().get_position().x)
        .collect();
    let max = x_values.iter().max().unwrap();
    let min = x_values.iter().min().unwrap();
    (max + min) / 2
}

///
///
///
///
fn has_node_to_be_moved(
    graph: &DirectedGraph<'_, RefCell<Node>, EdgeType>,
    cell: &Vec<&str>,
    margin: &Margin,
) -> Option<i32> {
    let children: Vec<_> = cell
        .iter()
        .flat_map(|n| graph.get_real_children(n))
        .collect();
    let parents: Vec<_> = cell
        .iter()
        .flat_map(|n| graph.get_real_parents(n))
        .collect();
    let same_rank_parents: Vec<_> = cell
        .iter()
        .flat_map(|n| graph.get_same_ranks_parents(n))
        .collect();
    // If node has children, center over them
    if !children.is_empty() {
        // let cell_x = cell.get_x(&graph.get_nodes());
        // Don't move if all children are more left
        // if children.iter().map(|n| graph.get_nodes().get(*n).unwrap().borrow().get_position().x).all(|p| p >= cell_x) {
        // Center only over the children that have no other parents
        let center_children = children
            .into_iter()
            .filter(|&c| graph.get_real_parents(c).len() == 1)
            .collect::<Vec<_>>();
        Some(get_center(graph.get_nodes(), &center_children))
        // } else {
        //     None
        // }
    } else if !parents.is_empty() {
        // else center under its parents
        Some(get_center(graph.get_nodes(), &parents))
    } else if same_rank_parents.len() == 1 {
        // Move same rank child closer to parent
        Some(move_closer(
            graph.get_nodes(),
            cell,
            same_rank_parents.first().unwrap(),
            margin,
        ))
    } else {
        None
    }
}

///
///
///
///
fn move_closer(
    nodes: &BTreeMap<String, RefCell<Node>>,
    cell: &Vec<&str>,
    parent: &str,
    margin: &Margin,
) -> i32 {
    let parent_x = nodes.get(parent).unwrap().borrow().get_position().x;
    let cell_x = cell.get_x(nodes);
    let parent_width = nodes.get(parent).unwrap().borrow().get_width();
    let cell_width = cell.get_max_width(nodes);
    if parent_x > cell_x {
        parent_x - parent_width / 2 - margin.left - margin.right - cell_width / 2
    } else {
        cell_x - cell_width / 2 - margin.left - margin.right - parent_width / 2
    }
}

#[cfg(test)]
mod test {}
