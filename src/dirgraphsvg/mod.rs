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
use crate::dirgraph::DirectedGraphEdgeType;
use crate::dirgraph::DirectedGraphNodeType;
use crate::dirgraphsvg::layout::layout_nodes;
use crate::dirgraphsvg::render::render_graph;
use crate::gsn::HorizontalIndex;

use self::edges::SingleEdge;
use self::layout::Margin;
use self::util::font::FontInfo;

impl<'a> DirectedGraphNodeType<'a> for RefCell<SvgNode> {
    fn get_forced_level(&self) -> Option<usize> {
        self.borrow().rank_increment
    }

    fn get_horizontal_index(&self, current_index: usize) -> Option<usize> {
        match self.borrow().horizontal_index {
            Some(HorizontalIndex::Absolute(x)) => match x {
                crate::gsn::AbsoluteIndex::Number(num) => num.try_into().ok(),
                crate::gsn::AbsoluteIndex::Last => Some(usize::MAX),
            },
            Some(HorizontalIndex::Relative(x)) => (x + current_index as i32).try_into().ok(),
            None => None,
        }
    }
}

impl<'a> DirectedGraphEdgeType<'a> for EdgeType {
    fn is_primary_child_edge(&self) -> bool {
        matches!(
            *self,
            EdgeType::OneWay(SingleEdge::SupportedBy)
                | EdgeType::OneWay(SingleEdge::Composite)
                | EdgeType::TwoWay((SingleEdge::SupportedBy, _))
                | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                | EdgeType::TwoWay((SingleEdge::Composite, _))
                | EdgeType::TwoWay((_, SingleEdge::Composite))
        )
    }

    fn is_secondary_child_edge(&self) -> bool {
        matches!(*self, EdgeType::OneWay(SingleEdge::InContextOf))
    }
}

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
        _cycles_allowed: bool,
    ) -> Result<(), std::io::Error> {
        // Calculate node sizes
        nodes
            .values_mut()
            .for_each(|n| n.calculate_optimal_size(&self.font));

        let nodes: BTreeMap<String, RefCell<SvgNode>> = nodes
            .into_iter()
            .map(|(a, b)| (a, RefCell::new(b)))
            .collect();
        // Rank nodes
        let graph = DirectedGraph::new(&nodes, &edges);
        let ranks = &graph.rank_nodes();
        // dbg!(&graph);
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
        let b1 = SvgNode::new_goal("id", &n, &[]);
        let mut nodes = BTreeMap::new();
        nodes.insert("G1".to_owned(), b1);
        d = d.add_meta_information(&mut vec!["A1".to_owned(), "B2".to_owned()]);
        let mut string_buffer = Vec::new();
        d.write(nodes, BTreeMap::new(), &mut string_buffer, false)
            .unwrap();
        println!("{}", std::str::from_utf8(string_buffer.as_slice()).unwrap());
    }
}
