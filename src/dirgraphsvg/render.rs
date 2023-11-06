use std::{cell::RefCell, collections::BTreeMap};

use anyhow::Context;
use svg::{
    node::element::{
        path::Data, Anchor, Element, Group, Marker, Path, Polyline, Rectangle, Style, Symbol, Title,
    },
    Document,
};

use crate::dirgraph::DirectedGraph;

use super::{
    edges::{EdgeType, SingleEdge},
    escape_node_id,
    nodes::{Node, Port},
    util::{escape_url, font::FontInfo},
    DirGraph,
};

const MARKER_HEIGHT: u32 = 10;

///
///
///
///
///
pub(super) fn render_graph(
    render_graph: &DirGraph,
    graph: &DirectedGraph<'_, RefCell<Node>, EdgeType>,
    ranks: &Vec<Vec<Vec<&str>>>,
    width: i32,
    height: i32,
) -> Document {
    let mut width = width;
    let mut height = height;
    let mut document = Document::new();
    let nodes = graph.get_nodes();
    let edges = graph.get_edges();
    document = setup_basics(document);
    document = setup_stylesheets(
        document,
        &render_graph.css_stylesheets,
        render_graph.embed_stylesheets,
    );
    // Draw nodes
    document = render_nodes(document, render_graph, nodes, ranks);
    // Draw edges
    document = render_edges(document, render_graph, nodes, edges);
    // Order is important here. render_legend may modify self.width and self.height
    document = render_legend(document, render_graph, &mut width, &mut height);
    document = document.set("viewBox", (0u32, 0u32, width, height));
    document
}

///
/// Render the edges
///
/// TODO Make edges nicer, if e.g., start marker is used. Make the first and last MARKER_HEIGHT pixels vertical.
///
///
fn render_edges(
    mut document: Document,
    render_graph: &DirGraph,
    nodes: &BTreeMap<String, RefCell<Node>>,
    edges: &BTreeMap<String, Vec<(String, EdgeType)>>,
) -> Document {
    for (source, targets) in edges {
        for (target, edge_type) in targets {
            let s = nodes.get(source).unwrap().borrow();
            let t = nodes.get(target).unwrap().borrow();
            let (marker_start_height, marker_end_height, support_distance) = match edge_type {
                // EdgeType::Invisible => (0i32, 0i32, 3i32 * MARKER_HEIGHT as i32),
                EdgeType::OneWay(_) => (0i32, MARKER_HEIGHT as i32, 3i32 * MARKER_HEIGHT as i32),
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
                } else if s_pos.y - s.get_height() / 2 - render_graph.margin.top
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
                | EdgeType::TwoWay((_, SingleEdge::InContextOf)) => Some("url(#incontextof_arrow)"),
                EdgeType::OneWay(SingleEdge::SupportedBy)
                | EdgeType::TwoWay((_, SingleEdge::SupportedBy)) => Some("url(#supportedby_arrow)"),
                EdgeType::OneWay(SingleEdge::Composite)
                | EdgeType::TwoWay((_, SingleEdge::Composite)) => Some("url(#composite_arrow)"),
                // EdgeType::Invisible => None,
            };
            let arrow_start_id = match &edge_type {
                EdgeType::TwoWay((SingleEdge::InContextOf, _)) => Some("url(#incontextof_arrow)"),
                EdgeType::TwoWay((SingleEdge::SupportedBy, _)) => Some("url(#supportedby_arrow)"),
                EdgeType::TwoWay((SingleEdge::Composite, _)) => Some("url(#composite_arrow)"),
                _ => None,
            };
            let mut classes = "gsnedge".to_string();
            match edge_type {
                EdgeType::OneWay(SingleEdge::InContextOf)
                | EdgeType::TwoWay((_, SingleEdge::InContextOf))
                | EdgeType::TwoWay((SingleEdge::InContextOf, _)) => classes.push_str(" gsninctxt"),
                EdgeType::OneWay(SingleEdge::SupportedBy)
                | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                | EdgeType::TwoWay((SingleEdge::SupportedBy, _)) => classes.push_str(" gsninspby"),
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
            document = document.add(e);
        }
    }
    document
}

