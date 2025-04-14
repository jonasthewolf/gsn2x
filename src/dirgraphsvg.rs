pub mod edges;
mod layout;
pub mod nodes;
mod render;
mod util;

use std::cell::RefCell;
use std::collections::BTreeMap;
pub use util::{escape_node_id, escape_text};

use edges::EdgeType;
use nodes::{Port, SvgNode};

use crate::dirgraph::DirectedGraph;
use crate::dirgraphsvg::layout::layout_nodes;
use crate::dirgraphsvg::render::render_graph;

use self::layout::Margin;
use self::util::font::FontInfo;

#[derive(Default)]
pub struct DirGraph<'a> {
    margin: Margin,
    font: FontInfo,
    css_stylesheets: Vec<&'a str>,
    embed_stylesheets: bool,
    meta_information: Option<Vec<String>>,
}

impl<'a> DirGraph<'a> {
    pub fn add_css_stylesheets(mut self, css: &mut Vec<&'a str>) -> Self {
        self.css_stylesheets.append(css);
        self
    }

    pub fn embed_stylesheets(mut self, embed: bool) -> Self {
        self.embed_stylesheets = embed;
        self
    }

    pub fn add_meta_information(mut self, meta: &mut Vec<String>) -> Self {
        self.meta_information.get_or_insert(Vec::new()).append(meta);
        self
    }

    pub fn write(
        self,
        mut nodes: BTreeMap<String, SvgNode>,
        edges: BTreeMap<String, Vec<(String, EdgeType)>>,
        mut output: impl std::io::Write,
        edge_decorators: BTreeMap<(String, String), Vec<String>>,
    ) -> Result<(), std::io::Error> {
        // Calculate node sizes
        nodes
            .values_mut()
            .for_each(|n| n.calculate_size(&self.font));
        // Translate to RefCell to be usable by DirectedGraph
        let nodes: BTreeMap<String, RefCell<SvgNode>> = nodes
            .into_iter()
            .map(|(a, b)| (a, RefCell::new(b)))
            .collect();
        // Rank nodes
        let mut graph = DirectedGraph::new(&nodes, &edges);
        graph.add_edge_decorators(edge_decorators);
        let ranks = &graph.rank_nodes();
        // Layout graph
        let (width, height) = layout_nodes(&graph, ranks, &self.margin);
        // Render to SVG
        let document = render_graph(&self, &graph, ranks, width, height);
        output.write_all("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".as_bytes())?;
        svg::write(output, &document)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::gsn::GsnNode;

    use super::*;

    #[test]
    fn test_render_legend() {
        let mut d = DirGraph::default();
        let n = GsnNode {
            text: "Test".to_owned(),
            ..Default::default()
        };
        let b1 = SvgNode::new_goal("id", &n, false, false, &[], None);
        let mut nodes = BTreeMap::new();
        nodes.insert("G1".to_owned(), b1);
        d = d.add_meta_information(&mut vec!["A1".to_owned(), "B2".to_owned()]);
        let mut string_buffer = Vec::new();
        d.write(nodes, BTreeMap::new(), &mut string_buffer, BTreeMap::new())
            .unwrap();
        println!("{}", std::str::from_utf8(string_buffer.as_slice()).unwrap());
    }
}
