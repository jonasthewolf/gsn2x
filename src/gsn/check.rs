use super::GsnNode;
use crate::diagnostics::Diagnostics;
use crate::yaml_fix::MyMap;
use std::collections::{BTreeMap, HashSet};

///
///
///
pub fn check_nodes(
    diag: &mut Diagnostics,
    nodes: &MyMap<String, GsnNode>,
    excluded_modules: Option<Vec<&str>>,
) {
    check_node_references(diag, nodes, excluded_modules);
    check_root_nodes(diag, nodes)
        .map(|_| {
            check_levels(diag, nodes);
            check_cycles(diag, nodes);
        })
        .or::<()>(Ok(()))
        .unwrap();
}

///
/// Check if there is one and only one unreferenced node
/// and if it is a Goal
///
///
fn check_root_nodes(
    diag: &mut Diagnostics,
    nodes: &MyMap<String, GsnNode>,
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
        }
        x if x == 1 => {
            let rootn = root_nodes.get(0).unwrap();
            if !rootn.starts_with('G') {
                diag.add_error(
                    None,
                    format!(
                        "C02: The root element should be a goal, but {} was found.",
                        rootn
                    ),
                );
            }
        }
        x if x == 0 && nodes.len() > 0 => {
            diag.add_error(
                None,
                "C01: There are no unreferenced elements found.".to_owned(),
            );
        }
        _ => {
            // Ignore empty document.
        }
    }
    if diag.errors == 0 {
        Ok(root_nodes)
    } else {
        Err(())
    }
}

///
/// Check references of a node
///
///
fn check_node_references(
    diag: &mut Diagnostics,
    nodes: &MyMap<String, GsnNode>,
    excluded_modules: Option<Vec<&str>>,
) {
    let ex_mods = excluded_modules.iter().flatten().collect::<Vec<_>>();
    for (id, node) in nodes
        .iter()
        .filter(|(_, n)| !ex_mods.contains(&&n.module.as_str()))
    {
        if let Some(context) = node.in_context_of.as_ref() {
            context
                .iter()
                .filter(|&n| !nodes.contains_key(n))
                .for_each(|wref| {
                    diag.add_error(
                        Some(&node.module),
                        format!("C03: Element {} has unresolved {}: {}", id, "context", wref),
                    );
                });
        }
        if let Some(support) = node.supported_by.as_ref() {
            support
                .iter()
                .filter(|&n| !nodes.contains_key(n))
                .for_each(|wref| {
                    diag.add_error(
                        Some(&node.module),
                        format!(
                            "C03: Element {} has unresolved {}: {}",
                            id, "supported by element", wref
                        ),
                    );
                });
        }
    }
}

