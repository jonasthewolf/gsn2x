use crate::dirgraphsvg::edges::EdgeType;
use crate::dirgraphsvg::{escape_node_id, nodes::SvgNode};
use crate::file_utils::{get_filename, get_relative_path, set_extension, translate_to_output_path};
use crate::gsn::{GsnNode, GsnNodeType, Module};
use anyhow::Result;
use chrono::Utc;
use clap::ArgMatches;

use std::collections::{BTreeMap, HashMap};
use std::io::Write;

#[derive(Default, Eq, PartialEq)]
pub enum RenderLegend {
    No,
    #[default]
    Short,
    Full,
}

pub struct RenderOptions<'a> {
    pub stylesheets: Vec<String>,
    pub layers: Vec<String>,
    pub legend: RenderLegend,
    pub embed_stylesheets: bool,
    pub architecture_filename: Option<&'a str>,
    pub evidences_filename: Option<&'a str>,
    pub complete_filename: Option<&'a str>,
    pub output_directory: Option<&'a str>,
    pub skip_argument: bool,
    pub word_wrap: Option<u32>,
}

impl<'a> RenderOptions<'a> {
    pub fn new(
        matches: &'a ArgMatches,
        stylesheets: Vec<String>,
        embed_stylesheets: bool,
        output_directory: Option<&'a String>,
    ) -> Self {
        let legend = if matches.get_flag("NO_LEGEND") {
            RenderLegend::No
        } else if matches.get_flag("FULL_LEGEND") {
            RenderLegend::Full
        } else {
            RenderLegend::Short
        };
        let layers = matches
            .get_many::<String>("LAYERS")
            .into_iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();

        RenderOptions {
            stylesheets,
            layers,
            legend,
            embed_stylesheets,
            architecture_filename: match matches.get_flag("NO_ARCHITECTURE_VIEW") {
                true => None,
                false => matches
                    .get_one::<String>("ARCHITECTURE_VIEW")
                    .and_then(|p| get_filename(p)),
            },

            evidences_filename: match matches.get_flag("NO_EVIDENCES") {
                true => None,
                false => matches
                    .get_one::<String>("EVIDENCES")
                    .and_then(|p| get_filename(p)),
            },
            complete_filename: match matches.get_flag("NO_COMPLETE_VIEW") {
                true => None,
                false => matches
                    .get_one::<String>("COMPLETE_VIEW")
                    .and_then(|p| get_filename(p)),
            },
            output_directory: output_directory.map(|x| x.as_str()),
            skip_argument: matches.get_flag("NO_ARGUMENT_VIEW"),
            word_wrap: matches.get_one::<u32>("WORD_WRAP").copied(),
        }
    }
}

///
///
///
///
///
pub fn svg_from_gsn_node(
    identifier: &str,
    gsn_node: &GsnNode,
    layers: &[String],
    char_wrap: Option<u32>,
) -> SvgNode {
    // Create node
    match gsn_node.node_type.unwrap() {
        // unwrap ok, since checked during validation
        GsnNodeType::Goal => SvgNode::new_goal(identifier, gsn_node, layers, char_wrap),
        GsnNodeType::Solution => SvgNode::new_solution(identifier, gsn_node, layers, char_wrap),
        GsnNodeType::Strategy => SvgNode::new_strategy(identifier, gsn_node, layers, char_wrap),
        GsnNodeType::Context => SvgNode::new_context(identifier, gsn_node, layers, char_wrap),
        GsnNodeType::Assumption => SvgNode::new_assumption(identifier, gsn_node, layers, char_wrap),
        GsnNodeType::Justification => {
            SvgNode::new_justification(identifier, gsn_node, layers, char_wrap)
        }
    }
}

///
///
///
///
///
pub fn away_svg_from_gsn_node(
    identifier: &str,
    gsn_node: &GsnNode,
    module: &Module,
    source_module: &Module,
    layers: &[String],
    char_wrap: Option<u32>,
) -> Result<SvgNode> {
    let mut module_url = get_relative_path(
        &module.relative_module_path,
        &source_module.relative_module_path,
        Some("svg"),
    )?;
    module_url.push('#');
    module_url.push_str(&escape_node_id(identifier));

    // Create node
    Ok(match gsn_node.node_type.unwrap() {
        // unwrap ok, since checked during validation
        GsnNodeType::Goal => {
            SvgNode::new_away_goal(identifier, gsn_node, layers, Some(module_url), char_wrap)
        }
        GsnNodeType::Solution => {
            SvgNode::new_away_solution(identifier, gsn_node, layers, Some(module_url), char_wrap)
        }
        GsnNodeType::Strategy => SvgNode::new_strategy(identifier, gsn_node, layers, char_wrap),
        GsnNodeType::Context => {
            SvgNode::new_away_context(identifier, gsn_node, layers, Some(module_url), char_wrap)
        }
        GsnNodeType::Assumption => {
            SvgNode::new_away_assumption(identifier, gsn_node, layers, Some(module_url), char_wrap)
        }
        GsnNodeType::Justification => SvgNode::new_away_justification(
            identifier,
            gsn_node,
            layers,
            Some(module_url),
            char_wrap,
        ),
    })
}

