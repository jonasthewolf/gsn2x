use crate::gsn::{from_gsn_node, get_forced_levels, GsnNode, ModuleDependency};
use crate::yaml_fix::MyMap;
use dirgraphsvg::edges::EdgeType;
use dirgraphsvg::nodes::Node;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Write;
use std::rc::Rc;

pub enum View {
    Argument,
    Architecture,
    Complete,
    Evidences,
}

pub struct StaticRenderContext<'a> {
    pub modules: &'a [String],
    pub input_files: &'a [&'a str],
    pub layers: &'a Option<Vec<&'a str>>,
    pub stylesheet: Option<&'a str>,
}

///
/// Use Tera to create dot-file.
/// Templates are inlined in executable.
///
///
pub fn render_view(
    _module: &str,
    nodes: &MyMap<String, GsnNode>,
    _dependencies: Option<&BTreeMap<String, BTreeMap<String, ModuleDependency>>>,
    _output: &mut impl Write,
    view: View,
    _ctx: &StaticRenderContext,
) -> Result<(), anyhow::Error> {
    // Note the max() at the end, so we don't get a NaN when calculating width
    let num_solutions = nodes
        .iter()
        .filter(|(id, _)| id.starts_with("Sn"))
        .count()
        .max(1);
    let _width = (num_solutions as f32).log10().ceil() as usize;
    let _template = match view {
        View::Argument => "argument.dot",
        View::Architecture => "architecture.dot",
        View::Complete => "complete.dot",
        View::Evidences => "evidences.md",
    };
    render_complete(nodes)?;
    Ok(())
}

///
/// Render all nodes in one diagram
///
fn render_complete(nodes: &MyMap<String, GsnNode>) -> Result<(), anyhow::Error> {
    let dg = dirgraphsvg::DirGraph::default();
    let mut edges: BTreeMap<String, Vec<(String, EdgeType)>> = nodes
        .iter()
        .map(|(id, node)| (id.to_owned(), node.get_edges()))
        .collect();
    let forced_levels = get_forced_levels(nodes);
    let mut svg_nodes: BTreeMap<String, Rc<RefCell<dyn Node>>> = nodes
        .iter()
        .map(|(id, node)| (id.to_owned(), from_gsn_node(id, node, &forced_levels)))
        .collect();
    dg.add_nodes(&mut svg_nodes)
        .add_edges(&mut edges)
        .write_to_file(std::path::Path::new("complete.svg"))?;

    Ok(())
}
