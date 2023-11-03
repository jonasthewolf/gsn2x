pub mod edges;
mod layout;
pub mod nodes;
mod render;
mod util;

use serde_yaml::with;
use std::collections::BTreeMap;
pub use util::{escape_node_id, escape_text};

use edges::EdgeType;
use nodes::{Node, Port};

use crate::dirgraph::DirectedGraph;
use crate::dirgraph::DirectedGraphEdgeType;
use crate::dirgraph::DirectedGraphNodeType;
use crate::dirgraphsvg::layout::layout_nodes;
use crate::dirgraphsvg::render::render_graph;
// use crate::dirgraphsvg::layout::layout_nodes;
use crate::gsn::HorizontalIndex;

use self::edges::SingleEdge;
use self::layout::Margin;
use self::util::font::FontInfo;

impl<'a> DirectedGraphNodeType<'a> for Node {
    fn is_final_node(&'a self) -> bool {
        false
    }

    fn get_forced_level(&'a self) -> Option<usize> {
        self.rank_increment
    }

    fn get_horizontal_index(&'a self, current_index: usize) -> Option<usize> {
        match self.horizontal_index {
            Some(HorizontalIndex::Absolute(x)) => x.try_into().ok(),
            Some(HorizontalIndex::Relative(x)) => (x + current_index as i32).try_into().ok(),
            None => None,
        }
    }

    fn get_mut(&'a mut self) -> &'a mut Self {
        self
    }
}

impl<'a> DirectedGraphEdgeType<'a> for EdgeType {
    fn is_primary_child_edge(&'a self) -> bool {
        !self.is_secondary_child_edge()
    }

    fn is_secondary_child_edge(&'a self) -> bool {
        match *self {
            EdgeType::OneWay(SingleEdge::InContextOf) => true,
            EdgeType::TwoWay((s, t))
                if s == SingleEdge::InContextOf || t == SingleEdge::InContextOf =>
            {
                true
            }
            _ => false,
        }
    }
}

pub struct DirGraph<'a> {
    margin: Margin,
    nodes: BTreeMap<String, Node>,
    edges: BTreeMap<String, Vec<(String, EdgeType)>>,
    forced_levels: BTreeMap<&'a str, Vec<&'a str>>,
    font: FontInfo,
    css_stylesheets: Vec<&'a str>,
    embed_stylesheets: bool,
    meta_information: Option<Vec<String>>,
}

impl<'a> Default for DirGraph<'a> {
    fn default() -> Self {
        Self {
            // TODO Where now?
            // width: 210,
            // height: 297,
            margin: Margin::default(),
            font: FontInfo::default(),
            css_stylesheets: Vec::new(),
            embed_stylesheets: false,
            forced_levels: BTreeMap::new(),
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
            meta_information: None,
        }
    }
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

    pub fn add_nodes(mut self, mut nodes: BTreeMap<String, Node>) -> Self {
        self.nodes.append(&mut nodes);
        self
    }

    pub fn add_edges(mut self, edges: &mut BTreeMap<String, Vec<(String, EdgeType)>>) -> Self {
        self.edges.append(edges);
        self
    }

    pub fn add_forced_levels(mut self, levels: &BTreeMap<&'a str, Vec<&'a str>>) -> Self {
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
        // Calculate node sizes
        self.nodes
            .values_mut()
            .for_each(|n| n.calculate_optimal_size(&self.font));

        // Rank nodes
        let graph = DirectedGraph::new(&self.nodes, &self.edges);
        let ranks = &graph.rank_nodes();
        // dbg!(ranks);
        dbg!(&graph);
        // Layout graph
        let (width, height) = layout_nodes(&mut self.nodes, &graph, &ranks, &self.margin, &graph.get_parent_edges());

        let width = 500;
        let height = 500;
        // // Render to SVG
        let document = render_graph(
            &self.nodes,
            &self.edges,
            ranks,
            width,
            height,
            &self.font,
            self.margin,
        );
        output.write_all("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".as_bytes())?;
        svg::write(output, &document)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_render_legend() {
        let mut d = DirGraph::default();
        let b1 = Node::new_away_goal("id", "text", "module", None, None, None, None, vec![]);
        let mut nodes = BTreeMap::new();
        nodes.insert("G1".to_owned(), b1);
        d = d.add_nodes(nodes);
        d = d.add_meta_information(&mut vec!["A1".to_owned(), "B2".to_owned()]);
        let mut string_buffer = Vec::new();
        d.write(&mut string_buffer, false).unwrap();
        println!("{}", std::str::from_utf8(string_buffer.as_slice()).unwrap());
    }
}
