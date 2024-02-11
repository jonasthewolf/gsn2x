use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};

use crate::dirgraph::DirectedGraph;

use super::{edges::EdgeType, nodes::SvgNode, util::point2d::Point2D};

///
/// Struct for margin setup
///
pub struct Margin {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

///
/// Default values for Margin
///
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

///
/// A Cell is a list of node IDs
///
pub(crate) trait Cell {
    ///
    /// Get the maximum width of all nodes within the cell
    ///
    fn get_max_width(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>) -> i32;

    ///
    /// Get the x coordinate of the cell
    ///
    fn get_x(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>) -> i32;

    ///
    /// Get the center y coordinate of the cell
    ///
    fn get_center_y(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>) -> i32;

    ///
    /// Set the x coordinates of all nodes in the cell
    ///
    fn set_position(
        &self,
        nodes: &BTreeMap<String, RefCell<SvgNode>>,
        margin: &Margin,
        pos: Point2D<i32>,
    );

    ///
    /// Get the height of a cell, including inner margins
    ///
    fn get_height(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>, margin: &Margin) -> i32;
}

impl Cell for Vec<&str> {
    ///
    ///
    ///
    fn get_max_width(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>) -> i32 {
        self.iter()
            .filter_map(|&n| nodes.get(n))
            .map(|n| n.borrow().get_width())
            .max()
            .unwrap_or_default()
    }

    ///
    ///
    ///
    fn get_x(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>) -> i32 {
        let n = nodes.get(self.first().unwrap().to_owned()).unwrap(); // unwraps ok, since nodes must exist.
        n.borrow().get_position().x
    }

    ///
    ///
    ///
    fn get_center_y(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>) -> i32 {
        (self
            .iter()
            .map(|&n| nodes.get(n).unwrap().borrow().get_position().y as f64)
            .sum::<f64>()
            / self.len() as f64) as i32
    }