///
/// Check for cycles in `supported by` references
///
///
///
fn check_cycles(diag: &mut Diagnostics, nodes: &MyMap<String, GsnNode>) {
    let mut visited: HashSet<String> = HashSet::new();
    let mut root_nodes = super::get_root_nodes(nodes);
    let cloned_root_nodes = root_nodes.to_vec();
    let mut stack = Vec::new();

    for root in &root_nodes {
        visited.insert(root.to_owned());
    }
    stack.append(&mut root_nodes);
    while let Some(p_id) = stack.pop() {
        for child_node in nodes.get(&p_id).unwrap().supported_by.iter().flatten() {
            if !child_node.starts_with("Sn") {
                stack.push(child_node.to_owned());
                if !visited.insert(child_node.to_owned()) {
                    diag.add_error(
                        None,
                        format!("C04: Cycle detected at element {}.", child_node),
                    );
                    stack.clear();
                    break;
                }
            } else {
                // Remember the solutions for reachability analyis.
                visited.insert(child_node.to_owned());
            }
            // Also remember the incontext elements for the reachability analysis below.
            nodes
                .get(child_node)
                .unwrap()
                .in_context_of
                .iter()
                .flatten()
                .for_each(|x| {
                    visited.insert(x.to_owned());
                });
        }
    }
    let node_keys = HashSet::from_iter(nodes.keys().cloned());
    let unvisited: HashSet<&String> = node_keys.difference(&visited).collect();

    if !unvisited.is_empty() {
        diag.add_error(
            None,
            format!(
                "C08: The following element(s) are not reachable from the root element(s) ({}): {}",
                cloned_root_nodes.join(", "),
                Vec::from_iter(unvisited)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        );
    }
}

///
/// Check if level statement is used more than once.
///
///
///
fn check_levels(diag: &mut Diagnostics, nodes: &MyMap<String, GsnNode>) {
    let mut levels = BTreeMap::<&str, usize>::new();
    for node in nodes.values() {
        if let Some(l) = &node.level {
            *levels.entry(l.trim()).or_insert(0) += 1;
        }
    }
    levels
        .iter()
        .filter(|(_, &count)| count == 1)
        .for_each(|(l, _)| diag.add_warning(None, format!("C05: Level {} is only used once.", l)));
}

///
/// Checks if the layers handed in via command line parameters
/// are actually used at at least one node.
/// Also checks if no reserved words are used, like 'level' or 'text'
///
pub fn check_layers(diag: &mut Diagnostics, nodes: &MyMap<String, GsnNode>, layers: &[&str]) {
    let reserved_words = [
        "text",
        "inContextOf",
        "supportedBy",
        "classes",
        "url",
        "level",
        "undeveloped",
    ];
    for l in layers {
        if reserved_words.contains(l) {
            diag.add_error(
                None,
                format!("{} is a reserved attribute and cannot be used as layer.", l),
            );
            continue;
        }
        if !nodes
            .iter()
            .any(|(_, n)| n.additional.contains_key(l.to_owned()))
        {
            diag.add_warning(
                None,
                format!(
                    "Layer {} is not used in file. No additional output will be generated.",
                    l
                ),
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::diagnostics::DiagType;

    #[test]
    fn unresolved_ref_context() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                in_context_of: Some(vec!["C1".to_owned()]),
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        check_nodes(&mut d, &nodes, None);
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
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: Some(vec!["G2".to_owned()]),
                ..Default::default()
            },
        );
        check_nodes(&mut d, &nodes, None);
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
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        nodes.insert("C1".to_owned(), GsnNode::default());
        check_nodes(&mut d, &nodes, None);
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
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G0".to_owned(),
            GsnNode {
                supported_by: Some(vec!["G1".to_owned()]),
                ..Default::default()
            },
        );
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: Some(vec!["G2".to_owned()]),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                supported_by: Some(vec!["G1".to_owned()]),
                ..Default::default()
            },
        );
        check_cycles(&mut d, &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, None);
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(d.messages[0].msg, "C04: Cycle detected at element G1.");
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn simple_cycle_2() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: Some(vec!["S1".to_owned()]),
                ..Default::default()
            },
        );
        nodes.insert(
            "S1".to_owned(),
            GsnNode {
                supported_by: Some(vec!["G2".to_owned()]),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                supported_by: Some(vec!["G1".to_owned()]),
                ..Default::default()
            },
        );
        check_nodes(&mut d, &nodes, None);
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
    fn wrong_root() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert("Sn1".to_owned(), GsnNode::default());
        check_nodes(&mut d, &nodes, None);
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
        let mut nodes = MyMap::<String, GsnNode>::new();

        let mut admap = MyMap::new();
        admap.insert("layer1".to_owned(), "dontcare".to_owned());
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                additional: admap,
                ..Default::default()
            },
        );
        check_layers(&mut d, &nodes, &["layer1"]);
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn layer_does_not_exist() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();

        nodes.insert("Sn1".to_owned(), GsnNode::default());
        check_layers(&mut d, &nodes, &["layer1"]);
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
        let mut nodes = MyMap::<String, GsnNode>::new();

        let mut admap = MyMap::new();
        admap.insert("layer1".to_owned(), "dontcare".to_owned());
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                additional: admap,
                ..Default::default()
            },
        );
        check_layers(&mut d, &nodes, &["layer1", "layer2"]);
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
        let mut nodes = MyMap::<String, GsnNode>::new();

        let mut admap = MyMap::new();
        admap.insert("layer1".to_owned(), "dontcare".to_owned());
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                additional: admap,
                ..Default::default()
            },
        );
        check_layers(&mut d, &nodes, &["inContextOf", "layer2"]);
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
        let nodes = MyMap::<String, GsnNode>::new();
        assert!(check_root_nodes(&mut d, &nodes).is_ok());
        assert_eq!(d.messages.len(), 0);
    }
}
