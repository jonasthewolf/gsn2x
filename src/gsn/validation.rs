use super::{Challenge, GsnNode, GsnNodeType, Module, get_node_type_from_text};
use crate::diagnostics::Diagnostics;
use std::collections::{BTreeMap, HashSet};

///
/// Validate all ids and nodes
///
///
pub fn validate_module(
    diag: &mut Diagnostics,
    module_name: &str,
    module_info: &Module,
    nodes: &BTreeMap<String, GsnNode>,
    extended_check: bool,
    warn_dialectic: bool,
) -> Result<(), ()> {
    let all_results = nodes
        .iter()
        .filter(|(_, n)| n.module == module_name)
        .flat_map(|(id, node)| {
            [
                // Validate that type of node is known
                validate_type(diag, module_name, id, node),
                // Validate if id and type do not contradict
                validate_id(diag, module_name, id, node, extended_check),
                // Validate all references of node
                validate_references(diag, module_name, nodes, id, node),
                // Validate all assurance claim points
                validate_assurance_claim_point(diag, module_name, nodes, id, node),
                // Validate if defeated is correctly set
                validate_defeated(diag, module_name, nodes, id, node),
            ]
        })
        .collect::<Vec<Result<(), ()>>>();
    all_results
        .into_iter()
        .collect::<Result<Vec<_>, ()>>()
        .and_then(|_| validate_module_extensions(diag, module_name, nodes, module_info))
        .and_then(|_| validate_dialectic_extension(diag, module_name, nodes, warn_dialectic))
}

///
/// Validate type
///
/// Check if the node has a type assigned.
/// The type is typically derived from the id, but can be overwritten by the `node_type` attribute.
///
fn validate_type(diag: &mut Diagnostics, module: &str, id: &str, node: &GsnNode) -> Result<(), ()> {
    if node.node_type.is_none() {
        diag.add_error(
            Some(module),
            format!(
                "V01: Element {id} is of unknown type. Please see documentation for supported types"
            ),
        );
        Err(())
    } else {
        Ok(())
    }
}

///
/// Validate id
///
/// Check if node id starts with a know prefix
///
fn validate_id(
    diag: &mut Diagnostics,
    module: &str,
    id: &str,
    node: &GsnNode,
    extended_check: bool,
) -> Result<(), ()> {
    if extended_check {
        if let Some(type_from_id) = get_node_type_from_text(id) {
            if let Some(type_from_node) = node.node_type {
                if type_from_node != type_from_id {
                    diag.add_warning(
                        Some(module),
                        format!(
                            "V08: Element {} has type {}, but ID indicates type {}",
                            id, type_from_node, type_from_id
                        ),
                    );
                }
            }
        }
    }
    Ok(())
}

///
/// Validate all references
///
/// - Check in_context references for well-formedness
/// - Check supported_by references for well-formedness
/// - Check challenges references for well-formedness
/// - Check if undeveloped is correctly set
///
fn validate_references(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &BTreeMap<String, GsnNode>,
    id: &str,
    node: &GsnNode,
) -> Result<(), ()> {
    let incontext_res = if node.in_context_of.is_empty() {
        Ok(())
    } else {
        let mut valid_ref_types = vec![];
        // Only goals and strategies can have contexts, assumptions and justifications
        if node.node_type == Some(GsnNodeType::Strategy)
            || node.node_type == Some(GsnNodeType::Goal)
        {
            valid_ref_types.append(&mut vec![
                GsnNodeType::Justification,
                GsnNodeType::Assumption,
                GsnNodeType::Context,
            ]);
        }
        validate_reference(
            diag,
            module,
            nodes,
            id,
            &node.in_context_of,
            "context",
            &valid_ref_types,
        )
    };
    let supportedby_res = if !node.supported_by.is_empty() {
        let mut valid_ref_types = vec![];
        // Only goals and strategies can have other goals, strategies and solutions
        if node.node_type == Some(GsnNodeType::Strategy)
            || node.node_type == Some(GsnNodeType::Goal)
        {
            valid_ref_types.append(&mut vec![
                GsnNodeType::Goal,
                GsnNodeType::CounterGoal,
                GsnNodeType::Solution,
                GsnNodeType::CounterSolution,
                GsnNodeType::Strategy,
            ]);
        }
        if node.node_type == Some(GsnNodeType::CounterGoal) {
            valid_ref_types.append(&mut vec![
                GsnNodeType::CounterGoal,
                GsnNodeType::CounterSolution,
            ]);
        }

        let valid_refs = validate_reference(
            diag,
            module,
            nodes,
            id,
            &node.supported_by,
            "supported by element",
            &valid_ref_types,
        );
        let devundev = if node.undeveloped {
            diag.add_error(
                Some(module),
                format!("V03: Undeveloped element {id} has supporting arguments."),
            );
            Err(())
        } else {
            Ok(())
        };
        valid_refs.and(devundev)
    } else if (node.node_type == Some(GsnNodeType::Strategy)
        || node.node_type == Some(GsnNodeType::Goal))
        && !node.undeveloped
    {
        // No "supported by" entries, but Strategy and Goal => undeveloped
        diag.add_warning(Some(module), format!("V02: Element {id} is undeveloped."));
        Ok(())
    } else {
        Ok(())
    };
    let challenges_res = if node.challenges.is_none() {
        Ok(())
    } else if !(node.node_type == Some(GsnNodeType::CounterGoal)
        || node.node_type == Some(GsnNodeType::CounterSolution))
        && node.challenges.is_some()
    {
        diag.add_error(
            Some(module),
            format!(
                "V12: {id} is not a CounterGoal nor CounterSolution but challenges another element or relation."
            ),
        );
        Err(())
    } else {
        Ok(())
    };
    incontext_res.and(supportedby_res).and(challenges_res)
}