    ///
    ///
    ///
    fn set_position(
        &self,
        nodes: &BTreeMap<String, RefCell<SvgNode>>,
        margin: &Margin,
        pos: Point2D<i32>,
    ) {
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

    ///
    ///
    ///
    fn get_height(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>, margin: &Margin) -> i32 {
        self.iter()
            .filter_map(|&x| nodes.get(x))
            .map(|n| n.borrow().get_height())
            .sum::<i32>()
            + (margin.top + margin.bottom) * (self.len() - 1) as i32
    }
}

///
/// Iteratively move nodes horizontally until no movement detected
///  
///
pub(super) fn layout_nodes(
    graph: &DirectedGraph<'_, RefCell<SvgNode>, EdgeType>,
    ranks: &[Vec<Vec<&str>>],
    margin: &Margin,
) -> (i32, i32) {
    let nodes = graph.get_nodes();
    // This number should be safe that it yields a final, good looking image
    let limit = 3 * ranks.len();
    for run in 0..=limit {
        let mut changed = false;
        let mut y = margin.top;
        let mut cells_with_centered_parents = HashSet::new();
        for v_rank in ranks.iter() {
            let mut x = margin.left;
            let dy_max = get_max_height(nodes, margin, v_rank);
            y += dy_max / 2;
            for cell in v_rank.iter() {
                let w = cell.get_max_width(nodes);
                let old_x = cell.get_x(nodes);
                x = std::cmp::max(x + w / 2, old_x);
                // Not at the first run
                if run > 0 {
                    if let Some(new_x) = has_node_to_be_moved(
                        graph,
                        cell,
                        margin,
                        &mut cells_with_centered_parents,
                        run,
                    ) {
                        if new_x > x {
                            x = std::cmp::max(x, new_x);
                            // eprintln!("Changed {:?} {} {} {}", &cell, x, old_x, new_x);
                            changed = true;
                        }
                    }
                }
                cell.set_position(nodes, margin, Point2D { x, y });
                x += w / 2 + margin.left + margin.right;
            }
            y += margin.bottom + dy_max / 2 + margin.top;
        }
        if !(run == 0 || changed) {
            if run <= limit {
                println!("OK");
            }
            break;
        }
        if changed && run == limit {
            println!("Diagram took too many iterations ({run}). See documentation (https://jonasthewolf.github.io/gsn2x/) for hints how to solve this situation.");
        }
    }
    calculate_size_of_document(nodes, ranks, margin)
}

///
///  Calculate size of document
///
///
fn calculate_size_of_document(
    nodes: &BTreeMap<String, RefCell<SvgNode>>,
    ranks: &[Vec<Vec<&str>>],
    margin: &Margin,
) -> (i32, i32) {
    let width = ranks
        .iter()
        .map(|rank| {
            let n = rank.last().unwrap(); // unwrap ok, since there is at least one rank.
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
    nodes: &BTreeMap<String, RefCell<SvgNode>>,
    margin: &Margin,
    rank: &[Vec<&str>],
) -> i32 {
    rank.iter()
        .map(|cell| cell.get_height(nodes, margin))
        .max()
        .unwrap()
}

///
/// Get the center of all nodes in the `set`.
///
fn get_center(nodes: &BTreeMap<String, RefCell<SvgNode>>, set: &[&str]) -> i32 {
    let x_values: Vec<_> = set
        .iter()
        .map(|&node| nodes.get(node).unwrap().borrow().get_position().x)
        .collect();
    let max = x_values.iter().max().unwrap_or(&0);
    let min = x_values.iter().min().unwrap_or(&0);
    (max + min) / 2
}

///
/// Decides if a node has to be moved.
///
fn has_node_to_be_moved<'b>(
    graph: &'b DirectedGraph<'b, RefCell<SvgNode>, EdgeType>,
    cell: &Vec<&str>,
    margin: &Margin,
    moved_nodes: &mut HashSet<&'b str>,
    run: usize,
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
    let nodes = graph.get_nodes();
    let my_x = cell.get_x(&nodes);
    let child_len = children.len();
    // If node has children, center over them
    if child_len > 0 {
        if child_len == 1 && my_x <= children.get_x(&nodes) && parents.len() == 0 {
            // If node has no parents and exactly one child, center over it
            let mut my_new_x = children.get_x(&nodes);
            let all_parent_cells = children
                .iter()
                .filter(|c| graph.get_real_children(c).len() > 0)
                .filter_map(|c| match graph.get_real_parents(&c) {
                    p if p.len() > 1 => Some(p.iter().map(|&p| vec![p]).collect::<Vec<_>>()),
                    _ => None,
                })
                .flatten()
                .collect::<Vec<_>>();
            let my_pos = all_parent_cells.iter().position(|p| cell == p).unwrap_or(0);
            let half = all_parent_cells.len() / 2;
            // let mut last_sub = 0; // FIXME more than two parents
            for index in (my_pos..half).rev() {
                my_new_x -= margin.left;
                let last_sub = all_parent_cells[index].get_max_width(&nodes) / 2;
                my_new_x -= last_sub;
            }
            // children
            //     .into_iter()
            //     .filter(|&c| dbg!(graph.get_real_parents(dbg!(c)).len() > 1))
            //     .for_each(|c| {moved_nodes.insert(c); });
            // my_new_x -= last_sub;
            Some(my_new_x)
        } else {
            // Only center over the children that have no other parents...
            let center_children = children
                .into_iter()
                .filter(|&c| graph.get_real_parents(c).len() == 1)
                .collect::<Vec<_>>();
            // ... and remember them. We don't move them later.
            if center_children.len() > 1 {
                center_children.iter().for_each(|c| {
                    moved_nodes.insert(c);
                });
            }
            // On the first iteration of moving, ensure that we have moved the node with a single parent
            // exactly under its parent. This might happen if e.g. there is a large in-context node at the parent.
            // Then the child is too far left and never moved.
            // We disregard the margin.left, to not create unnecessary white space at the left.
            let min_x = if run <= 1
                && parents.len() == 1
                && graph.get_real_children(parents.first().unwrap()).len() == 1
            {
                let p_x = parents.get_x(&nodes);
                if p_x <= margin.left {
                    0
                } else {
                    p_x
                }
            } else {
                0
            };
            Some(std::cmp::max(min_x, get_center(&nodes, &center_children)))
        }
    } else if parents.len() > 0 {
        // else center under parents if the parent has centered above the current cell and other nodes
        if cell.iter().any(|c| moved_nodes.contains(c)) {
            None
        } else {
            Some(get_center(&nodes, &parents))
        }
    } else if same_rank_parents.len() == 1 {
        // Move same rank child closer to parent
        Some(move_closer(
            &nodes,
            cell,
            same_rank_parents.first().unwrap(),
            margin,
        ))
    } else {
        None
    }
}

///
/// Move "in context" nodes closer to their parents.
/// Only nodes to the left of their parents are moved.
///
fn move_closer(
    nodes: &BTreeMap<String, RefCell<SvgNode>>,
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
