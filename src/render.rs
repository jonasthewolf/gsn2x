use crate::dirgraph::EdgeDecorator;
use crate::dirgraphsvg::edges::EdgeType;
use crate::dirgraphsvg::{escape_node_id, nodes::SvgNode};
use crate::file_utils::{get_filename, get_relative_path};
use crate::gsn::{GsnNode, GsnNodeType, Module};
use anyhow::Result;
use clap::ArgMatches;
use time::OffsetDateTime;
use time::format_description::well_known::Iso8601;

use std::collections::BTreeMap;
use std::io::Write;
use std::time::SystemTime;

#[derive(Debug, Default, Eq, PartialEq)]
pub enum RenderLegend {
    No,
    #[default]
    Short,
    Full,
}

pub struct RenderOptions<'a> {
    pub stylesheets: Vec<String>,
    pub masked_elements: Vec<String>,
    pub layers: Vec<String>,
    pub legend: RenderLegend,
    pub embed_stylesheets: bool,
    pub architecture_filename: Option<&'a str>,
    pub evidence_filename: Option<&'a str>,
    pub complete_filename: Option<&'a str>,
    pub output_directory: &'a str,
    pub skip_argument: bool,
    pub char_wrap: Option<u32>,
}

impl<'a> RenderOptions<'a> {
    pub fn new(
        matches: &'a ArgMatches,
        stylesheets: Vec<String>,
        embed_stylesheets: bool,
        output_directory: &'a str,
    ) -> Self {
        let legend = get_render_legend(matches);
        let layers = matches
            .get_many::<String>("LAYERS")
            .into_iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        let masked_elements: Vec<String> = matches
            .get_many::<String>("MASKED_MODULE")
            .unwrap_or_default()
            .chain(
                matches
                    .get_many::<String>("EXCLUDED_MODULE")
                    .unwrap_or_default(),
            )
            .cloned()
            .collect();

        RenderOptions {
            stylesheets,
            masked_elements,
            layers,
            legend,
            embed_stylesheets,
            architecture_filename: match matches.get_flag("NO_ARCHITECTURE_VIEW") {
                true => None,
                false => matches
                    .get_one::<String>("ARCHITECTURE_VIEW")
                    .and_then(|p| get_filename(p)),
            },
            evidence_filename: match matches.get_flag("NO_EVIDENCE") {
                true => None,
                false => matches
                    .get_one::<String>("EVIDENCE")
                    .and_then(|p| get_filename(p)),
            },
            complete_filename: match matches.get_flag("NO_COMPLETE_VIEW") {
                true => None,
                false => matches
                    .get_one::<String>("COMPLETE_VIEW")
                    .and_then(|p| get_filename(p)),
            },
            output_directory,
            skip_argument: matches.get_flag("NO_ARGUMENT_VIEW"),
            char_wrap: matches.get_one::<u32>("CHAR_WRAP").copied(),
        }
    }
}

///
/// Get RenderLegend form ArgMatches
///
fn get_render_legend(matches: &ArgMatches) -> RenderLegend {
    if matches.get_flag("NO_LEGEND") {
        RenderLegend::No
    } else if matches.get_flag("FULL_LEGEND") {
        RenderLegend::Full
    } else {
        RenderLegend::Short
    }
}

///
/// Create a SVG node from a GSN Node
///
///
///
pub fn svg_from_gsn_node(
    identifier: &str,
    gsn_node: &GsnNode,
    masked: bool,
    layers: &[String],
    char_wrap: Option<u32>,
) -> SvgNode {
    // Create node
    match gsn_node.node_type.unwrap() {
        // unwrap ok, since checked during validation
        GsnNodeType::Goal => SvgNode::new_goal(identifier, gsn_node, masked, layers, char_wrap),
        GsnNodeType::Solution => {
            SvgNode::new_solution(identifier, gsn_node, masked, layers, char_wrap)
        }
        GsnNodeType::Strategy => {
            SvgNode::new_strategy(identifier, gsn_node, masked, layers, char_wrap)
        }
        GsnNodeType::Context => {
            SvgNode::new_context(identifier, gsn_node, masked, layers, char_wrap)
        }
        GsnNodeType::Assumption => {
            SvgNode::new_assumption(identifier, gsn_node, masked, layers, char_wrap)
        }
        GsnNodeType::Justification => {
            SvgNode::new_justification(identifier, gsn_node, masked, layers, char_wrap)
        }
        GsnNodeType::CounterGoal => {
            SvgNode::new_counter_goal(identifier, gsn_node, masked, layers, char_wrap)
        }
        GsnNodeType::CounterSolution => {
            SvgNode::new_counter_solution(identifier, gsn_node, masked, layers, char_wrap)
        }
    }
}

