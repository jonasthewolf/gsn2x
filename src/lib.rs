pub mod edges;
pub mod nodes;
mod util;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};

use edges::EdgeType;
use nodes::{Node, Port};
use rusttype::Font;
use svg::{
    node::element::{path::Data, Marker, Path, Polyline},
    Document,
};
use util::font::{get_default_font, get_font};

pub struct Margin {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Default for Margin {
    fn default() -> Self {
        Self {
            top: 20,
            right: 10,
            bottom: 20,
            left: 10,
        }
    }
}

pub struct FontInfo {
    font: Font<'static>,
    name: String,
    size: f32,
}

pub struct DirGraph {
    width: u32,
    height: u32,
    margin: Margin,
    wrap: u32,
    font: FontInfo,
    nodes: BTreeMap<String, Rc<RefCell<dyn Node>>>,
    edges: BTreeMap<String, Vec<(String, EdgeType)>>,
}

impl Default for DirGraph {
    fn default() -> Self {
        Self {
            width: 210,
            height: 297,
            margin: Margin::default(),
            wrap: 40,
            font: FontInfo {
                font: get_default_font().unwrap(),
                name: util::font::DEFAULT_FONT_FAMILY_NAME.to_owned(),
                size: 12.0,
            },
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
        }
    }
}

impl DirGraph {
    pub fn set_size(mut self, width: u32, height: u32) -> DirGraph {
        self.width = width;
        self.height = height;
        self
    }

    pub fn set_wrap(mut self, wrap: u32) -> DirGraph {
        self.wrap = wrap;
        self
    }

    pub fn set_margin(mut self, margin: Margin) -> DirGraph {
        self.margin = margin;
        self
    }

    pub fn set_font(mut self, font: &str, size: f32) -> DirGraph {
        self.font = FontInfo {
            font: get_font(font).unwrap(),
            name: font.to_owned(),
            size,
        };
        self
    }

    pub fn add_nodes(mut self, nodes: &mut BTreeMap<String, Rc<RefCell<dyn Node>>>) -> DirGraph {
        self.nodes.append(nodes);
        self
    }

    pub fn add_node(mut self, node: Rc<RefCell<dyn Node>>) -> DirGraph {
        self.nodes
            .insert(node.borrow().get_id().to_owned(), node.clone());
        self
    }

    pub fn add_edge(
        mut self,
        source: Rc<RefCell<dyn Node>>,
        target: Rc<RefCell<dyn Node>>,
        edge_type: EdgeType,
    ) -> DirGraph {
        let entry = self
            .edges
            .entry(source.borrow().get_id().to_owned())
            .or_default();
        entry.push((target.borrow().get_id().to_owned(), edge_type));
        self
    }

    pub fn write_to_file(self, file: &std::path::Path) -> Result<(), std::io::Error> {
        let mut document = Document::new().set("viewBox", (0u32, 0u32, self.width, self.height));

        document = self.setup_basics(document);
        document = self.layout(document);
        svg::save(file, &document)?;
        Ok(())
    }

    fn setup_basics(&self, mut doc: Document) -> Document {
        let supportedby_polyline = Polyline::new()
            .set("points", "0 0, 10 4.5, 0 9")
            .set("fill", "black");
        let supportedby_arrow = Marker::new()
            .set("id", "supportedby_arrow")
            .set("markerWidth", 10u32)
            .set("markerHeight", 9u32)
            .set("refX", 0f32)
            .set("refY", 4.5f32)
            .set("orient", "auto")
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
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(incontext_polyline);

        doc = doc.add(supportedby_arrow).add(incontext_arrow);
        doc
    }

    fn place_nodes(&self) -> Vec<Vec<String>> {
        let mut ranks = Vec::new();

        // Copy IDs
        let mut n_ids: BTreeSet<String> = self
            .nodes
            .keys()
            .into_iter()
            .map(|x| x.to_owned())
            .collect();
        // Find root nodes
        for t_edges in self.edges.values() {
            for (target, _) in t_edges {
                n_ids.remove(target);
            }
        }
        // Add inContextOf nodes again
        n_ids.append(&mut self.find_in_context_nodes(&n_ids));
        let mut rank = 0;
        loop {
            // Assign ranks to nodes
            let v: Vec<_> = n_ids.iter().map(|x| x.to_owned()).collect();
            for (i, id) in v.iter().enumerate() {
                let mut n = self.nodes.get(id).unwrap().borrow_mut();
                n.set_horizontal_rank(i);
                n.set_vertical_rank(rank);
            }
            ranks.insert(rank, v);
            // Find children
            n_ids = self.find_child_nodes(&n_ids);
            if n_ids.is_empty() {
                break;
            }
            rank += 1;
        }
        ranks
    }

    fn count_crossings_same_rank(&self, rank_nodes: Vec<String>) -> usize {
        let mut sum = 0usize;
        for rn in &rank_nodes {
            if let Some(edges) = self.edges.get(rn) {
                for other_rn in &rank_nodes {
                    sum += edges
                        .iter()
                        .filter(|(id, _)| id == other_rn)
                        .filter(|(id, _)| {
                            (self.nodes.get(id).unwrap().borrow().get_horizontal_rank() as i32
                                - self
                                    .nodes
                                    .get(other_rn)
                                    .unwrap()
                                    .borrow()
                                    .get_horizontal_rank() as i32)
                                .abs()
                                > 1
                        })
                        .count()
                }
            }
        }
        sum
    }