///
///
///
pub fn render_architecture(
    output: &mut impl Write,
    modules: &HashMap<String, Module>,
    dependencies: BTreeMap<String, BTreeMap<String, EdgeType>>,
    render_options: &RenderOptions,
    architecture_path: &str,
    output_path: &str,
) -> Result<()> {
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let svg_nodes: BTreeMap<String, SvgNode> = modules
        .iter()
        .filter(|(k, _)| dependencies.contains_key(k.to_owned()))
        .map(|(k, module)| {
            let module_node = GsnNode {
                text: module
                    .meta
                    .brief
                    .as_ref()
                    .map(|m| m.to_owned())
                    .unwrap_or_else(|| "".to_owned()),
                ..Default::default()
            };
            let module_url = Some({
                let target_svg = set_extension(&module.relative_module_path, "svg");
                let target_path = translate_to_output_path(output_path, &target_svg)?;
                get_relative_path(
                    &target_path,
                    architecture_path,
                    None, // is already made "svg" above
                )?
            });
            Ok((
                k.to_owned(),
                SvgNode::new_module(k, &module_node, &[], module_url, render_options.word_wrap),
            ))
        })
        .collect::<Result<BTreeMap<String, SvgNode>>>()?;
    let edges: BTreeMap<String, Vec<(String, EdgeType)>> = dependencies
        .into_iter()
        .map(|(k, v)| (k, Vec::from_iter(v)))
        .collect();

    dg = dg
        .embed_stylesheets(render_options.embed_stylesheets)
        .add_css_stylesheets(
            &mut render_options
                .stylesheets
                .iter()
                .map(AsRef::as_ref)
                .collect(),
        );

    dg.write(svg_nodes, edges, output, true)?;

    Ok(())
}

///
/// Render all nodes in one diagram
///
/// TODO mask modules MASK_MODULE
///
/// FIXME: Problem horizontal index only applies to argument view
///        potential solution: puzzle individual argument views together...
///
pub fn render_complete(
    output: &mut impl Write,
    nodes: &BTreeMap<String, GsnNode>,
    render_options: &RenderOptions,
) -> Result<()> {
    // let masked_modules_opt = matches
    //     .values_of("MASK_MODULE")
    //     .map(|x| x.map(|y| y.to_owned()).collect::<Vec<String>>());
    // let masked_modules = masked_modules_opt.iter().flatten().collect::<Vec<_>>();
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let edges: BTreeMap<String, Vec<(String, EdgeType)>> = nodes
        .iter()
        // .filter(|(_, node)| !masked_modules.contains(&&node.module))
        // TODO continue masking here
        .map(|(id, node)| {
            (
                id.to_owned(),
                node.get_edges()
                    .iter()
                    .map(|(s, t)| (s.to_owned(), EdgeType::from(t)))
                    .collect(),
            )
        })
        .collect();
    let svg_nodes: BTreeMap<String, SvgNode> = nodes
        .iter()
        .map(|(id, node)| {
            (
                id.to_owned(),
                svg_from_gsn_node(id, node, &render_options.layers, render_options.word_wrap),
            )
        })
        .collect();
    dg = dg
        .embed_stylesheets(render_options.embed_stylesheets)
        .add_css_stylesheets(
            &mut render_options
                .stylesheets
                .iter()
                .map(AsRef::as_ref)
                .collect(),
        );

    dg.write(svg_nodes, edges, output, false)?;

    Ok(())
}

