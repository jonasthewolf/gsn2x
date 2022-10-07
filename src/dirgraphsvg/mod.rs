pub mod edges;
mod graph;
pub mod nodes;
mod util;
use anyhow::Context;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
pub use util::{escape_node_id, escape_text};

use edges::{EdgeType, SingleEdge};
use graph::{rank_nodes, NodePlace};
use nodes::{setup_basics, Node, Port};
use svg::{
    node::element::{path::Data, Marker, Path, Polyline, Rectangle, Style, Symbol, Text, Title},
    Document,
};
use util::point2d::Point2D;

use self::{graph::calculate_parent_edge_map, util::font::FontInfo};

const MARKER_HEIGHT: u32 = 10;

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

pub struct DirGraph<'a> {
    width: i32,
    height: i32,
    margin: Margin,
    wrap: u32,
    font: FontInfo,
    css_stylesheets: Vec<&'a str>,
    embed_stylesheets: bool,
    forced_levels: BTreeMap<&'a str, Vec<&'a str>>,
    nodes: BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: BTreeMap<String, Vec<(String, EdgeType)>>,
    document: Document,
    meta_information: Option<Vec<String>>,
}

impl<'a> Default for DirGraph<'a> {
    fn default() -> Self {
        Self {
            width: 210,
            height: 297,
            margin: Margin::default(),
            wrap: 40,
            font: FontInfo::default(),
            css_stylesheets: Vec::new(),
            embed_stylesheets: false,
            forced_levels: BTreeMap::new(),
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
            document: Document::new(),
            meta_information: None,
        }
    }
}

impl<'a> DirGraph<'a> {
    pub fn _set_wrap(mut self, wrap: u32) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn _set_margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    pub fn _add_css_stylesheet(mut self, css: &'a str) -> Self {
        self.css_stylesheets.push(css);
        self
    }

    pub fn add_css_stylesheets(mut self, css: &mut Vec<&'a str>) -> Self {
        self.css_stylesheets.append(css);
        self
    }

    pub fn embed_stylesheets(mut self, embed: bool) -> Self {
        self.embed_stylesheets = embed;
        self
    }

    pub fn add_nodes(mut self, mut nodes: BTreeMap<String, Rc<RefCell<dyn Node>>>) -> Self {
        self.nodes.append(&mut nodes);
        self
    }

    pub fn _add_node(mut self, node: Rc<RefCell<dyn Node>>) -> Self {
        self.nodes
            .insert(node.borrow().get_id().to_owned(), node.clone());
        self
    }

    pub fn _add_edge(
        mut self,
        source: Rc<RefCell<dyn Node>>,
        target: Rc<RefCell<dyn Node>>,
        edge_type: EdgeType,
    ) -> Self {
        let entry = self
            .edges
            .entry(source.borrow().get_id().to_owned())
            .or_default();
        entry.push((target.borrow().get_id().to_owned(), edge_type));
        self
    }

    pub fn add_edges(mut self, edges: &mut BTreeMap<String, Vec<(String, EdgeType)>>) -> Self {
        self.edges.append(edges);
        self
    }

    pub fn add_levels(mut self, levels: &BTreeMap<&'a str, Vec<&'a str>>) -> Self {
        for (level, nodes) in levels {
            self.forced_levels.insert(level, nodes.to_vec());
        }
        self
    }

    pub fn add_meta_information(mut self, meta: &mut Vec<String>) -> Self {
        self.meta_information.get_or_insert(Vec::new()).append(meta);
        self
    }

    pub fn write(
        mut self,
        mut output: impl std::io::Write,
        cycles_allowed: bool,
    ) -> Result<(), std::io::Error> {
        self = self.setup_basics();
        self = self.setup_stylesheets();
        self = self.layout(cycles_allowed);
        // Order is important here. render_legend may modify self.width and self.height
        self = self.render_legend();
        self.document = self
            .document
            .set("viewBox", (0u32, 0u32, self.width, self.height));
        output.write_all("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".as_bytes())?;
        svg::write(output, &self.document)?;
        Ok(())
    }

