use std::{collections::BTreeMap, io::Write};

use crate::{
    dirgraph::DirectedGraph,
    gsn::{self, GsnEdgeType, GsnNode, GsnNodeType, Module},
    render::RenderOptions,
};

use anyhow::Result;

///
/// Output list of evidence.
///
/// No template engine is used in order to keep dependencies to a minimum.
///
///
pub(crate) fn render_evidence(
    output: &mut impl Write,
    nodes: &BTreeMap<String, GsnNode>,
    render_options: &RenderOptions,
) -> Result<()> {
    writeln!(output)?;
    writeln!(output, "List of Evidence")?;
    writeln!(output)?;

    let mut solutions: Vec<(&String, &GsnNode)> = nodes
        .iter()
        .filter(|(_, node)| node.node_type == Some(GsnNodeType::Solution))
        .filter(|(id, node)| {
            !(render_options.masked_elements.contains(id)
                || render_options.masked_elements.contains(&node.module))
        })
        .collect();
    solutions.sort_by_key(|(k, _)| *k);
    if solutions.is_empty() {
        writeln!(output, "No evidence found.")?;
        println!("No evidence found.");
    } else {
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
        println!("OK")
    }

    Ok(())
}

///
/// Print statistics
///
///
pub(crate) fn render_statistics(
    output: &mut impl Write,
    nodes: &BTreeMap<String, GsnNode>,
    modules: &BTreeMap<String, Module>,
) -> Result<()> {
    writeln!(output, "Statistics")?;
    writeln!(output, "==========")?;
    writeln!(output, "Number of modules:   {}", modules.len())?;
    writeln!(output, "Number of nodes:     {}", nodes.len())?;
    writeln!(
        output,
        "  Goals:             {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Goal))
            .count()
    )?;
    writeln!(
        output,
        "  Strategies:        {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Strategy))
            .count()
    )?;
    writeln!(
        output,
        "  Solutions:         {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Solution))
            .count()
    )?;
    writeln!(
        output,
        "  Assumptions:       {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Assumption))
            .count()
    )?;
    writeln!(
        output,
        "  Justifications:    {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Justification))
            .count()
    )?;
    writeln!(
        output,
        "  Contexts:          {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Context))
            .count()
    )?;
    writeln!(
        output,
        "  Counter Goals:     {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::CounterGoal))
            .count()
    )?;
    writeln!(
        output,
        "  Counter Solutions: {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::CounterSolution))
            .count()
    )?;
    writeln!(
        output,
        "  Defeated Elements: {}",
        nodes.iter().filter(|n| n.1.defeated).count()
    )?;
    Ok(())
}

///
/// Render dump of complete graph into single YAML file that is ordered according to rank.
///
pub(crate) fn render_yaml_docs(
    mut output: &mut impl Write,
    nodes: &BTreeMap<String, GsnNode>,
    modules: &BTreeMap<String, Module>,
) -> Result<()> {
    let edges: BTreeMap<String, Vec<(String, GsnEdgeType)>> = nodes
        .iter()
        .map(|(id, node)| (id.to_owned(), node.get_edges()))
        .collect();
    let graph = DirectedGraph::new(nodes, &edges);
    let ranks = graph.rank_nodes();

    for (m_id, m) in modules {
        writeln!(&mut output, "--- # {} in {}\n", m_id, m.orig_file_name)?;
        serde_yml::to_writer(&mut output, &m.meta)?;
        for rank in ranks.iter().flatten() {
            let rank_map = rank
                .iter()
                .map(|&n| (n, nodes.get(n).unwrap()))
                .filter(|(_, n)| &n.module == m_id)
                .collect::<BTreeMap<&str, &GsnNode>>();
            if !rank_map.is_empty() {
                serde_yml::to_writer(&mut output, &rank_map)?;
            }
        }
        writeln!(&mut output, "... # {}\n", m_id)?;
    }

    // serde_yml::to_string(value)
    Ok(())
}