///
/// Render all nodes in one diagram
///
///  1) Map gsn nodes to svg nodes
///     foreign module nodes will be mapped to the away svg node
///  2) Replace the edges with the right ones
///  3) filter all foreign modules that have no edge to this module
///
pub fn render_argument(
    output: &mut impl Write,
    module_name: &str,
    modules: &HashMap<String, Module>,
    nodes: &BTreeMap<String, GsnNode>,
    render_options: &RenderOptions,
) -> Result<()> {
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let mut svg_nodes: BTreeMap<String, SvgNode> = nodes
        .iter()
        .filter(|(_, node)| node.module == module_name)
        .map(|(id, node)| {
            (
                id.to_owned(),
                svg_from_gsn_node(id, node, &render_options.layers, render_options.word_wrap),
            )
        })
        .collect();

    svg_nodes.append(
        &mut nodes
            .iter()
            .filter(|(_, node)| node.module != module_name)
            .map(|(id, node)| {
                Ok((
                    id.to_owned(),
                    away_svg_from_gsn_node(
                        id,
                        node,
                        // unwraps are ok, since node.module and modules are consistently created
                        modules.get(&node.module).unwrap(),
                        modules.get(module_name).unwrap(),
                        &render_options.layers,
                        render_options.word_wrap,
                    )?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?,
    );

    let edges: BTreeMap<String, Vec<(String, EdgeType)>> = nodes
        .iter()
        .map(|(id, node)| {
            (
                id.to_owned(),
                node.get_edges()
                    .into_iter()
                    .filter(|(target, _)| {
                        !(node.module != module_name
                            // unwrap is ok, since all references are checked at the beginning
                            && nodes.get(target).unwrap().module != module_name)
                    })
                    .map(|(s, t)| (s.to_owned(), EdgeType::from(&t)))
                    .collect::<Vec<(String, EdgeType)>>(),
            )
        })
        .filter(|(_, targets)| !targets.is_empty())
        .collect();

    svg_nodes.retain(|id, _| {
        edges.contains_key(id)
            || edges.values().flatten().any(|(x, _)| x == id)
            // unwrap is ok, since all references are checked at the beginning
            || nodes.get(id).unwrap().module == module_name
    });

    dg = dg
        .embed_stylesheets(render_options.embed_stylesheets)
        .add_css_stylesheets(
            &mut render_options
                .stylesheets
                .iter()
                .map(AsRef::as_ref)
                .collect(),
        );

    // Add meta information if requested
    if render_options.legend != RenderLegend::No {
        let mut meta_info = vec![format!("Generated on: {}", Utc::now())];
        if let Some(meta) = &modules.get(module_name).map(|x| &x.meta) {
            meta_info.insert(0, format!("Module: {}", meta.name));
            if let Some(brief) = &meta.brief {
                meta_info.insert(1, brief.to_owned());
            }
            if render_options.legend == RenderLegend::Full {
                let add = format!("{:?}", meta.additional);
                meta_info.append(&mut add.lines().map(|x| x.to_owned()).collect::<Vec<String>>());
            }
        }
        dg = dg.add_meta_information(&mut meta_info);
    }

    dg.write(svg_nodes, edges, output, false)?;

    Ok(())
}

///
/// Output list of evidences.
///
/// No template engine is used in order to keep dependencies to a minimum.
///
///
pub(crate) fn render_evidences(
    output: &mut impl Write,
    nodes: &BTreeMap<String, GsnNode>,
    render_options: &RenderOptions,
) -> Result<()> {
    writeln!(output)?;
    writeln!(output, "List of Evidences")?;
    writeln!(output)?;

    let mut solutions: Vec<(&String, &GsnNode)> = nodes
        .iter()
        .filter(|(_, node)| node.node_type == Some(GsnNodeType::Solution))
        .collect();
    solutions.sort_by_key(|(k, _)| *k);
    if solutions.is_empty() {
        writeln!(output, "No evidences found.")?;
    }
    let width = (solutions.len() as f32).log10().ceil() as usize;
    for (i, (id, node)) in solutions.into_iter().enumerate() {
        writeln!(
            output,
            "{:>width$}. {}: {}",
            i + 1,
            id,
            node.text
                .replace('\n', &format!("\n{: >w$}", ' ', w = width + 4 + id.len()))
        )?;
        let width = width + 2;
        writeln!(output)?;
        writeln!(output, "{: >width$}{}", ' ', node.module)?;
        writeln!(output)?;
        if let Some(url) = &node.url {
            writeln!(output, "{: >width$}{}", ' ', url)?;
            writeln!(output)?;
        }
        for (layer, text) in node
            .additional
            .iter()
            .filter(|(l, _)| render_options.layers.iter().any(|x| &x == l))
        {
            writeln!(
                output,
                "{: >width$}{}: {}",
                ' ',
                layer.to_ascii_uppercase(),
                text.replace(
                    '\n',
                    &format!("\n{: >w$}", ' ', w = width + 2 + layer.len())
                )
            )?;
            writeln!(output)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use crate::gsn::GsnNode;

    use super::svg_from_gsn_node;

    #[test]
    #[should_panic]
    fn cover_unreachable() {
        let gsn_node = GsnNode::default();
        svg_from_gsn_node("X2", &gsn_node, &[], None);
    }
}