    ///
    /// Layout the graph on a pseudo-stack layout
    ///
    /// 1) Let each element calculate its size
    /// 2) Calculate forced levels
    /// 3) Rank the nodes
    /// 4) Position nodes initially
    /// 5) Center nodes and draw them
    /// 6) Draw the edges
    ///
    ///
    ///
    fn layout(mut self, cycles_allowed: bool) -> Self {
        // Calculate node sizes
        self.nodes
            .values()
            .for_each(|n| n.borrow_mut().calculate_size(&self.font, self.wrap));

        // Rank nodes
        let ranks = rank_nodes(
            &mut self.nodes,
            &mut self.edges,
            &self.forced_levels,
            cycles_allowed,
        );

        // Draw nodes
        self = self.render_nodes(&ranks);

        // Draw edges
        self.render_edges()
    }

    ///
    ///
    ///
    fn render_nodes(mut self, ranks: &BTreeMap<usize, BTreeMap<usize, NodePlace>>) -> Self {
        // Generate edge map from children to parents
        let edge_map = calculate_parent_edge_map(&self.edges);
        // Iteratively move nodes horizontally until no movement detected
        let mut first_run = true;

        const LIMIT: i32 = 150; // Arbitrary value
        for limiter in 1..=LIMIT {
            let mut changed = false;
            let mut y = self.margin.top;
            for v_rank in ranks.values() {
                let mut x = self.margin.left;
                let dy_max = self.get_max_height(v_rank);
                y += dy_max / 2;
                for np in v_rank.values() {
                    let w = np.get_max_width(&self.nodes);
                    let old_x = np.get_x(&self.nodes);
                    x = std::cmp::max(x + w / 2, old_x);
                    if !first_run {
                        if let Some(new_x) = self.has_node_to_be_moved(np, &edge_map) {
                            if new_x > x {
                                x = std::cmp::max(x, new_x);
                                // eprintln!("Changed {:?} {} {} {}", &np, x, old_x, new_x);
                                changed = true;
                            }
                        }
                    }
                    np.set_position(&self.nodes, &self.margin, Point2D { x, y });
                    x += w / 2 + self.margin.left + self.margin.right;
                }
                y += self.margin.bottom + dy_max / 2 + self.margin.top;
            }
            if !(first_run || changed) {
                break;
            }
            first_run = false;
            if changed && limiter == LIMIT {
                eprintln!("Rendering a diagram took too many iterations ({}). See README.md for hints how to solve this situation.", limiter);
            }
        }

        // Draw the nodes
        for rank in ranks.values() {
            for np in rank.values() {
                match np {
                    NodePlace::Node(id) => {
                        let mut n = self.nodes.get(id).unwrap().borrow_mut();
                        self.document = self.document.add(n.render(&self.font));
                    }
                    NodePlace::MultipleNodes(ids) => {
                        for id in ids {
                            let mut n = self.nodes.get(id).unwrap().borrow_mut();
                            self.document = self.document.add(n.render(&self.font));
                        }
                    }
                }
            }
        }
        // Calculate size of document
        self.width = ranks
            .values()
            .map(|rank| {
                let n = rank.values().last().unwrap();
                n.get_x(&self.nodes) + n.get_max_width(&self.nodes)
            })
            .max()
            .unwrap_or(0);
        self.height = ranks
            .values()
            .map(|rank| self.margin.top + self.get_max_height(rank) + self.margin.bottom)
            .sum();
        self
    }

    ///
    ///
    ///
    ///
    ///
    fn has_node_to_be_moved(
        &self,
        np: &NodePlace,
        edge_map: &BTreeMap<String, Vec<(String, EdgeType)>>,
    ) -> Option<i32> {
        if let Some(x_new) = self.should_in_context_node_move(np, edge_map) {
            Some(x_new)
        } else if let Some(x_new) = self.should_parent_move(np, edge_map) {
            Some(x_new)
        } else {
            self.should_child_move(np, edge_map)
        }
    }

