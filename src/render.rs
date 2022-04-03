use crate::gsn::{from_gsn_node, get_levels, GsnNode, ModuleDependency};
use crate::yaml_fix::MyMap;
use dirgraphsvg::edges::EdgeType;
use dirgraphsvg::nodes::away_node::{AwayNode, AwayType};
use dirgraphsvg::nodes::Node;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Write;
use std::rc::Rc;

pub struct StaticRenderContext<'a> {
    pub modules: &'a [String],
    pub input_files: &'a [&'a str],
    pub layers: &'a Option<Vec<&'a str>>,
    pub stylesheet: Option<&'a str>,
}

///
///
///
///
pub fn render_architecture(
    _nodes: &MyMap<String, GsnNode>,
    _dependencies: Option<&BTreeMap<String, BTreeMap<String, ModuleDependency>>>,
    _output: &mut impl Write,
    _ctx: &StaticRenderContext,
) -> Result<(), anyhow::Error> {
    // TODO
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

///
/// Render all nodes in one diagram
///
/// TODO make it right
///
pub fn render_argument(
    module: &str,
    nodes: &MyMap<String, GsnNode>,
    output: &mut impl Write,
    ctx: &StaticRenderContext,
) -> Result<(), anyhow::Error> {
    let mut dg = dirgraphsvg::DirGraph::default();
    let mut svg_nodes = BTreeMap::<String, Rc<RefCell<dyn Node>>>::new();
    let mut edges: BTreeMap<String, Vec<(String, EdgeType)>> = nodes
        .iter()
        .filter(|(_, node)| node.module == module)
        .map(|(id, node)| {
            (
                id.to_owned(),
                node.get_edges()
                    .into_iter()
                    .map(|(t, et)| {
                        let target_node = nodes.get(&t).unwrap();
                        let target_mod = target_node.module.to_owned();
                        if target_mod != module {
                            if !svg_nodes.contains_key(&target_mod) {
                                let target_type = match &t {
                                    x if x.starts_with('G') => AwayType::Goal,
                                    x if x.starts_with("Sn") => AwayType::Solution,
                                    x if x.starts_with('A') => AwayType::Assumption,
                                    x if x.starts_with('J') => AwayType::Justification,
                                    x if x.starts_with('C') => AwayType::Context,
                                    _ => unimplemented!(), // TODO Strategy
                                };
                                svg_nodes.insert(
                                    target_mod.to_owned(),
                                    Rc::new(RefCell::new(AwayNode::new(
                                        &target_mod,
                                        &target_node.text,
                                        &target_mod,
                                        AwayType::Goal,
                                        match target_type {
                                            AwayType::Assumption => Some("A".to_owned()),
                                            AwayType::Justification => Some("J".to_owned()),
                                            _ => None,
                                        },
                                        None,
                                        None,
                                    ))),
                                );
                            }
                            (target_mod, et) // TODO wrong edge type
                        } else {
                            (t, et)
                        }
                    })
                    .collect::<Vec<(String, EdgeType)>>(),
            )
        })
        .collect();

    svg_nodes.append(
        &mut nodes
            .iter()
            .filter(|(_, node)| node.module == module)
            .map(|(id, node)| (id.to_owned(), from_gsn_node(id, node)))
            .collect(),
    );

    dbg!(&module);
    dbg!(&edges);
    dbg!(&svg_nodes.keys());

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
    writeln!(output)?;
    writeln!(output, "List of Evidences")?;
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