    fn find_child_nodes(&self, parents: &BTreeSet<String>) -> BTreeSet<String> {
        let mut children = BTreeSet::new();
        for p in parents {
            // Direct children
            if let Some(es) = self.edges.get(p) {
                let mut targets = es
                    .iter()
                    .filter_map(|(id, et)| match et {
                        EdgeType::SupportedBy => Some(id.to_owned()),
                        _ => None,
                    })
                    .collect();
                children.append(&mut targets);
            }
        }
        children.append(&mut self.find_in_context_nodes(&children));
        children
    }

    fn find_in_context_nodes(&self, parents: &BTreeSet<String>) -> BTreeSet<String> {
        let mut additional_nodes = BTreeSet::<String>::new();
        for id in parents {
            if let Some(target) = self.edges.get(id) {
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

    ///
    /// Layout the graph on a pseudo-stack layout
    ///
    /// 1) Let each element identify its size
    /// 2) Find and count the nodes on top level
    ///    Top level nodes only appear in source, but not in target of edges
    ///    Assumption: There are no unreferenced nodes.
    ///    Then, its the set of all nodes without the set of target nodes
    /// 3) Draw them
    ///    How to sort nodes on the same level?
    /// 4) Draw the edges
    ///
    fn layout(&self, mut doc: Document) -> Document {
        self.nodes
            .values()
            .for_each(|n| n.borrow_mut().calculate_size(&self.font, self.wrap));
        let ranks = self.place_nodes();
        // TODO Don't use HashSet, that makes order of nodes random.
        let mut n_rendered: BTreeSet<String> = BTreeSet::new();

        // TODO support forced level
        let mut last_max_height = 0;
        let mut y = self.margin.top;
        for rank in ranks {
            let mut x_s = HashMap::with_capacity(rank.len());
            let mut width_sum = 0;
            for id in rank.iter() {
                let n = self.nodes.get(id).unwrap().borrow();
                width_sum += n.get_width() + self.margin.left + self.margin.right;
                x_s.insert(id.clone(), width_sum - n.get_width() - self.margin.right);
            }
            let x_offset = std::cmp::max((self.width - width_sum) / 2, 0);
            for id in rank.iter() {
                let mut n = self.nodes.get(id).unwrap().borrow_mut();
                // TODO panics if width is too small for node
                last_max_height = std::cmp::max(last_max_height, n.get_height());
                doc = doc.add(n.render(x_s.get(id).unwrap() + x_offset, y, &self.font));
                n_rendered.insert(n.get_id().to_owned());
            }
            y += last_max_height + self.margin.top + self.margin.bottom;
            last_max_height = 0;
        }
        // Draw edges
        for (source, targets) in &self.edges {
            let s = self.nodes.get(source).unwrap();
            for (target, edge_type) in targets {
                let t = self.nodes.get(target).unwrap();
                doc = self.render_edge(doc, s.clone(), t.clone(), edge_type);
            }
        }
        doc
    }

    fn render_edge(
        &self,
        doc: Document,
        source: Rc<RefCell<dyn Node>>,
        target: Rc<RefCell<dyn Node>>,
        edge_type: &EdgeType,
    ) -> Document {
        // TODO class and id
        let s = source.borrow();
        let t = target.borrow();
        let (start, end) = if s.get_vertical_rank() < t.get_vertical_rank() {
            (
                s.get_coordinates(Port::South),
                t.get_coordinates(Port::North),
            )
        } else if s.get_vertical_rank() > t.get_vertical_rank() {
            (
                s.get_coordinates(Port::North),
                t.get_coordinates(Port::South),
            )
        } else {
            // s.get_vertical_rank() == t.get_vertical_rank()
            if s.get_horizontal_rank() > t.get_horizontal_rank() {
                (s.get_coordinates(Port::West), t.get_coordinates(Port::East))
            } else {
                (s.get_coordinates(Port::East), t.get_coordinates(Port::West))
            }
        };
        let parameters = match edge_type {
            edges::EdgeType::InContextOf => (
                (start.x + end.x) / 2,
                start.y,
                (start.x + end.x) / 2,
                end.y,
                end.x - 10, // Marker height
                end.y,
            ),
            edges::EdgeType::SupportedBy => (
                start.x,
                (start.y + end.y) / 2,
                end.x,
                (start.y + end.y) / 2,
                end.x,
                end.y - 10, // Marker height
            ),
        };
        let data = Data::new()
            .move_to((start.x, start.y))
            .cubic_curve_to(parameters);
        let arrow_id = match edge_type {
            edges::EdgeType::InContextOf => "url(#incontextof_arrow)",
            edges::EdgeType::SupportedBy => "url(#supportedby_arrow)",
        };
        let e = Path::new()
            .set("d", data)
            .set("marker-end", arrow_id)
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1u32);
        doc.add(e)
    }
}