    ///
    ///
    ///
    ///
    ///
    fn should_in_context_node_move(
        &self,
        np: &NodePlace,
        edge_map: &BTreeMap<String, Vec<(String, EdgeType)>>,
    ) -> Option<i32> {
        match np {
            NodePlace::Node(current_node) => {
                let parent = edge_map
                    .get(current_node)
                    .into_iter()
                    .flatten()
                    .filter(|(_, ct)| {
                        matches!(
                            ct,
                            EdgeType::OneWay(SingleEdge::InContextOf)
                                | EdgeType::TwoWay((_, SingleEdge::InContextOf))
                                | EdgeType::OneWay(SingleEdge::Composite)
                                | EdgeType::TwoWay((_, SingleEdge::Composite))
                        )
                    })
                    .map(|(n, _)| n)
                    .last();
                let current_x = self
                    .nodes
                    .get(current_node)
                    .unwrap()
                    .borrow()
                    .get_position()
                    .x;
                match parent.map(|p| self.nodes.get(p).unwrap().borrow()) {
                    Some(n) if n.get_position().x > current_x => Some(
                        n.get_position().x
                            - n.get_width() / 2
                            - self.margin.left
                            - self.margin.right
                            - self.nodes.get(current_node).unwrap().borrow().get_width() / 2,
                    ),
                    Some(_) => None, // Nodes to the right will automatically be shifted
                    None => None,
                }
            }
            NodePlace::MultipleNodes(current_nodes) => {
                // Currently, it is only possible that inContext nodes with the same parent end up in
                // in a MultipleNodes node place. Thus, it is sufficient to check for the parent of
                // the first contained node.
                let parent = edge_map
                    .get(current_nodes.first().unwrap())
                    .into_iter()
                    .flatten()
                    .filter(|(_, ct)| {
                        matches!(
                            ct,
                            EdgeType::OneWay(SingleEdge::InContextOf)
                                | EdgeType::TwoWay((_, SingleEdge::InContextOf))
                                | EdgeType::OneWay(SingleEdge::Composite)
                                | EdgeType::TwoWay((_, SingleEdge::Composite))
                        )
                    })
                    .map(|(n, _)| n)
                    .last();
                let current_x = np.get_x(&self.nodes);
                match parent.map(|p| self.nodes.get(p).unwrap().borrow()) {
                    Some(n) if n.get_position().x > current_x => Some(
                        n.get_position().x
                            - n.get_width() / 2
                            - self.margin.left
                            - self.margin.right
                            - np.get_max_width(&self.nodes) / 2,
                    ),
                    Some(_) => None, // Nodes to the right will automatically be shifted
                    None => None,
                }
            }
        }
    }

    ///
    ///
    ///
    ///
    fn should_child_move(
        &self,
        node_place: &NodePlace,
        edge_map: &BTreeMap<String, Vec<(String, EdgeType)>>,
    ) -> Option<i32> {
        match node_place {
            NodePlace::Node(current_node) => {
                // Collect all nodes pointing to current_node
                let parents: Vec<&(String, EdgeType)> = edge_map
                    .get(current_node)
                    .iter()
                    .cloned()
                    .flatten()
                    .filter(|(_, et)| {
                        matches!(
                            et,
                            EdgeType::OneWay(SingleEdge::SupportedBy)
                                | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                                | EdgeType::OneWay(SingleEdge::Composite)
                                | EdgeType::TwoWay((_, SingleEdge::Composite))
                        )
                    })
                    .collect();
                // Collect all nodes that are pointed to by the parents of current_node
                let parents_children = parents
                    .iter()
                    .map(|&(c, _)| {
                        self.edges
                            .get(c)
                            .unwrap()
                            .iter()
                            .filter(|(_, et)| {
                                matches!(
                                    et,
                                    EdgeType::OneWay(SingleEdge::SupportedBy)
                                        | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                                        | EdgeType::OneWay(SingleEdge::Composite)
                                        | EdgeType::TwoWay((_, SingleEdge::Composite))
                                )
                            })
                            .count()
                    })
                    .max()
                    .unwrap_or(0);
                if parents.len() < parents_children {
                    None
                } else {
                    let mm: Vec<i32> = parents
                        .iter()
                        .map(|&(parent, _)| {
                            self.nodes.get(parent).unwrap().borrow().get_position().x
                        })
                        .collect();
                    if mm.is_empty() {
                        // Can happen in rare theoretical, minimal cases.
                        None
                    } else {
                        let min = *mm.iter().min().unwrap();
                        let max = *mm.iter().max().unwrap();
                        // eprintln!("Child {} of nodes {} should move to {}", current_node, parents.iter().map(|(a,_)| a.as_str()).collect::<Vec<&str>>().join(","), (min+max)/2);
                        Some((min + max) / 2)
                    }
                }
            }
            NodePlace::MultipleNodes(_) => None,
        }
    }

