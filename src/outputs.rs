use std::{collections::BTreeMap, fs::File, io::Write, path::Path};

use crate::{
    dirgraph::DirectedGraph,
    gsn::{self, GsnEdgeType, GsnNode, GsnNodeType, Module},
    render::RenderOptions,
};

use anyhow::{Context, Result};

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
    nodes: &BTreeMap<String, GsnNode>,
    modules: &BTreeMap<String, Module>,
) {
    println!("Statistics");
    println!("==========");
    println!("Number of modules:   {}", modules.len());
    println!("Number of nodes:     {}", nodes.len());
    println!(
        "  Goals:             {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Goal))
            .count()
    );
    println!(
        "  Strategies:        {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Strategy))
            .count()
    );
    println!(
        "  Solutions:         {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Solution))
            .count()
    );
    println!(
        "  Assumptions:       {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Assumption))
            .count()
    );
    println!(
        "  Justifications:    {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Justification))
            .count()
    );
    println!(
        "  Contexts:          {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::Context))
            .count()
    );
    println!(
        "  Counter Goals:     {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::CounterGoal))
            .count()
    );
    println!(
        "  Counter Solutions: {}",
        nodes
            .iter()
            .filter(|n| n.1.node_type == Some(gsn::GsnNodeType::CounterSolution))
            .count()
    );
    println!(
        "  Defeated Elements: {}",
        nodes.iter().filter(|n| n.1.defeated).count()
    );
}

///
/// Render dump of complete graph into single YAML file that is ordered according to rank.
///
pub(crate) fn render_yaml_docs(
    nodes: &BTreeMap<String, GsnNode>,
    modules: &BTreeMap<String, Module>,
    output_file: &str,
) -> Result<()> {
    let edges: BTreeMap<String, Vec<(String, GsnEdgeType)>> = nodes
        .iter()
        .map(|(id, node)| (id.to_owned(), node.get_edges()))
        .collect();
    let graph = DirectedGraph::new(nodes, &edges);
    let ranks = graph.rank_nodes();

    let output_path = Path::new(output_file);
    if !&output_path.parent().unwrap().exists() {
        // Create output directory; unwraps are ok, since file always have a parent
        std::fs::create_dir_all(output_path.parent().unwrap()).with_context(|| {
            format!(
                "Could not create directory {} for {}",
                output_path.display(),
                output_file
            )
        })?;
    }
    let mut output_file = Box::new(File::create(output_path).context(format!(
        "Failed to open output file {}",
        output_path.display()
    ))?) as Box<dyn std::io::Write>;

    for (m_id, m) in modules {
        output_file.write_fmt(format_args!("--- # {} in {}\n", m_id, m.orig_file_name))?;
        serde_yml::to_writer(&mut output_file, &m.meta)?;
        for rank in ranks.iter().flatten() {
            let rank_map = rank
                .iter()
                .map(|&n| (n, nodes.get(n).unwrap()))
                .filter(|(_, n)| &n.module == m_id)
                .collect::<BTreeMap<&str, &GsnNode>>();
            if !rank_map.is_empty() {
                serde_yml::to_writer(&mut output_file, &rank_map)?;
            }
        }
        output_file.write_fmt(format_args!("... # {}\n", m_id))?;
    }

    // serde_yml::to_string(value)
    Ok(())
}
