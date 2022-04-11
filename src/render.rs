use crate::dirgraphsvg::edges::EdgeType;
use crate::dirgraphsvg::{escape_text, nodes::*};
use crate::gsn::{get_levels, GsnNode, Module};
use crate::yaml_fix::MyMap;
use chrono::Utc;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::rc::Rc;

pub fn svg_from_gsn_node(
    id: &str,
    gsn_node: &GsnNode,
) -> Rc<RefCell<dyn crate::dirgraphsvg::nodes::Node>> {
    let layer_classes: Option<Vec<String>> = gsn_node
        .additional
        .keys()
        .map(|k| {
            let mut t = escape_text(&k.to_ascii_lowercase());
            t.insert_str(0, "gsn_");
            Some(t.to_owned())
        })
        .collect();
    let mut mod_class = gsn_node.module.to_owned();
    mod_class.insert_str(0, "gsn_module_");
    let classes = gsn_node
        .classes
        .iter()
        .chain(layer_classes.iter())
        .flatten()
        .map(|x| Some(x.to_owned()))
        .chain(vec![mod_class].into_iter().map(Some))
        .collect();
    match id {
        id if id.starts_with('G') => new_goal(
            id,
            &gsn_node.text,
            gsn_node.undeveloped.unwrap_or(false),
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with("Sn") => {
            new_solution(id, &gsn_node.text, gsn_node.url.to_owned(), classes)
        }
        id if id.starts_with('S') => new_strategy(
            id,
            &gsn_node.text,
            gsn_node.undeveloped.unwrap_or(false),
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with('C') => {
            new_context(id, &gsn_node.text, gsn_node.url.to_owned(), classes)
        }
        id if id.starts_with('A') => {
            new_assumption(id, &gsn_node.text, gsn_node.url.to_owned(), classes)
        }
        id if id.starts_with('J') => {
            new_justification(id, &gsn_node.text, gsn_node.url.to_owned(), classes)
        }
        _ => unreachable!(),
    }
}

pub fn away_svg_from_gsn_node(
    id: &str,
    gsn_node: &GsnNode,
) -> Rc<RefCell<dyn crate::dirgraphsvg::nodes::Node>> {
    let layer_classes: Option<Vec<String>> = gsn_node
        .additional
        .keys()
        .map(|k| {
            let mut t = escape_text(&k.to_ascii_lowercase());
            t.insert_str(0, "gsn_");
            Some(t.to_owned())
        })
        .collect();

    let mut mod_class = gsn_node.module.to_owned();
    mod_class.insert_str(0, "gsn_module_");

    let classes = gsn_node
        .classes
        .iter()
        .chain(layer_classes.iter())
        .flatten()
        .map(|x| Some(x.to_owned()))
        .chain(vec![mod_class].into_iter().map(Some))
        .collect();

    match id {
        id if id.starts_with('G') => new_away_goal(
            id,
            &gsn_node.text,
            &gsn_node.module,
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with("Sn") => new_away_solution(
            id,
            &gsn_node.text,
            &gsn_node.module,
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with('S') => new_strategy(
            id,
            &gsn_node.text,
            gsn_node.undeveloped.unwrap_or(false),
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with('C') => new_away_context(
            id,
            &gsn_node.text,
            &gsn_node.module,
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with('A') => new_away_assumption(
            id,
            &gsn_node.text,
            &gsn_node.module,
            gsn_node.url.to_owned(),
            classes,
        ),
        id if id.starts_with('J') => new_away_justification(
            id,
            &gsn_node.text,
            &gsn_node.module,
            gsn_node.url.to_owned(),
            classes,
        ),
        _ => unreachable!(),
    }
}

///
///
///
pub fn render_architecture(
    output: &mut impl Write,
    modules: &HashMap<String, Module>,
    dependencies: BTreeMap<String, BTreeMap<String, EdgeType>>,
    stylesheets: Option<Vec<&str>>,
) -> Result<(), anyhow::Error> {
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let svg_nodes: BTreeMap<String, Rc<RefCell<dyn Node>>> = modules
        .iter()
        .filter(|(k, _)| dependencies.contains_key(k.to_owned()))
        .map(|(k, module)| {
            (
                k.to_owned(),
                new_module(
                    k,
                    module
                        .meta
                        .as_ref()
                        .and_then(|m| m.brief.to_owned())
                        .unwrap_or_else(|| "".to_owned())
                        .as_str(),
                    None,
                    None,
                ) as Rc<RefCell<dyn Node>>,
            )
        })
        .collect();
    let mut edges: BTreeMap<String, Vec<(String, EdgeType)>> = dependencies
        .into_iter()
        .map(|(k, v)| (k, Vec::from_iter(v.into_iter())))
        .collect();

    dg = dg.add_nodes(svg_nodes).add_edges(&mut edges);

    if let Some(mut css) = stylesheets {
        dg = dg.add_css_sytlesheets(&mut css);
    }

    dg.write(output)?;

    Ok(())
}

///
/// Render all nodes in one diagram
///
/// TODO mask modules MASK_MODULE
///
pub fn render_complete(
    output: &mut impl Write,
    _matches: &clap::ArgMatches,
    nodes: &MyMap<String, GsnNode>,
    stylesheets: Option<Vec<&str>>,
) -> Result<(), anyhow::Error> {
    // let masked_modules_opt = matches
    //     .values_of("MASK_MODULE")
    //     .map(|x| x.map(|y| y.to_owned()).collect::<Vec<String>>());
    // let masked_modules = masked_modules_opt.iter().flatten().collect::<Vec<_>>();
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let mut edges: BTreeMap<String, Vec<(String, EdgeType)>> = nodes
        .iter()
        // .filter(|(_, node)| !masked_modules.contains(&&node.module))
        // TODO continue masking here
        .map(|(id, node)| (id.to_owned(), node.get_edges()))
        .collect();
    let svg_nodes: BTreeMap<String, Rc<RefCell<dyn Node>>> = nodes
        .iter()
        .map(|(id, node)| (id.to_owned(), svg_from_gsn_node(id, node)))
        .collect();
    dg = dg
        .add_nodes(svg_nodes)
        .add_edges(&mut edges)
        .add_levels(&get_levels(nodes));

    if let Some(mut css) = stylesheets {
        dg = dg.add_css_sytlesheets(&mut css);
    }

    dg.write(output)?;

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
    matches: &clap::ArgMatches,
    module_name: &str,
    module: &Module,
    nodes: &MyMap<String, GsnNode>,
    stylesheets: Option<Vec<&str>>,
) -> Result<(), anyhow::Error> {
    let mut dg = crate::dirgraphsvg::DirGraph::default();
    let mut svg_nodes: BTreeMap<String, Rc<RefCell<dyn Node>>> = nodes
        .iter()
        .filter(|(_, node)| node.module == module_name)
        .map(|(id, node)| (id.to_owned(), svg_from_gsn_node(id, node)))
        .collect();

    svg_nodes.append(
        &mut nodes
            .iter()
            .filter(|(_, node)| node.module != module_name)
            .map(|(id, node)| (id.to_owned(), away_svg_from_gsn_node(id, node)))
            .collect(),
    );

    let mut edges: BTreeMap<String, Vec<(String, EdgeType)>> = nodes
        .iter()
        .map(|(id, node)| {
            (
                id.to_owned(),
                node.get_edges()
                    .into_iter()
                    .filter(|(target, _)| {
                        !(node.module != module_name
                            && nodes.get(target).unwrap().module != module_name)
                    })
                    .collect::<Vec<(String, EdgeType)>>(),
            )
        })
        .filter(|(_, targets)| !targets.is_empty())
        .collect();

    svg_nodes = svg_nodes
        .into_iter()
        .filter(|(id, _)| edges.contains_key(id) || edges.values().flatten().any(|(x, _)| x == id))
        .collect();

    dg = dg
        .add_nodes(svg_nodes)
        .add_edges(&mut edges)
        .add_levels(&get_levels(nodes));

    if let Some(mut css) = stylesheets {
        dg = dg.add_css_sytlesheets(&mut css);
    }

    // Add meta information if requested
    if !matches.is_present("NO_LEGEND") {
        let mut meta_info = vec![format!("Generated on {}", Utc::now())];
        if let Some(meta) = &module.meta {
            meta_info.insert(0, format!("Module: {}", meta.name));
            if meta.brief.is_some() {
                meta_info.insert(1, meta.brief.as_deref().unwrap().to_owned());
            }
            if matches.is_present("FULL_LEGEND") {
                let add = format!("{:?}", meta.additional);
                meta_info.append(&mut add.lines().map(|x| x.to_owned()).collect::<Vec<String>>());
            }
        }
        dg = dg.add_meta_information(&mut meta_info);
    }

    dg.write(output)?;

    Ok(())
}

pub(crate) fn render_evidences(
    output: &mut impl Write,
    nodes: &MyMap<String, GsnNode>,
    layers: &Option<Vec<&str>>,
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
            .filter(|(l, _)| layers.iter().flatten().any(|x| x == l))
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
