use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::dirgraph::DirectedGraph;

use super::{
    edges::EdgeType,
    nodes::Node,
    util::{font::FontInfo, point2d::Point2D},
};

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
    ranks: &Vec<Vec<Vec<&str>>>,
    margin: &Margin,
    parent_edges: &BTreeMap<&str, Vec<(&str, EdgeType)>>,
) -> (i32, i32) {
    // Generate edge map from children to parents
    let edge_map = parent_edges;
    let mut first_run = true;
    let nodes = graph.get_nodes();
    let edges = graph.get_edges();
    // This number should be safe that it yields a final, good looking image
    // let limit = nodes.len() * nodes.len() * 2;
    let limit = 5;
    for limiter in 1..=limit {
        let mut changed = false;
        let mut y = margin.top;
        for v_rank in ranks.iter() {
            let mut x = margin.left;
            let dy_max = get_max_height(nodes, margin, v_rank);
            y += dy_max / 2;
            for np in v_rank.iter() {
                let w = np.get_max_width(nodes);
                let old_x = np.get_x(nodes);
                x = std::cmp::max(x + w / 2, old_x);
                if !first_run {
                    if let Some(new_x) = has_node_to_be_moved(nodes, edges, graph, np, edge_map) {
                        if new_x > x {
                            x = std::cmp::max(x, new_x);
                            // eprintln!("Changed {:?} {} {} {}", &np, x, old_x, new_x);
                            changed = true;
                        }
                    }
                }
                // dbg!(x, y);
                // nodes.get_mut("G1").unwrap().set_position(&Point2D { x, y });
                np.set_position(nodes, margin, Point2D { x, y });
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
/// 
fn has_node_to_be_moved(
    nodes: &BTreeMap<String, RefCell<Node>>,
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
    graph: &DirectedGraph<'_, RefCell<Node>, EdgeType>,
    cell: &[&str],
    edge_map: &BTreeMap<&str, Vec<(&str, EdgeType)>>,
) -> Option<i32> {
    let children: Vec<_> = cell
        .iter()
        .flat_map(|n| graph.get_real_children(n))
        .collect();
    let parents: Vec<_> = cell
        .iter()
        .filter_map(|n| edge_map.get(n))
        .flatten() // TODO Filter edge type?
        .map(|(p, _)| *p)
        .collect();
    if !children.is_empty() {
        Some(get_center(nodes, &children))
    } else if !parents.is_empty() {
        Some(get_center(nodes, &parents))
    } else {
        None
    }

    // if let Some(x_new) = should_in_context_node_move(np, edge_map) {
    //     Some(x_new)
    // } else if let Some(x_new) = should_parent_move(np, edge_map) {
    //     Some(x_new)
    // } else {
    //     should_child_move(np, edge_map)
    // }
}

// ///
// ///
// ///
// ///
// ///
// fn should_in_context_node_move(
//     &self,
//     np: &NodePlace,
//     edge_map: &BTreeMap<String, Vec<(String, EdgeType)>>,
// ) -> Option<i32> {
//     match np {
//         NodePlace::Node(current_node) => {
//             let parent = edge_map
//                 .get(current_node)
//                 .into_iter()
//                 .flatten()
//                 .filter(|(_, ct)| {
//                     matches!(
//                         ct,
//                         EdgeType::OneWay(SingleEdge::InContextOf)
//                             | EdgeType::TwoWay((_, SingleEdge::InContextOf))
//                             | EdgeType::OneWay(SingleEdge::Composite)
//                             | EdgeType::TwoWay((_, SingleEdge::Composite))
//                     )
//                 })
//                 .map(|(n, _)| n)
//                 .last();
//             let current_x = self.nodes.get(current_node).unwrap().get_position().x;
//             match parent.map(|p| self.nodes.get(p).unwrap()) {
//                 Some(n) if n.get_position().x > current_x => Some(
//                     n.get_position().x
//                         - n.get_width() / 2
//                         - self.margin.left
//                         - self.margin.right
//                         - self.nodes.get(current_node).unwrap().get_width() / 2,
//                 ),
//                 Some(_) => None, // Nodes to the right will automatically be shifted
//                 None => None,
//             }
//         }
//         NodePlace::MultipleNodes(current_nodes) => {
//             // Currently, it is only possible that inContext nodes with the same parent end up in
//             // in a MultipleNodes node place. Thus, it is sufficient to check for the parent of
//             // the first contained node.
//             let parent = edge_map
//                 .get(current_nodes.first().unwrap())
//                 .into_iter()
//                 .flatten()
//                 .filter(|(_, ct)| {
//                     matches!(
//                         ct,
//                         EdgeType::OneWay(SingleEdge::InContextOf)
//                             | EdgeType::TwoWay((_, SingleEdge::InContextOf))
//                             | EdgeType::OneWay(SingleEdge::Composite)
//                             | EdgeType::TwoWay((_, SingleEdge::Composite))
//                     )
//                 })
//                 .map(|(n, _)| n)
//                 .last();
//             let current_x = np.get_x(&self.nodes);
//             match parent.map(|p| self.nodes.get(p).unwrap()) {
//                 Some(n) if n.get_position().x > current_x => Some(
//                     n.get_position().x
//                         - n.get_width() / 2
//                         - self.margin.left
//                         - self.margin.right
//                         - np.get_max_width(&self.nodes) / 2,
//                 ),
//                 Some(_) => None, // Nodes to the right will automatically be shifted
//                 None => None,
//             }
//         }
//     }
// }

// ///
// /// Check if a child node should be moved to the right.
// ///
// /// If the current node has more than one parent, center the current node.
// /// If the current node has exactly one parent, move it directly beneath its parent.
// /// This is exactly the same as centering to the parent nodes.
// /// Move children that don't have own children.
// ///
// ///
// /// Only inContext nodes can be MultipleNodes, thus, we don't need to think about them here.
// ///
// ///
// fn should_child_move(
//     &self,
//     node_place: &NodePlace,
//     edge_map: &BTreeMap<String, Vec<(String, EdgeType)>>,
// ) -> Option<i32> {
//     match node_place {
//         NodePlace::Node(current_node) => {
//             // Collect all nodes pointing to current_node
//             let parents = edge_map
//                 .get(current_node)
//                 .into_iter()
//                 .flatten()
//                 .filter(|(_, et)| {
//                     matches!(
//                         et,
//                         EdgeType::OneWay(SingleEdge::SupportedBy)
//                             | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
//                             | EdgeType::OneWay(SingleEdge::Composite)
//                             | EdgeType::TwoWay((_, SingleEdge::Composite))
//                     )
//                 })
//                 .map(|(p, _)| p)
//                 .collect::<Vec<&String>>();
//             // Collect all child nodes of the current_node
//             let children = self
//                 .edges
//                 .get(current_node)
//                 .into_iter()
//                 .flatten()
//                 .filter(|(_, et)| {
//                     matches!(
//                         et,
//                         EdgeType::OneWay(SingleEdge::SupportedBy)
//                             | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
//                             | EdgeType::OneWay(SingleEdge::Composite)
//                             | EdgeType::TwoWay((_, SingleEdge::Composite))
//                     )
//                 })
//                 .count();
//             // Collect all nodes that are pointed to by the parents of current_node
//             let parents_max_children = parents
//                 .iter()
//                 .map(|&p| {
//                     self.edges
//                         .get(p)
//                         .into_iter()
//                         .flatten()
//                         .filter(|(_, et)| {
//                             matches!(
//                                 et,
//                                 EdgeType::OneWay(SingleEdge::SupportedBy)
//                                     | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
//                                     | EdgeType::OneWay(SingleEdge::Composite)
//                                     | EdgeType::TwoWay((_, SingleEdge::Composite))
//                             )
//                         })
//                         .count()
//                 })
//                 .max()
//                 .unwrap_or(0);

//             let parents_children = parents
//                 .iter()
//                 .flat_map(|&p| {
//                     self.edges
//                         .get(p)
//                         .into_iter()
//                         .flatten()
//                         .filter(|(_, et)| {
//                             matches!(
//                                 et,
//                                 EdgeType::OneWay(SingleEdge::SupportedBy)
//                                     | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
//                                     | EdgeType::OneWay(SingleEdge::Composite)
//                                     | EdgeType::TwoWay((_, SingleEdge::Composite))
//                             )
//                         })
//                         .map(|(c, _)| c)
//                 })
//                 .collect::<Vec<&String>>();
//             // Do the children of the current node's parents have other nodes as parents?
//             // true means they have other parents
//             // let parents_children_parents = parents_children
//             //     .iter()
//             //     .flat_map(|&c| {
//             //         edge_map
//             //             .get(c)
//             //             .into_iter()
//             //             .flatten()
//             //             .filter(|(_, et)| {
//             //                 matches!(
//             //                     et,
//             //                     EdgeType::OneWay(SingleEdge::SupportedBy)
//             //                         | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
//             //                         | EdgeType::OneWay(SingleEdge::Composite)
//             //                         | EdgeType::TwoWay((_, SingleEdge::Composite))
//             //                 )
//             //             })
//             //             .map(|(p, _)| p)
//             //     })
//             //     .any(|p| !parents.contains(&p));

//             // Get the parents minimum position and all the children of those parents maximum
//             let parents_min_x = parents
//                 .iter()
//                 .map(|&p| self.nodes.get(p).unwrap().get_position().x)
//                 .min()
//                 .unwrap_or(0);
//             let child_xs = parents_children
//                 .iter()
//                 .map(|&p| self.nodes.get(p).unwrap().get_position().x)
//                 .collect::<Vec<i32>>();
//             let child_x_max = *child_xs.iter().max().unwrap_or(&0);

//             // Move child if there are equal or more parents than children of the current node's parents
//             // Or move child if parent has children that have other parents
//             // (In other words: Don't move child if parent only has children that don't have other parents)
//             // Of if parent is already move so far to the right that all children are more to the left than their parents
//             if parents.len() >= parents_max_children
//                     // || (children == 0 && parents_children_parents)
//                     || (children == 0 && parents_min_x > child_x_max)
//             {
//                 let mm: Vec<i32> = parents
//                     .iter()
//                     .map(|&parent| self.nodes.get(parent).unwrap().get_position().x)
//                     .collect();
//                 if mm.is_empty() {
//                     // Can happen in rare theoretical, minimal cases.
//                     None
//                 } else {
//                     let min = *mm.iter().min().unwrap();
//                     let max = *mm.iter().max().unwrap();
//                     // eprintln!("Child {} of nodes {} should move to {}", current_node, parents.iter().map(|(a,_)| a.as_str()).collect::<Vec<&str>>().join(","), (min+max)/2);
//                     Some((min + max) / 2)
//                 }
//             } else {
//                 None
//             }
//         }
//         NodePlace::MultipleNodes(_) => None,
//     }
// }

// ///
// /// Check if a parent node should be moved.
// /// Since we start moving nodes to the right from the top, we need to consider the 1:1 case here.
// /// However, especially Solutions that don't have children, but are only child nodes, have to
// /// be considered in `should_child_move`.
// ///
// /// If node has children center them over all nodes that only have this one as parent.
// ///
// /// We only need to consider single nodes (NodePlace::Node), because
// /// MultipleNode cannot be supportedBy nodes. They are only inContext nodes.
// ///
// ///
// fn should_parent_move(
//     &self,
//     node_place: &NodePlace,
//     edge_map: &BTreeMap<String, Vec<(String, EdgeType)>>,
// ) -> Option<i32> {
//     match node_place {
//         NodePlace::Node(current_node) => {
//             // Collect all supportedBy children
//             let supby_children = self
//                 .edges
//                 .get(current_node)
//                 .into_iter()
//                 .flatten()
//                 // Filter children that have more than one parent
//                 .filter(|(c, _)| {
//                     edge_map
//                         .get(c)
//                         .unwrap()
//                         .iter()
//                         .filter(|(_, et)| {
//                             matches!(
//                                 et,
//                                 EdgeType::OneWay(SingleEdge::SupportedBy)
//                                     | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
//                                     | EdgeType::OneWay(SingleEdge::Composite)
//                                     | EdgeType::TwoWay((_, SingleEdge::Composite))
//                             )
//                         })
//                         .count()
//                         == 1
//                 })
//                 .filter_map(|(c, et)| match et {
//                     EdgeType::OneWay(SingleEdge::SupportedBy)
//                     | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
//                     | EdgeType::OneWay(SingleEdge::Composite)
//                     | EdgeType::TwoWay((_, SingleEdge::Composite)) => Some(c.as_str()),
//                     _ => None,
//                 })
//                 // Map to their current x position
//                 .map(|child| self.nodes.get(child).unwrap().get_position().x)
//                 .collect::<Vec<i32>>();

//             if supby_children.is_empty() {
//                 None // Node is actually not a parent and, thus, should not be moved here
//             } else {
//                 let min = supby_children.iter().min().unwrap();
//                 let max = supby_children.iter().max().unwrap();
//                 // eprintln!("Parent {} should move to {}", current_node, (min+max)/2);
//                 Some((min + max) / 2)
//             }
//         }
//         NodePlace::MultipleNodes(_) => None,
//     }
// }

#[cfg(test)]
mod test {}
