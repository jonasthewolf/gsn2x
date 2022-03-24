pub mod edges;
mod graph;
pub mod nodes;
mod util;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use edges::EdgeType;
use graph::{rank_nodes, NodePlace};
use nodes::{Node, Port};
use rusttype::Font;
use svg::{
    node::element::{path::Data, Marker, Path, Polyline},
    Document,
};
use util::{
    font::{get_default_font, get_font},
    point2d::Point2D,
};

const MARKER_HEIGHT: u32 = 10;

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
            right: 20,
            bottom: 20,
            left: 20,
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
    css_stylesheets: Vec<String>,
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
            css_stylesheets: Vec::new(),
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

    pub fn add_css_sytlesheet(mut self, css: &str) -> DirGraph {
        self.css_stylesheets.push(css.to_owned());
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

    pub fn write_to_file(mut self, file: &std::path::Path) -> Result<(), std::io::Error> {
        let mut document = Document::new().set("viewBox", (0u32, 0u32, self.width, self.height));

        document = setup_basics(document);
        document = self.layout(document);
        svg::save(file, &document)?;
        Ok(())
    }

    ///
    /// Layout the graph on a pseudo-stack layout
    ///
    /// 1) Let each element identify its size
    /// 2) Rank the nodes
    /// 3) Draw them
    /// 4) Draw the edges
    ///
    /// TODO Automatic width & height of document
    /// TODO Center in document
    ///
    fn layout(&mut self, mut doc: Document) -> Document {
        self.nodes
            .values()
            .for_each(|n| n.borrow_mut().calculate_size(&self.font, self.wrap));
        let ranks = rank_nodes(&mut self.nodes, &mut self.edges);
        let mut n_rendered: BTreeSet<String> = BTreeSet::new();

        let mut x = self.margin.left;
        let mut y = self.margin.top;
        for rank in ranks.values() {
            let height_max = rank
                .values()
                .map(|id| match id {
                    NodePlace::Node(id) => self.nodes.get(id).unwrap().borrow().get_height(),
                    NodePlace::MultipleNodes(ids) => {
                        ids.iter()
                            .map(|id| self.nodes.get(id).unwrap().borrow().get_height())
                            .sum::<u32>()
                            + (self.margin.top + self.margin.bottom) * (ids.len() - 1) as u32
                    }
                })
                .max()
                .unwrap();
            for np in rank.values() {
                match np {
                    NodePlace::Node(id) => {
                        let mut n = self.nodes.get(id).unwrap().borrow_mut();
                        x += n.get_width() / 2;
                        n.set_position(&Point2D {
                            x,
                            y: y + height_max / 2,
                        });
                        x += n.get_width() / 2 + self.margin.left + self.margin.right;
                        doc = doc.add(n.render(&self.font));
                        n_rendered.insert(n.get_id().to_owned());
                    }
                    NodePlace::MultipleNodes(ids) => {
                        let x_max = ids
                            .iter()
                            .map(|id| self.nodes.get(id).unwrap().borrow().get_width())
                            .max()
                            .unwrap();
                        let mut y_n = y;
                        for id in ids {
                            let mut n = self.nodes.get(id).unwrap().borrow_mut();
                            let n_height = n.get_height();
                            n.set_position(&Point2D {
                                x: x + x_max / 2,
                                y: y_n + n_height / 2,
                            });
                            y_n += n_height + self.margin.top + self.margin.bottom;
                            doc = doc.add(n.render(&self.font));
                            n_rendered.insert(n.get_id().to_owned());
                        }
                        x += x_max;
                    }
                }
            }
            x = self.margin.left;
            y += height_max + self.margin.left + self.margin.right;
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

    ///
    /// TODO make edges for invisible nodes smoother
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    fn render_edge(
        &self,
        doc: Document,
        source: Rc<RefCell<dyn Node>>,
        target: Rc<RefCell<dyn Node>>,
        edge_type: &EdgeType,
    ) -> Document {
        // TODO class and id
        let (marker_height, support_distance) = match edge_type {
            EdgeType::InContextOf => (MARKER_HEIGHT, 3 * MARKER_HEIGHT),
            EdgeType::SupportedBy => (MARKER_HEIGHT, 3 * MARKER_HEIGHT),
            EdgeType::Invisible => (0, 3 * MARKER_HEIGHT),
        };
        let s = source.borrow();
        let s_pos = s.get_position();
        let t = target.borrow();
        let t_pos = t.get_position();
        let (start, start_sup, end, end_sup) =
            if s_pos.y + s.get_height() / 2 < t_pos.y - t.get_height() / 2 {
                (
                    s.get_coordinates(Port::South),
                    s.get_coordinates(Port::South)
                        .move_relative(0, support_distance as i32),
                    t.get_coordinates(Port::North)
                        .move_relative(0, -(marker_height as i32)),
                    t.get_coordinates(Port::North)
                        .move_relative(0, -(support_distance as i32)),
                )
            } else if s_pos.y - s.get_height() / 2 > t_pos.y + t.get_height() / 2 {
                (
                    s.get_coordinates(Port::North),
                    s.get_coordinates(Port::North)
                        .move_relative(0, -(support_distance as i32)),
                    t.get_coordinates(Port::South)
                        .move_relative(0, marker_height as i32),
                    t.get_coordinates(Port::South)
                        .move_relative(0, support_distance as i32),
                )
            } else {
                // s.get_vertical_rank() == t.get_vertical_rank()
                if s_pos.x - s.get_width() / 2 > t_pos.x + t.get_width() / 2 {
                    (
                        s.get_coordinates(Port::West),
                        s.get_coordinates(Port::West),
                        t.get_coordinates(Port::East)
                            .move_relative(marker_height as i32, 0),
                        t.get_coordinates(Port::East)
                            .move_relative(support_distance as i32, 0),
                    )
                } else {
                    (
                        s.get_coordinates(Port::East),
                        s.get_coordinates(Port::East),
                        t.get_coordinates(Port::West)
                            .move_relative(-(marker_height as i32), 0),
                        t.get_coordinates(Port::West)
                            .move_relative(-(support_distance as i32), 0),
                    )
                }
            };
        let parameters = (start_sup.x, start_sup.y, end_sup.x, end_sup.y, end.x, end.y);
        let data = Data::new()
            .move_to((start.x, start.y))
            .cubic_curve_to(parameters);
        let arrow_id = match edge_type {
            edges::EdgeType::InContextOf => "url(#incontextof_arrow)",
            edges::EdgeType::SupportedBy => "url(#supportedby_arrow)",
            edges::EdgeType::Invisible => "",
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

fn setup_basics(mut doc: Document) -> Document {
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
        .set("markerUnits", "users_posaceOnUse")
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
        .set("markerUnits", "users_posaceOnUse")
        .add(incontext_polyline);

    doc = doc.add(supportedby_arrow).add(incontext_arrow);
    doc
}