    ///
    /// There are two cases:
    /// 1) 1:1 => The parent (current_node) has exactly one child.
    ///           This child has exactly current_node as its own parent.
    ///           Move the parent if it is further to the left than its child.
    /// 2) 1:n => The parent (current_node) has multiple children.
    ///           It has to have more children than each child parents to be moved.
    ///           Move the parent to the center of all (supportedBy) child nodes.
    ///
    fn should_parent_move(
        &self,
        node_place: &NodePlace,
        edge_map: &BTreeMap<String, Vec<(String, EdgeType)>>,
    ) -> Option<i32> {
        match node_place {
            NodePlace::Node(current_node) => {
                // Collect all children
                let cur_edges: Vec<&(String, EdgeType)> = self
                    .edges
                    .get(current_node)
                    .iter()
                    .cloned()
                    .flatten()
                    .collect();
                // Filter them for supportedBy nodes
                let supby_children = cur_edges
                    .iter()
                    .filter_map(|(c, et)| match et {
                        EdgeType::OneWay(SingleEdge::SupportedBy)
                        | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                        | EdgeType::OneWay(SingleEdge::Composite)
                        | EdgeType::TwoWay((_, SingleEdge::Composite)) => Some(c.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<&str>>();
                match supby_children.len() {
                    0 => None, // Node is actually not a parent and, thus, should not be moved here
                    1 => {
                        // Exactly one child
                        let child = *supby_children.first().unwrap();
                        let child_num_parents = edge_map
                            .get(child)
                            .unwrap()
                            .iter()
                            .filter(|(_, ct)| {
                                matches!(
                                    ct,
                                    EdgeType::OneWay(SingleEdge::SupportedBy)
                                        | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                                        | EdgeType::OneWay(SingleEdge::Composite)
                                        | EdgeType::TwoWay((_, SingleEdge::Composite))
                                )
                            })
                            .count();
                        // let x_parent = self
                        //     .nodes
                        //     .get(current_node)
                        //     .unwrap()
                        //     .borrow()
                        //     .get_position()
                        //     .x;
                        let x_child = self.nodes.get(child).unwrap().borrow().get_position().x;
                        if child_num_parents > 1 {
                            None
                        } else {
                            // eprintln!("Parent {} of single node {} should move to {}", current_node, child, x_child);
                            Some(x_child)
                        }
                    }
                    _ =>
                    // More than one child
                    {
                        let childrens_parent = supby_children
                            .iter()
                            .map(|&child| {
                                edge_map
                                    .get(child)
                                    .unwrap()
                                    .iter()
                                    .filter(|(_, ct)| {
                                        matches!(
                                            ct,
                                            EdgeType::OneWay(SingleEdge::SupportedBy)
                                                | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                                                | EdgeType::OneWay(SingleEdge::Composite)
                                                | EdgeType::TwoWay((_, SingleEdge::Composite))
                                        )
                                    })
                                    .count()
                            })
                            .max()
                            .unwrap();
                        if childrens_parent > supby_children.len() {
                            None
                        } else {
                            let mm: Vec<i32> = supby_children
                                .iter()
                                .map(|&child| {
                                    self.nodes.get(child).unwrap().borrow().get_position().x
                                })
                                .collect();
                            if mm.is_empty() {
                                None
                            } else {
                                let min = mm.iter().min().unwrap();
                                let max = mm.iter().max().unwrap();
                                // eprintln!("Parent {} of nodes {} should move to {}", current_node, supby_children.join(","), (min+max)/2);
                                Some((min + max) / 2)
                            }
                        }
                    }
                }
            }
            NodePlace::MultipleNodes(_) => None, // MultipleNode cannot be supportedBy nodes
        }
    }

    ///
    /// Get the maximum height of a rank
    ///
    ///
    fn get_max_height(&self, rank: &BTreeMap<usize, NodePlace>) -> i32 {
        rank.values()
            .map(|id| match id {
                NodePlace::Node(id) => self.nodes.get(id).unwrap().borrow().get_height(),
                NodePlace::MultipleNodes(ids) => {
                    ids.iter()
                        .map(|id| self.nodes.get(id).unwrap().borrow().get_height())
                        .sum::<i32>()
                        + (self.margin.top + self.margin.bottom) * (ids.len() - 1) as i32
                }
            })
            .max()
            .unwrap()
    }

    ///
    /// Render the edges
    ///
    /// TODO Make edges nicer, if e.g., start marker is used. Make the first and last MARKER_HEIGHT pixels vertical.
    ///
    ///
    fn render_edges(mut self) -> Self {
        for (source, targets) in &self.edges {
            for (target, edge_type) in targets {
                let s = self.nodes.get(source).unwrap().borrow();
                let t = self.nodes.get(target).unwrap().borrow();
                let (marker_start_height, marker_end_height, support_distance) = match edge_type {
                    // EdgeType::Invisible => (0i32, 0i32, 3i32 * MARKER_HEIGHT as i32),
                    EdgeType::OneWay(_) => {
                        (0i32, MARKER_HEIGHT as i32, 3i32 * MARKER_HEIGHT as i32)
                    }
                    EdgeType::TwoWay(_) => (
                        MARKER_HEIGHT as i32,
                        MARKER_HEIGHT as i32,
                        3i32 * MARKER_HEIGHT as i32,
                    ),
                };
                let s_pos = s.get_position();
                let t_pos = t.get_position();
                let (start, start_sup, end, end_sup) =
                    if s_pos.y + s.get_height() / 2 < t_pos.y - t.get_height() / 2 {
                        (
                            s.get_coordinates(&Port::South)
                                .move_relative(0, marker_start_height),
                            s.get_coordinates(&Port::South)
                                .move_relative(0, support_distance),
                            t.get_coordinates(&Port::North)
                                .move_relative(0, -marker_end_height),
                            t.get_coordinates(&Port::North)
                                .move_relative(0, -support_distance),
                        )
                    } else if s_pos.y - s.get_height() / 2 - self.margin.top
                        > t_pos.y + t.get_height() / 2
                    {
                        (
                            s.get_coordinates(&Port::North)
                                .move_relative(0, -marker_start_height),
                            s.get_coordinates(&Port::North)
                                .move_relative(0, -support_distance),
                            t.get_coordinates(&Port::South)
                                .move_relative(0, marker_end_height),
                            t.get_coordinates(&Port::South)
                                .move_relative(0, support_distance),
                        )
                    } else if s_pos.x - s.get_width() / 2 > t_pos.x + t.get_width() / 2 {
                        (
                            s.get_coordinates(&Port::West)
                                .move_relative(-marker_start_height, 0),
                            s.get_coordinates(&Port::West),
                            t.get_coordinates(&Port::East)
                                .move_relative(marker_end_height, 0),
                            t.get_coordinates(&Port::East)
                                .move_relative(support_distance, 0),
                        )
                    } else {
                        (
                            s.get_coordinates(&Port::East)
                                .move_relative(marker_start_height, 0),
                            s.get_coordinates(&Port::East),
                            t.get_coordinates(&Port::West)
                                .move_relative(-marker_end_height, 0),
                            t.get_coordinates(&Port::West)
                                .move_relative(-support_distance, 0),
                        )
                    };
                let parameters = (start_sup.x, start_sup.y, end_sup.x, end_sup.y, end.x, end.y);
                let data = Data::new()
                    .move_to((start.x, start.y))
                    .cubic_curve_to(parameters);
                let arrow_end_id = match &edge_type {
                    EdgeType::OneWay(SingleEdge::InContextOf)
                    | EdgeType::TwoWay((_, SingleEdge::InContextOf)) => {
                        Some("url(#incontextof_arrow)")
                    }
                    EdgeType::OneWay(SingleEdge::SupportedBy)
                    | EdgeType::TwoWay((_, SingleEdge::SupportedBy)) => {
                        Some("url(#supportedby_arrow)")
                    }
                    EdgeType::OneWay(SingleEdge::Composite)
                    | EdgeType::TwoWay((_, SingleEdge::Composite)) => Some("url(#composite_arrow)"),
                    // EdgeType::Invisible => None,
                };
                let arrow_start_id = match &edge_type {
                    EdgeType::TwoWay((SingleEdge::InContextOf, _)) => {
                        Some("url(#incontextof_arrow)")
                    }
                    EdgeType::TwoWay((SingleEdge::SupportedBy, _)) => {
                        Some("url(#supportedby_arrow)")
                    }
                    EdgeType::TwoWay((SingleEdge::Composite, _)) => Some("url(#composite_arrow)"),
                    _ => None,
                };
                let mut classes = "gsnedge".to_string();
                match edge_type {
                    EdgeType::OneWay(SingleEdge::InContextOf)
                    | EdgeType::TwoWay((_, SingleEdge::InContextOf))
                    | EdgeType::TwoWay((SingleEdge::InContextOf, _)) => {
                        classes.push_str(" gsninctxt")
                    }
                    EdgeType::OneWay(SingleEdge::SupportedBy)
                    | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                    | EdgeType::TwoWay((SingleEdge::SupportedBy, _)) => {
                        classes.push_str(" gsninspby")
                    }
                    EdgeType::OneWay(SingleEdge::Composite)
                    | EdgeType::TwoWay((_, SingleEdge::Composite)) => {
                        // Already covered by all other matches
                        //| EdgeType::TwoWay((SingleEdge::Composite, _))
                        classes.push_str(" gsncomposite")
                    } // EdgeType::Invisible => classes.push_str(" gsninvis"),
                };
                let mut e = Path::new()
                    .set("d", data)
                    .set("fill", "none")
                    .set("stroke", "black")
                    .set("stroke-width", 1u32);
                if let Some(arrow_id) = arrow_end_id {
                    e = e.set("marker-end", arrow_id);
                }
                if let Some(arrow_id) = arrow_start_id {
                    e = e.set("marker-start", arrow_id);
                }
                e = e.set("class", classes);
                self.document = self.document.add(e);
            }
        }
        self
    }

    ///
    ///
    ///
    ///
    ///
    fn setup_basics(mut self) -> Self {
        let supportedby_polyline = Polyline::new()
            .set("points", "0 0, 10 4.5, 0 9")
            .set("fill", "black");
        let supportedby_arrow = Marker::new()
            .set("id", "supportedby_arrow")
            .set("markerWidth", 10u32)
            .set("markerHeight", 9u32)
            .set("refX", 0f32)
            .set("refY", 4.5f32)
            .set("orient", "auto-start-reverse")
            .set("markerUnits", "userSpaceOnUse")
            .add(supportedby_polyline);

        let incontext_polyline = Polyline::new()
            .set("points", "0 0, 10 4.5, 0 9, 0 0")
            .set("stroke", "black")
            .set("stroke-width", 1u32)
            .set("fill", "none");
        let incontext_arrow = Marker::new()
            .set("id", "incontextof_arrow")
            .set("markerWidth", 10u32)
            .set("markerHeight", 9u32)
            .set("refX", 0f32)
            .set("refY", 4.5f32)
            .set("orient", "auto-start-reverse")
            .set("markerUnits", "userSpaceOnUse")
            .add(incontext_polyline);

        let composite_polyline1 = Polyline::new()
            .set("points", "0 0, 6 4.5, 0 9")
            .set("stroke", "black")
            .set("stroke-width", 1u32)
            .set("fill", "none");
        let composite_polyline2 = Polyline::new()
            .set("points", "4 0, 10 4.5, 4 9")
            .set("stroke", "black")
            .set("stroke-width", 1u32)
            .set("fill", "none");
        let composite_polyline3 = Polyline::new()
            .set("points", "0 4.5, 10 4.5")
            .set("stroke", "black")
            .set("stroke-width", 1u32)
            .set("fill", "none");
        let composite_arrow = Marker::new()
            .set("id", "composite_arrow")
            .set("markerWidth", 10u32)
            .set("markerHeight", 9u32)
            .set("refX", 0f32)
            .set("refY", 4.5f32)
            .set("orient", "auto-start-reverse")
            .set("markerUnits", "userSpaceOnUse")
            .add(composite_polyline1)
            .add(composite_polyline2)
            .add(composite_polyline3);

        let mi_r1 = Rectangle::new()
            .set("x", 0u32)
            .set("y", 0u32)
            .set("width", 10u32)
            .set("height", 5u32)
            .set("stroke", "black")
            .set("stroke-width", 1u32)
            .set("fill", "lightgrey");
        let mi_r2 = Rectangle::new()
            .set("x", 0u32)
            .set("y", 5u32)
            .set("width", 20u32)
            .set("height", 10u32)
            .set("stroke", "black")
            .set("stroke-width", 1u32)
            .set("fill", "lightgrey");
        let module_image = Symbol::new().set("id", "module_icon").add(mi_r1).add(mi_r2);
        self.document = self.document.add(module_image);

        self.document = self
            .document
            .add(composite_arrow)
            .add(supportedby_arrow)
            .add(incontext_arrow);
        self
    }

    ///
    ///
    ///
    ///
    fn setup_stylesheets(mut self) -> Self {
        if !self.css_stylesheets.is_empty() {
            if self.embed_stylesheets {
                for css in &self.css_stylesheets {
                    let css_str = std::fs::read_to_string(css)
                        .context(format!("Failed to open CSS file {} for embedding", css))
                        .unwrap();
                    let style =
                        Style::new(format!("<![CDATA[{}]]>", css_str)).set("type", "text/css");
                    self.document = self.document.add(style);
                }
            } else {
                // Only link them
                let style = Style::new(
                    self.css_stylesheets
                        .iter()
                        .map(|x| format!("@import (\"{x}\")"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
                self.document = self.document.add(style);
            }
        }
        self
    }

    ///
    ///
    ///
    ///
    fn render_legend(mut self) -> Self {
        if let Some(meta) = &self.meta_information {
            let mut g = setup_basics("gsn_module", &["gsnmodule".to_owned()], &None);
            let title = Title::new().add(svg::node::Text::new("Module Information"));
            use svg::Node;
            g.append(title);

            let mut text_height = 0;
            let mut text_width = 0;
            let mut lines = Vec::new();
            for t in meta {
                let (width, height) =
                    crate::dirgraphsvg::util::font::text_bounding_box(&self.font, t, false);
                lines.push((width, height));
                text_height += height;
                text_width = std::cmp::max(text_width, width);
            }

            if self.width < text_width + 20i32 {
                self.width = text_width + 40i32;
            }
            self.height += text_height + 40i32;
            let x = self.width - text_width - 20;
            let y_base = self.height - text_height - 20;
            let mut y_running = 0;
            for (t, (w, h)) in meta.iter().zip(lines) {
                y_running += h;
                let text = Text::new()
                    .set("x", x)
                    .set("y", y_base + y_running)
                    .set("textLength", w)
                    .set("font-size", self.font.size)
                    .set("font-family", self.font.name.as_str())
                    .add(svg::node::Text::new(t));
                g.append(text);
            }
            self.document = self.document.add(g);
        }
        self
    }
}

#[cfg(test)]
mod test {
    use super::{nodes::new_away_goal, *};

    #[test]
    fn call_unused() {
        let d = DirGraph::default();
        let b1 = new_away_goal("id", "text", "module", None, None, vec![]);
        d._add_css_stylesheet("css")
            ._add_edge(
                b1.clone(),
                new_away_goal("id2", "text", "module", None, None, vec![]),
                EdgeType::OneWay(SingleEdge::SupportedBy),
            )
            ._add_node(b1)
            ._set_margin(Margin {
                ..Default::default()
            })
            ._set_wrap(20);
    }

    #[test]
    fn test_render_legend() {
        let mut d = DirGraph::default();
        let b1 = new_away_goal("id", "text", "module", None, None, vec![]);
        let mut nodes = BTreeMap::new();
        nodes.insert("G1".to_owned(), b1 as Rc<RefCell<dyn Node>>);
        d = d.add_nodes(nodes);
        d = d.add_meta_information(&mut vec!["A1".to_owned(), "B2".to_owned()]);
        let mut string_buffer = Vec::new();
        d.write(&mut string_buffer, false).unwrap();
        println!("{}", std::str::from_utf8(string_buffer.as_slice()).unwrap());
    }
}