///
///
///
///
fn render_nodes(
    mut document: Document,
    render_graph: &DirGraph,
    nodes: &BTreeMap<String, RefCell<Node>>,
    ranks: &Vec<Vec<Vec<&str>>>,
) -> Document {
    // Draw the nodes
    for rank in ranks {
        for np in rank {
            for &id in np {
                let n = nodes.get(id).unwrap();
                document = document.add(n.borrow().render(&render_graph.font));
            }
        }
    }
    document
    // Calculate size of document
    // let width = ranks
    //     .iter()
    //     .map(|rank| {
    //         let n = nodes.get(rank.iter().last().unwrap().first().unwrap().to_owned()).unwrap();
    //         n.get_x(&nodes) + n.get_max_width(&nodes)
    //     })
    //     .max()
    //     .unwrap_or(0);
    // let height = ranks
    //     .iter()
    //     .map(|rank| margin.top + self.get_max_height(rank) + margin.bottom)
    //     .sum();
    // (width, height)
}

///
///
///
///
fn render_legend(
    mut document: Document,
    render_graph: &DirGraph,
    width: &mut i32,
    height: &mut i32,
) -> Document {
    if let Some(meta) = &render_graph.meta_information {
        let mut g = create_group("gsn_module", &["gsnmodule".to_owned()], &None);
        let title = Title::new().add(svg::node::Text::new("Module Information"));
        use svg::Node;
        g.append(title);

        let mut text_height = 0;
        let mut text_width = 0;
        let mut lines = Vec::new();
        for t in meta {
            let (width, height) =
                crate::dirgraphsvg::util::font::text_bounding_box(&render_graph.font, t, false);
            lines.push((width, height));
            text_height += height;
            text_width = std::cmp::max(text_width, width);
        }

        if *width < text_width + 20i32 {
            *width = text_width + 40i32;
        }
        *height += text_height + 40i32;
        let x = *width - text_width - 20;
        let y_base = *height - text_height - 20;
        let mut y_running = 0;
        for (text, (_, h)) in meta.iter().zip(lines) {
            y_running += h;
            g = add_text(g, text, x, y_base + y_running, &render_graph.font, false);
        }
        document = document.add(g);
    }
    document
}

///
///
///
///
///
fn setup_basics(mut document: Document) -> Document {
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
    document = document.add(module_image);

    document = document
        .add(composite_arrow)
        .add(supportedby_arrow)
        .add(incontext_arrow)
        .set("classes", "gsndiagram");
    document
}

///
///
///
///
fn setup_stylesheets(
    mut document: Document,
    css_stylesheets: &[&str],
    embed_stylesheets: bool,
) -> Document {
    if !css_stylesheets.is_empty() {
        if embed_stylesheets {
            for css in css_stylesheets {
                let css_str = std::fs::read_to_string(css)
                    .context(format!("Failed to open CSS file {css} for embedding"))
                    .unwrap();
                let style = Style::new(format!("<![CDATA[{css_str}]]>")).set("type", "text/css");
                document = document.add(style);
            }
        } else {
            // Only link them
            let style = Style::new(
                css_stylesheets
                    .iter()
                    .map(|x| format!("@import {x}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            document = document.add(style);
        }
    }
    document
}

///
///
///
///
///
pub(crate) fn create_group(id: &str, classes: &[String], url: &Option<String>) -> Element {
    let mut g = Group::new().set("id", escape_node_id(id));
    g = g.set("class", classes.join(" "));
    if let Some(url) = &url {
        let link = Anchor::new();
        link.set("href", escape_url(url.as_str())).add(g).into()
    } else {
        g.into()
    }
}

///
///
///
///
pub(crate) fn add_text(
    mut context: Element,
    text: &str,
    x: i32,
    y: i32,
    font: &FontInfo,
    bold: bool,
) -> Element {
    use svg::node::element::Text;
    let mut text = Text::new()
        .set("x", x)
        .set("y", y)
        .set("font-size", font.size)
        .set("font-family", font.name.as_str())
        .add(svg::node::Text::new(text));
    if bold {
        text = text.set("font-weight", "bold");
    }
    use svg::Node;
    context.append(text);
    context
}