///
/// Validate references.
///
/// - Check if node does not reference itself.
/// - Check if a list of references only contains unique values.
/// - Check if a reference in the correct list i.e., inContextOf, challenges or supportedBy
///
fn validate_reference(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &BTreeMap<String, GsnNode>,
    node: &str,
    refs: &[String],
    diag_str: &str,
    valid_ref_types: &[GsnNodeType],
) -> Result<(), ()> {
    // HashSet ok, since order is never important.
    let mut set = HashSet::with_capacity(refs.len());
    let valid_references = refs
        .iter()
        .flat_map(|n| {
            [
                if n == node {
                    diag.add_error(
                        Some(module),
                        format!("V06: Element {node} references itself in {diag_str}."),
                    );
                    Err(())
                } else {
                    Ok(())
                },
                {
                    if !set.insert(n) {
                        diag.add_warning(
                            Some(module),
                            format!("V05: Element {node} has duplicate entry {n} in {diag_str}."),
                        );
                    }
                    if !valid_ref_types
                        .iter()
                        .any(|&r| nodes.get(n).map(|x| x.node_type == Some(r)).unwrap_or(true))
                    {
                        diag.add_error(
                    Some(module),
                    format!("V04: Element {node} has invalid type of reference {n} in {diag_str}."),
                );
                        Err(())
                    } else {
                        Ok(()) // Ok is correct, since not all references must exist yet. Checked by C03.
                    }
                },
            ]
        })
        .collect::<Vec<_>>();
    valid_references
        .into_iter()
        .collect::<Result<Vec<_>, ()>>()
        .map(|_| ())
}

///
/// Check if the Assurance Claim Points are referencing sensible
///
fn validate_assurance_claim_point(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &BTreeMap<String, GsnNode>,
    id: &str,
    node: &GsnNode,
) -> Result<(), ()> {
    let mut potential_references = vec![id];
    potential_references.extend(
        node.supported_by
            .iter()
            .filter(|&n| nodes.contains_key(n))
            .map(String::as_str),
    );
    potential_references.extend(
        node.in_context_of
            .iter()
            .filter(|&n| nodes.contains_key(n))
            .map(String::as_str),
    );
    let results = node.acp.iter().flat_map(|(acp, references)|
        references.iter().map(|r| {
            if !&potential_references.contains(&r.as_str()) {
                diag.add_error(
                    Some(module),
                    format!("V09: Element {id} has an assurance claim point {acp} that references {r}, but this is neither its own ID nor any of the connected elements."),
                );
                Err(())
            } else { Ok(()) }
        }).collect::<Vec<Result<(),()>>>()
    ).collect::<Vec<_>>();
    results
        .into_iter()
        .collect::<Result<Vec<_>, ()>>()
        .map(|_| ())
}