///
/// Create an Away SVG node from a normal Node.
///
///
///
pub fn away_svg_from_gsn_node(
    identifier: &str,
    gsn_node: &GsnNode,
    masked: bool,
    module: &Module,
    source_module: &Module,
    layers: &[String],
    char_wrap: Option<u32>,
) -> Result<SvgNode> {
    let module_url = if masked {
        None
    } else {
        let mut x = get_relative_path(
            module.output_path.as_ref().unwrap(), // unwrap ok, since output_path is set initially.
            source_module.output_path.as_ref().unwrap(), // unwrap ok, since output_path is set initially.
        );
        x.push('#');
        x.push_str(&escape_node_id(identifier));
        Some(x)
    };
    // Create node
    Ok(match gsn_node.node_type.unwrap() {
        // unwrap ok, since checked during validation
        GsnNodeType::Goal => {
            SvgNode::new_away_goal(identifier, gsn_node, masked, layers, module_url, char_wrap)
        }
        GsnNodeType::Solution => {
            SvgNode::new_away_solution(identifier, gsn_node, masked, layers, module_url, char_wrap)
        }
        GsnNodeType::Strategy => {
            SvgNode::new_away_strategy(identifier, gsn_node, masked, layers, module_url, char_wrap)
        }
        GsnNodeType::Context => {
            SvgNode::new_away_context(identifier, gsn_node, masked, layers, module_url, char_wrap)
        }
        GsnNodeType::Assumption => SvgNode::new_away_assumption(
            identifier, gsn_node, masked, layers, module_url, char_wrap,
        ),
        GsnNodeType::Justification => SvgNode::new_away_justification(
            identifier, gsn_node, masked, layers, module_url, char_wrap,
        ),
        _ => unreachable!(),
    })
}

