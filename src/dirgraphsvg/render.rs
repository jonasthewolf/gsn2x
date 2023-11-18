use std::{cell::RefCell, collections::BTreeMap};

use anyhow::Context;
use svg::{
    node::element::{Group, Marker, Polyline, Rectangle, Style, Symbol, Text, Title},
    Document, Node,
};

use crate::dirgraph::DirectedGraph;

use super::{
    edges::{render_edge, EdgeType},
    escape_node_id,
    layout::{Cell, Margin},
    nodes::SvgNode,
    util::{font::FontInfo, point2d::Point2D},
    DirGraph,
};

///
/// Render the complete graph
///
pub(super) fn render_graph(
    render_graph: &DirGraph,
    graph: &DirectedGraph<'_, RefCell<SvgNode>, EdgeType>,
    ranks: &Vec<Vec<Vec<&str>>>,
    width: i32,
    height: i32,
) -> Document {
    let mut width = width;
    let mut height = height;
    let mut document = Document::new();
    document = setup_basics(document);
    document = setup_stylesheets(
        document,
        &render_graph.css_stylesheets,
        render_graph.embed_stylesheets,
    );
    // Draw nodes
    render_nodes(&mut document, graph, render_graph, ranks);
    // Draw edges
    render_edges(&mut document, graph, render_graph, ranks, width);
    // Order is important here. render_legend may modify self.width and self.height
    render_legend(&mut document, render_graph, &mut width, &mut height);
    document = document.set("viewBox", (0u32, 0u32, width, height));
    document
}

///
/// Render the edges
///
fn render_edges(
    document: &mut Document,
    graph: &DirectedGraph<'_, RefCell<SvgNode>, EdgeType>,
    render_graph: &DirGraph,
    ranks: &[Vec<Vec<&str>>],
    width: i32,
) {
    let bounding_boxes = ranks
        .iter()
        .map(|rank| get_bounding_boxes_in_rank(graph.get_nodes(), rank, &render_graph.margin))
        .collect::<Vec<_>>();

    let edges = graph.get_edges();
    for (source, targets) in edges {
        for (target, edge_type) in targets {
            let edge = render_edge(
                graph,
                ranks,
                &bounding_boxes,
                source,
                target,
                edge_type,
                width,
            );
            document.append(edge);
        }
    }
}

pub(crate) const TOP_LEFT_CORNER: usize = 0;
pub(crate) const TOP_RIGHT_CORNER: usize = 1;
pub(crate) const BOTTOM_RIGHT_CORNER: usize = 2;
pub(crate) const BOTTOM_LEFT_CORNER: usize = 3;

///
/// Get the full spaces for a given rank (incl. margin).
/// A vector of the coordinates of the bounding boxes is returned.
///
fn get_bounding_boxes_in_rank(
    nodes: &BTreeMap<String, RefCell<SvgNode>>,
    rank: &Vec<Vec<&str>>,
    margin: &Margin,
) -> Vec<[Point2D<i32>; 4]> {
    let mut boxes = vec![];
    for cell in rank {
        let x = cell.get_x(nodes);
        let y = cell.get_center_y(nodes);
        let width = cell.get_max_width(nodes);
        let height = cell.get_height(nodes, margin);
        let corners = [
            Point2D {
                // Top left
                x: x - width / 2 - margin.left,
                y: y - height / 2 - margin.top,
            },
            Point2D {
                // Top right
                x: x + width / 2 + margin.right,
                y: y - height / 2 - margin.top,
            },
            Point2D {
                // Bottom right
                x: x + width / 2 + margin.right,
                y: y + height / 2 + margin.bottom,
            },
            Point2D {
                // Bottom left
                x: x - width / 2 - margin.left,
                y: y + height / 2 + margin.bottom,
            },
        ];
        boxes.push(corners);
    }
    boxes
}

///
/// Render nodes
///
fn render_nodes(
    document: &mut Document,
    graph: &DirectedGraph<'_, RefCell<SvgNode>, EdgeType>,
    render_graph: &DirGraph,
    ranks: &Vec<Vec<Vec<&str>>>,
) {
    let nodes = graph.get_nodes();
    // Draw the nodes
    for rank in ranks {
        for np in rank {
            for &id in np {
                let n = nodes.get(id).unwrap();
                n.borrow().render(&render_graph.font, document);
            }
        }
    }
}

///
/// Render the optional legend
///
fn render_legend(
    document: &mut Document,
    render_graph: &DirGraph,
    width: &mut i32,
    height: &mut i32,
) {
    if let Some(meta) = &render_graph.meta_information {
        let mut g = create_group("gsn_module", &["gsnmodule".to_owned()]);
        let title = Title::new().add(svg::node::Text::new("Module Information"));
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
            g.append(create_text(
                text,
                x,
                y_base + y_running,
                &render_graph.font,
                false,
            ));
        }
        document.append(g);
    }
}

///
/// Setup the basic ingredients of the SVG
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
    document.append(module_image);

    document = document
        .add(composite_arrow)
        .add(supportedby_arrow)
        .add(incontext_arrow)
        .set("classes", "gsndiagram");
    document
}

///
/// Setup stylesheets in SVG
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
/// Crate a SVG group
///
pub(crate) fn create_group(id: &str, classes: &[String]) -> Group {
    Group::new()
        .set("id", escape_node_id(id))
        .set("class", classes.join(" "))
}

///
/// Create a SVG text element
///
pub(crate) fn create_text(text: &str, x: i32, y: i32, font: &FontInfo, bold: bool) -> Text {
    let mut text = Text::new()
        .set("x", x)
        .set("y", y)
        .set("font-size", font.size)
        .set("font-family", font.name.as_str())
        .add(svg::node::Text::new(text));
    if bold {
        text = text.set("font-weight", "bold");
    }
    text
}
