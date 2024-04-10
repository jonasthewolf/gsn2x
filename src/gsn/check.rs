use super::{GsnEdgeType, GsnNode, GsnNodeType};
use crate::{diagnostics::Diagnostics, dirgraph::DirectedGraph};
use std::collections::BTreeMap;

///
/// Entry function to all checks.
///
///
pub fn check_nodes(
    diag: &mut Diagnostics,
    nodes: &BTreeMap<String, GsnNode>,
    excluded_modules: &[&str],
) -> Result<(), ()> {
    check_node_references(diag, nodes, excluded_modules)?;
    check_root_nodes(diag, nodes).and_then(|_| {
        let edges: BTreeMap<String, Vec<(String, GsnEdgeType)>> = nodes
            .iter()
            .map(|(id, node)| (id.to_owned(), node.get_edges()))
            .collect();
        let graph = DirectedGraph::new(nodes, &edges);
        check_cycles(diag, &graph).and_then(|_| check_unreachable(diag, &graph))
    })
}

///
/// Check if there is one and only one unreferenced node
/// and if it is a Goal
///
///
fn check_root_nodes(
    diag: &mut Diagnostics,
    nodes: &BTreeMap<String, GsnNode>,
) -> Result<Vec<String>, ()> {
    let root_nodes = super::get_root_nodes(nodes);
    match root_nodes.len() {
        x if x > 1 => {
            let mut wn = root_nodes.to_vec();
            wn.sort();
            diag.add_warning(
                None,
                format!(
                    "C01: There is more than one unreferenced element: {}.",
                    wn.join(", ")
                ),
            );
            Ok(root_nodes)
        }
        1 => {
            let rootn = root_nodes.first().unwrap(); // unwrap is ok, since we just checked that there is an element in Vec
            if nodes.get(rootn).unwrap().node_type != Some(GsnNodeType::Goal) {
                diag.add_error(
                    None,
                    format!("C02: The root element should be a goal, but {rootn} was found."),
                );
                Err(())
            } else {
                Ok(root_nodes)
            }
        }
        x if x == 0 && !nodes.is_empty() => {
            diag.add_error(
                None,
                "C01: There are no unreferenced elements found.".to_owned(),
            );
            Err(())
        }
        _ => {
            // Ignore empty document. root_nodes is empty here.
            Ok(root_nodes)
        }
    }
}

///
/// Check references of a node
///
///
fn check_node_references(
    diag: &mut Diagnostics,
    nodes: &BTreeMap<String, GsnNode>,
    excluded_modules: &[&str],
) -> Result<(), ()> {
    let results = nodes
        .iter()
        .filter(|(_, n)| !excluded_modules.contains(&n.module.as_str()))
        .flat_map(|(id, node)| {
            [
                node.in_context_of
                    .iter()
                    .filter(|&n| !nodes.contains_key(n))
                    .map(|wref| {
                        diag.add_error(
                            Some(&node.module),
                            format!("C03: Element {} has unresolved {}: {}", id, "context", wref),
                        );
                        Err(())
                    })
                    .collect::<Vec<Result<(), ()>>>(),
                node.supported_by
                    .iter()
                    .filter(|&n| !nodes.contains_key(n))
                    .map(|wref| {
                        diag.add_error(
                            Some(&node.module),
                            format!(
                                "C03: Element {} has unresolved {}: {}",
                                id, "supported by element", wref
                            ),
                        );
                        Err(())
                    })
                    .collect::<Vec<_>>(),
            ]
            .into_iter()
            .flatten()
        })
        .collect::<Vec<_>>();
    results
        .into_iter()
        .collect::<Result<Vec<_>, ()>>()
        .map(|_| ())
}

///
/// Check for cycles in `supported by` references
/// It also detects if there is a cycle in an independent graph.
///
///
fn check_cycles(
    diag: &mut Diagnostics,
    graph: &DirectedGraph<GsnNode, GsnEdgeType>,
) -> Result<(), ()> {
    let cycle = graph.get_first_cycle();
    if let Some((found, ring)) = cycle {
        diag.add_error(
            None,
            format!(
                "C04: Cycle detected at element {}. Cycle is {}.",
                found,
                &ring.join(" -> "),
            ),
        );
        Err(())
    } else {
        Ok(())
    }
}

///
///
///
fn check_unreachable(
    diag: &mut Diagnostics,
    graph: &DirectedGraph<GsnNode, GsnEdgeType>,
) -> Result<(), ()> {
    let unvisited = graph.get_unreachable_nodes();
    let root_nodes = graph.get_root_nodes();

    if unvisited.is_empty() {
        Ok(())
    } else {
        diag.add_error(
            None,
            format!(
                "C08: The following element(s) are not reachable from the root element(s) ({}): {}",
                root_nodes.join(", "),
                unvisited.join(", ")
            ),
        );
        Err(())
    }
}