///
/// Render architecture view
///
pub fn render_architecture(
    output: &mut impl Write,
    modules: &BTreeMap<String, Module>,
    dependencies: BTreeMap<String, BTreeMap<String, EdgeType>>,
    render_options: &RenderOptions,
    architecture_path: &str,
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
                horizontal_index: module.meta.horizontal_index,
                rank_increment: module.meta.rank_increment,
                ..Default::default()
            };
            let module_url = Some(get_relative_path(
                module.output_path.as_ref().unwrap(),
                architecture_path,
            ));
            Ok((
                k.to_owned(),
                SvgNode::new_module(
                    k,
                    &module_node,
                    render_options.masked_elements.contains(&module.meta.name),
                    &[],
                    module_url,
                    module.meta.char_wrap.or(render_options.char_wrap),
                ),
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

    dg.write(svg_nodes, edges, output, BTreeMap::new())?;

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
pub fn render_complete<'a>(
    output: &mut impl Write,
    nodes: &'a BTreeMap<String, GsnNode>,
    render_options: &RenderOptions,
) -> Result<()> {
    // let masked_modules_opt = matches
    //     .values_of("MASK_MODULE")
    //     .map(|x| x.map(|y| y.to_owned()).collect::<Vec<String>>());
    // let masked_modules = masked_modules_opt.iter().flatten().collect::<Vec<_>>();
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let edges: BTreeMap<String, Vec<(String, EdgeType<'a>)>> = nodes
        .iter()
        // .filter(|(_, node)| !masked_modules.contains(&&node.module))
        // TODO continue masking here
        .map(|(id, node)| {
            (
                id.to_owned(),
                node.get_edges()
                    .into_iter()
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
                svg_from_gsn_node(
                    id,
                    node,
                    render_options.masked_elements.contains(id),
                    &render_options.layers,
                    render_options.char_wrap,
                ),
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

    dg.write(svg_nodes, edges, output, BTreeMap::new())?;

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
pub fn render_argument<'a>(
    output: &mut impl Write,
    module_name: &str,
    modules: &'a BTreeMap<String, Module>,
    nodes: &'a BTreeMap<String, GsnNode>,
    render_options: &RenderOptions,
) -> Result<()> {
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let mut svg_nodes: BTreeMap<String, SvgNode> = nodes
        .iter()
        .filter(|(_, node)| node.module == module_name)
        .map(|(id, node)| {
            (
                id.to_owned(),
                svg_from_gsn_node(
                    id,
                    node,
                    render_options.masked_elements.contains(id),
                    &render_options.layers,
                    modules
                        .get(module_name)
                        .and_then(|m| m.meta.char_wrap)
                        .or(render_options.char_wrap),
                ),
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
                        render_options.masked_elements.contains(id),
                        // unwraps are ok, since node.module and modules are consistently created
                        modules.get(&node.module).unwrap(),
                        modules.get(module_name).unwrap(),
                        &render_options.layers,
                        modules
                            .get(module_name)
                            .and_then(|m| m.meta.char_wrap)
                            .or(render_options.char_wrap),
                    )?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?,
    );

    let edges: BTreeMap<String, Vec<(String, EdgeType<'a>)>> = nodes
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
                    .map(|(s, t)| (s.to_owned(), EdgeType::from(t)))
                    .collect::<Vec<(String, EdgeType<'a>)>>(),
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
        let time: OffsetDateTime = SystemTime::now().into();
        let mut meta_info = vec![format!("Generated on: {}", time.format(&Iso8601::DEFAULT)?)];
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

    // Create ACPs as edge decorators from node information.
    let mut acps: BTreeMap<(String, String), EdgeDecorator> = BTreeMap::new();
    nodes.iter().for_each(|(s, n)| {
        n.acp.iter().for_each(|(acp, ts)| {
            ts.iter().filter(|&t| s != t).for_each(|t| {
                acps.entry((s.to_owned(), t.to_owned()))
                    .and_modify(|a: &mut EdgeDecorator| match a {
                        EdgeDecorator::Acps(items) => {
                            if !items.contains(acp) {
                                items.push(acp.to_owned());
                            }
                        }
                        EdgeDecorator::Defeated => unreachable!(),
                    })
                    .or_insert_with(|| EdgeDecorator::Acps(vec![acp.to_owned()]));
            });
        })
    });
    let mut defeated_relations: BTreeMap<(String, String), EdgeDecorator> = nodes
        .iter()
        .flat_map(|(s, n)| {
            n.defeated_relation
                .iter()
                .map(|t| ((s.to_owned(), t.to_owned()), EdgeDecorator::Defeated))
        })
        .collect();
    acps.append(&mut defeated_relations);

    dg.write(svg_nodes, edges, output, acps)?;

    Ok(())
}

#[cfg(test)]
mod test {

    use clap::{Arg, ArgAction, Command};

    use crate::{gsn::GsnNode, render::RenderLegend};

    use super::{get_render_legend, svg_from_gsn_node};

    #[test]
    #[should_panic]
    fn cover_unreachable() {
        let gsn_node = GsnNode::default();
        svg_from_gsn_node("X2", &gsn_node, false, &[], None);
    }

    #[test]
    fn translate_render_legend() -> anyhow::Result<()> {
        let cmd = Command::new("gsn2x")
            .arg(Arg::new("NO_LEGEND").short('G').action(ArgAction::SetTrue))
            .arg(
                Arg::new("FULL_LEGEND")
                    .short('g')
                    .action(ArgAction::SetTrue),
            );
        let matches = cmd.clone().try_get_matches_from(vec!["gsn2x"])?;
        assert_eq!(get_render_legend(&matches), RenderLegend::Short);
        let matches = cmd.clone().try_get_matches_from(vec!["gsn2x", "-G"])?;
        assert_eq!(get_render_legend(&matches), RenderLegend::No);
        let matches = cmd.clone().try_get_matches_from(vec!["gsn2x", "-g"])?;
        assert_eq!(get_render_legend(&matches), RenderLegend::Full);
        Ok(())
    }
}
