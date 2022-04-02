use crate::gsn::{from_gsn_node, get_levels, GsnNode, ModuleDependency};
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
    _nodes: &MyMap<String, GsnNode>,
    _dependencies: Option<&BTreeMap<String, BTreeMap<String, ModuleDependency>>>,
    _output: &mut impl Write,
    view: View,
    _ctx: &StaticRenderContext,
) -> Result<(), anyhow::Error> {
    let _template = match view {
        View::Argument => "argument.dot",
        View::Architecture => "architecture.dot",
    };
    // render_complete(nodes, output, ctx)?;
    Ok(())
}

///
/// Render all nodes in one diagram
///
pub fn render_complete(
    nodes: &MyMap<String, GsnNode>,
    output: &mut impl Write,
    ctx: &StaticRenderContext,
) -> Result<(), anyhow::Error> {
    let mut dg = dirgraphsvg::DirGraph::default();
    let mut edges: BTreeMap<String, Vec<(String, EdgeType)>> = nodes
        .iter()
        .map(|(id, node)| (id.to_owned(), node.get_edges()))
        .collect();
    let svg_nodes: BTreeMap<String, Rc<RefCell<dyn Node>>> = nodes
        .iter()
        .map(|(id, node)| (id.to_owned(), from_gsn_node(id, node)))
        .collect();
    dg = dg
        .add_nodes(svg_nodes)
        .add_edges(&mut edges)
        .add_levels(&get_levels(nodes));

    if let Some(css) = ctx.stylesheet {
        dg = dg.add_css_sytlesheet(css);
    }

    dg.write(output)?;

    Ok(())
}

pub(crate) fn render_evidences(
    nodes: &MyMap<String, GsnNode>,
    output: &mut impl Write,
    ctx: &StaticRenderContext,
) -> Result<(), anyhow::Error> {
    writeln!(output, "List of Evidences")?;
    writeln!(output)?;
    writeln!(output)?;

    let solutions: Vec<(&String, &GsnNode)> = nodes
        .iter()
        .filter(|(id, _)| id.starts_with("Sn"))
        .collect();
    if solutions.is_empty() {
        writeln!(output, "No evidences found.")?;
    }
    let width = (solutions.len() as f32).log10().ceil() as usize;
    for (i, (id, node)) in solutions.into_iter().enumerate() {
        writeln!(output, "{:>width$}. {}: {}", i + 1, id, node.text)?;
        let width = width + 2;
        writeln!(output, "{: >width$}{}", ' ', node.module)?;
        if let Some(url) = &node.url {
            writeln!(output, "{: >width$}{}", ' ', url)?;
        }
        for (layer, text) in node
            .additional
            .iter()
            .filter(|(l, _)| ctx.layers.iter().flatten().any(|x| x == l))
        {
            writeln!(
                output,
                "{: >width$}{}: {}",
                ' ',
                layer.to_ascii_uppercase(),
                text
            )?;
        }
    }

    Ok(())
}