///
/// Checks if the layers handed in via command line parameters
/// are actually used at at least one node.
/// Also checks if no reserved words are used, like 'rankIncrement' or 'text'
///
pub fn check_layers(
    diag: &mut Diagnostics,
    nodes: &BTreeMap<String, GsnNode>,
    layers: &[&str],
) -> Result<(), ()> {
    let reserved_words = [
        "text",
        "inContextOf",
        "supportedBy",
        "classes",
        "url",
        "undeveloped",
        "nodeType",
        "rankIncrement",
        "horizontalIndex",
        "wordWrap",
        "acp",
    ];
    let layer_results = layers
        .iter()
        .map(|l| {
            if reserved_words.contains(l) {
                diag.add_error(
                    None,
                    format!("{l} is a reserved attribute and cannot be used as layer."),
                );
                Err(())
            } else if !nodes
                .iter()
                .any(|(_, n)| n.additional.contains_key(l.to_owned()))
            {
                diag.add_warning(
                    None,
                    format!(
                        "Layer {l} is not used in file. No additional output will be generated."
                    ),
                );
                Ok(())
            } else {
                // All fine, check next layer
                Ok(())
            }
        })
        .collect::<Vec<_>>();
    layer_results
        .into_iter()
        .collect::<Result<Vec<_>, ()>>()
        .map(|_| ())
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;
    use crate::diagnostics::DiagType;

    #[test]
    fn unresolved_ref_context() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                in_context_of: vec!["C1".to_owned()],
                undeveloped: Some(true),
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        assert!(check_nodes(&mut d, &nodes, &[]).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "C03: Element G1 has unresolved context: C1"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn unresolved_ref_support() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["G2".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        assert!(check_nodes(&mut d, &nodes, &[]).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "C03: Element G1 has unresolved supported by element: G2"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn unreferenced_id() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        nodes.insert("C1".to_owned(), GsnNode::default());
        assert!(check_nodes(&mut d, &nodes, &[]).is_ok());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, None);
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[0].msg,
            "C01: There is more than one unreferenced element: C1, G1."
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
    }

    #[test]
    fn simple_cycle() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G0".to_owned(),
            GsnNode {
                supported_by: vec!["G1".to_owned()],
                ..Default::default()
            },
        );
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["G2".to_owned()],
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                supported_by: vec!["G1".to_owned()],
                ..Default::default()
            },
        );
        let edges: BTreeMap<String, Vec<(String, GsnEdgeType)>> = nodes
            .iter()
            .map(|(id, node)| (id.to_owned(), node.get_edges()))
            .collect();
        let graph = DirectedGraph::new(&nodes, &edges);
        assert!(check_cycles(&mut d, &graph).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, None);
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "C04: Cycle detected at element G2. Cycle is G1 -> G2 -> G1."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn simple_cycle_2() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["S1".to_owned()],
                ..Default::default()
            },
        );
        nodes.insert(
            "S1".to_owned(),
            GsnNode {
                supported_by: vec!["G2".to_owned()],
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                supported_by: vec!["G1".to_owned()],
                ..Default::default()
            },
        );
        assert!(check_nodes(&mut d, &nodes, &[]).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, None);
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "C01: There are no unreferenced elements found."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn diamond() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["S1".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "S1".to_owned(),
            GsnNode {
                supported_by: vec!["G2".to_owned(), "G3".to_owned()],
                node_type: Some(GsnNodeType::Strategy),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                undeveloped: Some(true),
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "G3".to_owned(),
            GsnNode {
                supported_by: vec!["G2".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        assert!(check_nodes(&mut d, &nodes, &[]).is_ok());
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn independent_cycle() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["S1".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "S1".to_owned(),
            GsnNode {
                supported_by: vec!["G2".to_owned()],
                node_type: Some(GsnNodeType::Strategy),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                supported_by: vec!["G1".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Solution),
                ..Default::default()
            },
        );
        nodes.insert(
            "G3".to_owned(),
            GsnNode {
                supported_by: vec!["Sn1".to_owned(), "G4".to_owned()],
                in_context_of: vec!["A1".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "G4".to_owned(),
            GsnNode {
                undeveloped: Some(true),
                in_context_of: vec!["J1".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "A1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Assumption),
                ..Default::default()
            },
        );
        nodes.insert(
            "J1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Justification),
                ..Default::default()
            },
        );
        assert!(check_nodes(&mut d, &nodes, &[]).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, None);
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "C08: The following element(s) are not reachable from the root element(s) (G3): G1, G2, S1"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn wrong_root() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Solution),
                ..Default::default()
            },
        );
        assert!(check_nodes(&mut d, &nodes, &[]).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, None);
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "C02: The root element should be a goal, but Sn1 was found."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn layer_exists() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();

        let mut admap = BTreeMap::new();
        admap.insert("layer1".to_owned(), "dontcare".to_owned());
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                additional: admap,
                ..Default::default()
            },
        );
        let res = check_layers(&mut d, &nodes, &["layer1"]);
        assert!(res.is_ok());
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn layer_does_not_exist() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();

        nodes.insert("Sn1".to_owned(), GsnNode::default());
        let res = check_layers(&mut d, &nodes, &["layer1"]);
        assert!(res.is_ok());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, None);
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[0].msg,
            "Layer layer1 is not used in file. No additional output will be generated."
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
    }

    #[test]
    fn only_one_layer_exists() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();

        let mut admap = BTreeMap::new();
        admap.insert("layer1".to_owned(), "dontcare".to_owned());
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                additional: admap,
                ..Default::default()
            },
        );
        let res = check_layers(&mut d, &nodes, &["layer1", "layer2"]);
        assert!(res.is_ok());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, None);
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[0].msg,
            "Layer layer2 is not used in file. No additional output will be generated."
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
    }

    #[test]
    fn layer_reserved() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();

        let mut admap = BTreeMap::new();
        admap.insert("layer1".to_owned(), "dontcare".to_owned());
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                additional: admap,
                ..Default::default()
            },
        );
        let res = check_layers(&mut d, &nodes, &["inContextOf", "layer2"]);
        assert!(res.is_err());
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, None);
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "inContextOf is a reserved attribute and cannot be used as layer."
        );
        assert_eq!(d.messages[1].module, None);
        assert_eq!(d.messages[1].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[1].msg,
            "Layer layer2 is not used in file. No additional output will be generated."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 1);
    }

    #[test]
    fn empty_document() {
        let mut d = Diagnostics::default();
        let nodes = BTreeMap::<String, GsnNode>::new();
        assert!(check_root_nodes(&mut d, &nodes).is_ok());
        assert_eq!(d.messages.len(), 0);
    }
}
