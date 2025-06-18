use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};

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
    /// Get the maximum width of all nodes in the cell
    ///
    fn get_max_width(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>) -> i32 {
        self.iter()
            .filter_map(|&n| nodes.get(n))
            .map(|n| n.borrow().get_width())
            .max()
            .unwrap_or_default()
    }

    ///
    /// Get the x coordinate of the cell based on the first node.
    /// All node of the cell have the same x coordinate.
    ///
    fn get_x(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>) -> i32 {
        let n = nodes.get(self.first().unwrap().to_owned()).unwrap(); // unwraps ok, since nodes must exist.
        n.borrow().get_position().x
    }

    ///
    /// Get the center y coordinate of the cell.
    /// All nodes of a cell are vertically stacked.
    ///
    fn get_center_y(&self, nodes: &BTreeMap<String, RefCell<SvgNode>>) -> i32 {
        (self
            .iter()
            .map(|&n| nodes.get(n).unwrap().borrow().get_position().y as f64)
            .sum::<f64>()
            / self.len() as f64) as i32
    }

    ///
    /// Set the position of the cell.
    /// The x coordinate is the same for all nodes.
    /// The nodes are vertically stacked and separated by the top and bottom margin.
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
    /// Get the height of the cell by iteration of all nodes in the cell.
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
    let mut unstable_nodes = BTreeMap::new();
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
                    if let Some(new_x) =
                        has_node_to_be_moved(graph, cell, margin, &mut cells_with_centered_parents)
                    {
                        if new_x > x {
                            x = std::cmp::max(x, new_x);
                            // Often needed for debugging.
                            // eprintln!("Run {}: Changed {:?} {} {} {}", run, &cell, x, old_x, new_x);
                            unstable_nodes
                                .entry(cell)
                                .and_modify(|e| *e += 1)
                                .or_insert(1);
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
            println!(
                "Diagram took too many iterations ({run}). See documentation (https://jonasthewolf.github.io/gsn2x/) for hints how to solve this situation."
            );
            if let Some(max_value) = unstable_nodes.values().max() {
                let nodes_to_report = unstable_nodes
                    .iter()
                    .filter_map(|(c, v)| {
                        if v == max_value {
                            Some(c.join(", "))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                println!(
                    "The following nodes might cause the problem: {}",
                    nodes_to_report
                );
            }
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
            n.get_x(nodes) + n.get_max_width(nodes) / 2 + margin.right // /2 because x is the center coordinate
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
/// TODO Move single challenging parents, under single challenged nodes
///
fn has_node_to_be_moved<'b>(
    graph: &'b DirectedGraph<'b, RefCell<SvgNode>, EdgeType>,
    cell: &Vec<&str>,
    margin: &Margin,
    moved_nodes: &mut HashSet<&'b str>,
) -> Option<i32> {
    let children: Vec<_> = cell
        .iter()
        .flat_map(|n| graph.get_real_children(n))
        .collect();
    let parents: Vec<_> = cell
        .iter()
        .flat_map(|n| graph.get_real_parents(n))
        .collect();
    // Collect in a set first, since cell can have more than one entry pointing to the same parent.
    // Here, we are only interested in the unique set of parents.
    let mut seen: HashMap<&str, bool> = HashMap::new();
    let mut same_rank_parents: Vec<_> = cell
        .iter()
        .flat_map(|n| graph.get_same_ranks_parents(n))
        .collect();
    same_rank_parents.retain(|x| seen.insert(*x, true).is_none());
    let nodes = graph.get_nodes();
    let cell_x = cell.get_x(nodes);
    let child_len = children.len();
    // If node has children, center over them
    if child_len > 0 {
        if child_len == 1 && cell_x < children.get_x(nodes) && parents.is_empty() {
            // If node has no parents and exactly one child, center over it
            let child_x = children.get_x(nodes);
            let all_parent_cells = children
                .iter()
                .filter_map(|c| match graph.get_real_parents(c) {
                    // p if p.len() > 1 => Some(p.iter().map(|&p| vec![p]).collect::<Vec<_>>()),
                    p if p.len() > 1 => Some(p),
                    _ => None,
                })
                .flatten()
                .filter(|c| graph.get_real_children(c).len() == 1)
                .collect::<Vec<_>>();
            let parent_center = get_center(nodes, &all_parent_cells);
            let move_x = std::cmp::min(child_x - parent_center + cell_x, child_x);
            if move_x > cell_x { Some(move_x) } else { None }
        } else {
            // Only center over the children that have no other parents...
            let center_children = children
                .iter()
                .filter(|&c| graph.get_real_parents(c).len() == 1)
                .copied()
                .collect::<Vec<_>>();
            // ... and remember them. We don't move them later if they are already more to the right.
            if center_children.len() > 1 && cell_x <= get_center(nodes, &center_children) {
                center_children.iter().for_each(|c| {
                    moved_nodes.insert(c);
                });
            }

            // Ensure that we have moved the node with a single parent exactly under its parent.
            // This might happen if e.g. there is a large in-context node at the parent.
            // Then the child is too far left and never moved.
            let min_x = if parents.len() == 1
                && graph.get_real_children(parents.first().unwrap()).len() == 1
            {
                parents.get_x(nodes)
            } else if center_children.is_empty() && child_len > 1 {
                // If there is more than one child but only with at least one other parent,
                // make sure that the current parent are at least the same x as its parents.
                let min_cx = children
                    .iter()
                    .map(|&c| vec![c].get_x(nodes))
                    .min()
                    .unwrap_or(0);
                std::cmp::max(min_cx, cell_x)
            } else {
                0
            };
            Some(std::cmp::max(min_x, get_center(nodes, &center_children)))
        }
    } else if !parents.is_empty() {
        // else center under parents if the parent has centered above the current cell and other nodes
        if cell.iter().any(|c| moved_nodes.contains(c)) {
            None
        } else {
            let parent_center = get_center(nodes, &parents);
            // If there is just one parent, save some space if
            // there are other children of that parent (with again only one parent, so it will center over it)
            // by just moving as far as necessary to the right
            let center_children = parents
                .iter()
                .flat_map(|c| graph.get_real_children(c))
                .filter(|&c| !moved_nodes.contains(c))
                .filter(|&c| graph.get_real_parents(c).len() == 1)
                .collect::<Vec<_>>();
            if parents.len() == 1 {
                Some(std::cmp::min(
                    parent_center,
                    cell_x + parent_center - get_center(nodes, &center_children),
                ))
            } else if parents
                .iter()
                .flat_map(|c| graph.get_real_children(c))
                .filter(|c| !cell.contains(c))
                .map(|c| graph.get_real_parents(c))
                .any(|c| parents == c)
            {
                // More than one parent, but other parents share the same child nodes
                // We do not distribute them evenly beneath parents, because last parent could have
                // another independent child again.
                // We move the first one at least as far to the right as the left most parent.
                parents.iter().map(|&p| vec![p].get_x(nodes)).min()
            } else {
                // Single child beneath multiple parents => center it
                Some(parent_center)
            }
        }
    } else if same_rank_parents.len() == 1 {
        // Move same rank child closer to parent
        Some(move_closer(
            nodes,
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