///
/// Validate module extensions
///
fn validate_module_extensions(
    diag: &mut Diagnostics,
    module_name: &str,
    nodes: &BTreeMap<String, GsnNode>,
    module_info: &Module,
) -> Result<(), ()> {
    let results = module_info.meta.extends.iter().flat_map(|ext| {
        ext.develops.iter().flat_map(|(foreign_id, local_ids)| {
            local_ids.iter().map(|local_id| {
                if !(local_id.starts_with("Sn")
                    || local_id.starts_with('S')
                    || local_id.starts_with('G'))
                {
                    diag.add_error(
                            Some(module_name),
                            format!(
                                "V07: Element {local_id} is of wrong type. Only Strategies, Goals and Solutions can develop other Goals and Strategies."
                            ),
                        );
                    Err(())
                } else if !nodes
                    .iter()
                    .filter(|(_, n)| n.module == module_name)
                    .any(|(id, _)| id == local_id)
                {
                    diag.add_error(
                            Some(module_name),
                            format!(
                                "V07: Element {} in module {} supposed to develop {} in module {} does not exist.",
                                local_id,
                                module_name,
                                foreign_id,
                                ext.module
                            ),
                        );
                    Err(())
                } else {
                    // All fine.
                    Ok(())
                }
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();
    results
        .into_iter()
        .collect::<Result<Vec<_>, ()>>()
        .map(|_| ())
}

///
/// Validate defeated property
///
/// A node can be defeated if there is a challenging node, or a defeated relation.
/// A defeated relation do not need to be challenged, thus, no need to check.
///
fn validate_defeated(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &BTreeMap<String, GsnNode>,
    id: &str,
    node: &GsnNode,
) -> Result<(), ()> {
    let node_defeated = if node.defeated
        && !nodes
            .iter()
            .any(|(_, n)| matches!(&n.challenges, Some(Challenge::Node(n)) if n == id))
    {
        // TODO Amend check: an incoming defeated relation would be good, too????
        diag.add_error(
            Some(module),
            format!("V10: Element {id} is marked as defeated, but no other element challenges it."),
        );
        Err(())
    } else {
        Ok(())
    };

    let rel_defeated = if node.defeated_relation.is_empty() {
        Ok(())
    } else {
        node.defeated_relation.iter().try_for_each(|rel| {
            if node.supported_by.contains(rel) || node.in_context_of.contains(rel) || node.challenges == Some(Challenge::Node(rel.to_owned())) {
                Ok(())
            } else {
                diag.add_error(
                    Some(module),
                    format!("V13: Relation from {id} to {rel} is marked as defeated, but {id} has no relation to {rel}."),
                );
                Err(())
            }
        } )
    };

    node_defeated.and(rel_defeated)
}

///
/// Perform check if dialectic extension is used.
///
fn validate_dialectic_extension(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &BTreeMap<String, GsnNode>,
    warn_dialectic: bool,
) -> Result<(), ()> {
    if warn_dialectic {
        let dialectic_nodes = nodes
            .iter()
            .filter_map(|(id, node)| {
                if node.node_type == Some(GsnNodeType::CounterGoal)
                    || node.node_type == Some(GsnNodeType::CounterSolution)
                {
                    Some(id.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if dialectic_nodes.is_empty() {
            Ok(())
        } else {
            diag.add_warning(
                Some(module),
                format!(
                    "V11: Dialectic extension is used. See elements: {}",
                    dialectic_nodes.join(", ")
                ),
            );
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        diagnostics::DiagType,
        gsn::{ExtendsModule, ModuleInformation},
    };

    use super::*;
    #[test]
    fn unknown_id() {
        let mut d = Diagnostics::default();
        let mut node = GsnNode::default();
        node.fix_node_type("X1");
        assert!(validate_type(&mut d, "", "X1", &node).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V01: Element X1 is of unknown type. Please see documentation for supported types"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn unknown_id_validation() {
        let mut d = Diagnostics::default();
        let node = GsnNode {
            node_type: Some(GsnNodeType::Goal),
            ..Default::default()
        };
        assert!(validate_id(&mut d, "", "X1", &node, true).is_ok());
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn unknown_id_no_type() {
        // validate_id is not supposed to detect that situation
        let mut d = Diagnostics::default();
        let node = GsnNode::default();
        assert!(validate_id(&mut d, "", "X1", &node, false).is_ok());
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn inconsistent_id_type() {
        let mut d = Diagnostics::default();
        let mut node = GsnNode {
            node_type: Some(GsnNodeType::Assumption),
            ..Default::default()
        };
        node.fix_node_type("Sn1");
        assert!(validate_id(&mut d, "", "Sn1", &node, true).is_ok());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[0].msg,
            "V08: Element Sn1 has type Assumption, but ID indicates type Solution"
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
    }

    #[test]
    fn known_id() {
        let mut d = Diagnostics::default();
        let node = GsnNode::default();
        assert!(validate_id(&mut d, "", "Sn1", &node, true).is_ok());
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn solution_with_supported() {
        let mut d = Diagnostics::default();

        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                supported_by: vec!["G2".to_owned()],
                node_type: Some(GsnNodeType::Solution),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Goal),
                undeveloped: true,
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, false, false).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V04: Element Sn1 has invalid type of reference G2 in supported by element."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_context() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                in_context_of: vec!["C1".to_owned()],
                node_type: Some(GsnNodeType::Context),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_err());
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V06: Element C1 references itself in context."
        );
        assert_eq!(d.messages[1].module, Some("".to_owned()));
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "V04: Element C1 has invalid type of reference C1 in context."
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_support() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["G1".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        assert!(
            validate_module(
                &mut d,
                "",
                &Module {
                    orig_file_name: "".to_owned(),
                    meta: ModuleInformation::default(),
                    origin: crate::gsn::Origin::CommandLine,
                    canonical_path: None,
                    output_path: None,
                },
                &nodes,
                true,
                true,
            )
            .is_err()
        );
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V06: Element G1 references itself in supported by element."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_wrong_context() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                supported_by: vec!["C1".to_owned()],
                node_type: Some(GsnNodeType::Context),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_err());
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V06: Element C1 references itself in supported by element."
        );
        assert_eq!(d.messages[1].module, Some("".to_owned()));
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "V04: Element C1 has invalid type of reference C1 in supported by element."
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_wrong_support() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                in_context_of: vec!["G1".to_owned()],
                undeveloped: true,
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_err());
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V06: Element G1 references itself in context."
        );
        assert_eq!(d.messages[1].module, Some("".to_owned()));
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "V04: Element G1 has invalid type of reference G1 in context."
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn duplicate_ref_context() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                in_context_of: vec!["C1".to_owned(), "C1".to_owned()],
                supported_by: vec!["Sn1".to_owned()],
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
            "C1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Context),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_ok());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[0].msg,
            "V05: Element G1 has duplicate entry C1 in context."
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
    }

    #[test]
    fn duplicate_ref_support() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["G2".to_owned(), "G2".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                undeveloped: true,
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_ok());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[0].msg,
            "V05: Element G1 has duplicate entry G2 in supported by element."
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
    }

    #[test]
    fn wrong_ref_context() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                in_context_of: vec!["G2".to_owned(), "S1".to_owned(), "Sn1".to_owned()],
                undeveloped: true,
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                undeveloped: true,
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "S1".to_owned(),
            GsnNode {
                undeveloped: true,
                node_type: Some(GsnNodeType::Strategy),
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
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_err());
        assert_eq!(d.messages.len(), 3);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V04: Element G1 has invalid type of reference G2 in context."
        );
        assert_eq!(d.messages[1].module, Some("".to_owned()));
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "V04: Element G1 has invalid type of reference S1 in context."
        );
        assert_eq!(d.messages[2].module, Some("".to_owned()));
        assert_eq!(d.messages[2].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[2].msg,
            "V04: Element G1 has invalid type of reference Sn1 in context."
        );

        assert_eq!(d.errors, 3);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn wrong_ref_support() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["C1".to_owned(), "J1".to_owned(), "A1".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Context),
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
        nodes.insert(
            "A1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Assumption),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_err());
        assert_eq!(d.messages.len(), 3);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V04: Element G1 has invalid type of reference C1 in supported by element."
        );
        assert_eq!(d.messages[1].module, Some("".to_owned()));
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "V04: Element G1 has invalid type of reference J1 in supported by element."
        );
        assert_eq!(d.messages[2].module, Some("".to_owned()));
        assert_eq!(d.messages[2].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[2].msg,
            "V04: Element G1 has invalid type of reference A1 in supported by element."
        );
        assert_eq!(d.errors, 3);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn wrong_challenging() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                in_context_of: vec!["C1".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                undeveloped: true,
                ..Default::default()
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Context),
                challenges: Some(Challenge::Node("Sn1".to_owned())),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V12: C1 is not a CounterGoal nor CounterSolution but challenges another element or relation."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn defeated_unchallenged() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Goal),
                undeveloped: true,
                defeated: true,
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V10: Element G1 is marked as defeated, but no other element challenges it."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn defeated_unrelated() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Goal),
                undeveloped: true,
                defeated_relation: vec!["G2".to_owned()],
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Goal),
                undeveloped: true,
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V13: Relation from G1 to G2 is marked as defeated, but G1 has no relation to G2."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn warn_dialectic() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Goal),
                undeveloped: true,
                defeated: true,
                ..Default::default()
            },
        );
        nodes.insert(
            "CG1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::CounterGoal),
                undeveloped: true,
                challenges: Some(crate::gsn::Challenge::Node("G1".to_owned())),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_ok());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[0].msg,
            "V11: Dialectic extension is used. See elements: CG1"
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
    }

    #[test]
    fn unknown_ref() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["Sn2".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_ok());
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn undeveloped_goal() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                undeveloped: false,
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_ok());
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(d.messages[0].msg, "V02: Element G1 is undeveloped.");
        assert_eq!(d.messages[1].module, Some("".to_owned()));
        assert_eq!(d.messages[1].diag_type, DiagType::Warning);
        assert_eq!(d.messages[1].msg, "V02: Element G2 is undeveloped.");
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 2);
    }

    #[test]
    fn undeveloped_strategy() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "S1".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Strategy),
                ..Default::default()
            },
        );
        nodes.insert(
            "S2".to_owned(),
            GsnNode {
                undeveloped: false,
                node_type: Some(GsnNodeType::Strategy),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_ok());
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(d.messages[0].msg, "V02: Element S1 is undeveloped.");
        assert_eq!(d.messages[1].module, Some("".to_owned()));
        assert_eq!(d.messages[1].diag_type, DiagType::Warning);
        assert_eq!(d.messages[1].msg, "V02: Element S2 is undeveloped.");
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 2);
    }

    #[test]
    fn wrong_undeveloped() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["Sn2".to_owned()],
                undeveloped: true,
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        nodes.insert(
            "Sn2".to_owned(),
            GsnNode {
                node_type: Some(GsnNodeType::Solution),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V03: Undeveloped element G1 has supporting arguments."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn wrong_extension() {
        let mut d = Diagnostics::default();
        let nodes = BTreeMap::<String, GsnNode>::new();
        let mut develops = BTreeMap::new();
        develops.insert("G1".to_owned(), vec!["G2".to_owned()]);
        assert!(
            validate_module(
                &mut d,
                "mod",
                &Module {
                    orig_file_name: "mod".to_owned(),
                    meta: ModuleInformation {
                        name: "mod".to_owned(),
                        brief: Some("brief".to_owned()),
                        extends: vec![ExtendsModule {
                            module: "mod2".to_owned(),
                            develops,
                        }],
                        uses: vec![],
                        stylesheets: vec![],
                        horizontal_index: None,
                        rank_increment: None,
                        char_wrap: None,
                        additional: BTreeMap::new(),
                    },
                    origin: crate::gsn::Origin::CommandLine,
                    canonical_path: None,
                    output_path: None,
                },
                &nodes,
                true,
                true,
            )
            .is_err()
        );
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("mod".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V07: Element G2 in module mod supposed to develop G1 in module mod2 does not exist."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn wrong_extension_type() {
        let mut d = Diagnostics::default();
        let nodes = BTreeMap::<String, GsnNode>::new();
        let mut develops = BTreeMap::new();
        develops.insert("G1".to_owned(), vec!["X2".to_owned()]);
        assert!(
            validate_module(
                &mut d,
                "",
                &Module {
                    orig_file_name: "".to_owned(),
                    meta: ModuleInformation {
                        name: "mod".to_owned(),
                        brief: Some("brief".to_owned()),
                        extends: vec![ExtendsModule {
                            module: "mod2".to_owned(),
                            develops,
                        }],
                        uses: vec![],
                        stylesheets: vec![],
                        horizontal_index: None,
                        rank_increment: None,
                        char_wrap: None,
                        additional: BTreeMap::new(),
                    },
                    origin: crate::gsn::Origin::CommandLine,
                    canonical_path: None,
                    output_path: None,
                },
                &nodes,
                true,
                true,
            )
            .is_err()
        );
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V07: Element X2 is of wrong type. Only Strategies, Goals and Solutions can develop other Goals and Strategies."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn invalid_acp_ref() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: vec!["G2".to_owned()],
                node_type: Some(GsnNodeType::Goal),
                acp: BTreeMap::from([("ACP1".to_owned(), vec!["G3".to_owned()])]),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                undeveloped: true,
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        assert!(validate_module(&mut d, "", &Module::default(), &nodes, true, true).is_err());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V09: Element G1 has an assurance claim point ACP1 that references G3, but this is neither its own ID nor any of the connected elements."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }
}
