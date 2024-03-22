use super::{get_node_type_from_text, GsnNode, GsnNodeType, Module};
use crate::diagnostics::{DiagType, Diagnostics};
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
) {
    for (id, node) in nodes.iter().filter(|(_, n)| n.module == module_name) {
        // Validate that type of node is known
        validate_type(diag, module_name, id, node);
        // Validate if id and type do not contradict
        validate_id(diag, module_name, id, node);
        // Validate all references of node
        validate_references(diag, module_name, nodes, id, node);
        // Validate all assurance claim points
        validate_assurance_claim_point(diag, module_name, nodes, id, node);
    }
    validate_module_extensions(module_info, nodes, module_name, diag);
}

///
/// Validate type
///
/// Check if the node has a type assigned.
/// The type is typically derived from the id, but can be overwritten by the `node_type` attribute.
///
fn validate_type(diag: &mut Diagnostics, module: &str, id: &str, node: &GsnNode) {
    if node.node_type.is_none() {
        diag.add_msg(
            DiagType::Error,
            Some(module),
            format!("V01: Element {id} is of unknown type. Please see documentation for supported types"),
        );
    }
}

///
/// Validate id
///
/// Check if node id starts with a know prefix
///
fn validate_id(diag: &mut Diagnostics, module: &str, id: &str, node: &GsnNode) {
    if let Some(type_from_id) = get_node_type_from_text(id) {
        if let Some(type_from_node) = node.node_type {
            if type_from_node != type_from_id {
                diag.add_msg(
                    DiagType::Warning,
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

///
/// Validate all references
///
/// - Check in_context references for well-formedness
/// - Check supported_by references for well-formedness
/// - Check if undeveloped is correctly set
///
fn validate_references(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &BTreeMap<String, GsnNode>,
    id: &str,
    node: &GsnNode,
) {
    if !node.in_context_of.is_empty() {
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
        );
    }
    if !node.supported_by.is_empty() {
        let mut valid_ref_types = vec![];
        // Only goals and strategies can have other goals, strategies and solutions
        if node.node_type == Some(GsnNodeType::Strategy)
            || node.node_type == Some(GsnNodeType::Goal)
        {
            valid_ref_types.append(&mut vec![
                GsnNodeType::Goal,
                GsnNodeType::Solution,
                GsnNodeType::Strategy,
            ]);
        }
        validate_reference(
            diag,
            module,
            nodes,
            id,
            &node.supported_by,
            "supported by element",
            &valid_ref_types,
        );
        if Some(true) == node.undeveloped {
            diag.add_error(
                Some(module),
                format!("V03: Undeveloped element {id} has supporting arguments."),
            );
        }
    } else if (id.starts_with('S') && !id.starts_with("Sn") || id.starts_with('G'))
        && (Some(false) == node.undeveloped || node.undeveloped.is_none())
    {
        // No "supported by" entries, but Strategy and Goal => undeveloped
        diag.add_warning(Some(module), format!("V02: Element {id} is undeveloped."));
    }
}

///
/// Validate references.
///
/// - Check if node does not reference itself.
/// - Check if a list of references only contains unique values.
/// - Check if a reference in the correct list i.e., inContextOf or supportedBy
///
fn validate_reference(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &BTreeMap<String, GsnNode>,
    node: &str,
    refs: &[String],
    diag_str: &str,
    valid_ref_types: &[GsnNodeType],
) {
    // HashSet ok, since order is never important.
    let mut set = HashSet::with_capacity(refs.len());
    for n in refs {
        if n == node {
            diag.add_error(
                Some(module),
                format!("V06: Element {node} references itself in {diag_str}."),
            );
        }
        if !set.insert(n) {
            diag.add_warning(
                Some(module),
                format!("V05: Element {node} has duplicate entry {n} in {diag_str}."),
            );
        }
        if let Some(ref_node) = nodes.get(n) {
            if !valid_ref_types
                .iter()
                .any(|&r| ref_node.node_type.unwrap() == r)
            {
                diag.add_error(
                    Some(module),
                    format!("V04: Element {node} has invalid type of reference {n} in {diag_str}."),
                );
            }
        }
    }
}

///
///
///
fn validate_assurance_claim_point(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &BTreeMap<String, GsnNode>,
    id: &str,
    node: &GsnNode,
) {
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
    for (acp, references) in &node.acp {
        for r in references {
            if !potential_references.contains(&r.as_str()) {
                diag.add_error(
                    Some(module),
                    format!("V09: Element {id} has an assurance claim point {acp} that references {r}, but this is neither its own ID nor any of the connected elements."),
                );
            }
        }
    }
}

///
/// Validate module extensions
///
///
///
fn validate_module_extensions(
    module_info: &Module,
    nodes: &BTreeMap<String, GsnNode>,
    module_name: &str,
    diag: &mut Diagnostics,
) {
    if let Some(extensions) = &module_info.meta.extends {
        for ext in extensions {
            for (foreign_id, local_ids) in &ext.develops {
                for local_id in local_ids {
                    if !(local_id.starts_with("Sn")
                        || local_id.starts_with('S')
                        || local_id.starts_with('G'))
                    {
                        diag.add_msg(
                            DiagType::Error,
                            Some(module_name),
                            format!(
                                "V07: Element {local_id} is of wrong type. Only Strategies, Goals and Solutions can develop other Goals and Strategies."
                            ),
                        );
                    } else if !nodes
                        .iter()
                        .filter(|(_, n)| n.module == module_name)
                        .any(|(id, _)| id == local_id)
                    {
                        diag.add_msg(
                            DiagType::Error,
                            Some(module_name),
                            format!(
                                "V07: Element {} in module {} supposed to develop {} in module {} does not exist.",
                                local_id,
                                module_name,
                                foreign_id,
                                ext.module
                            ),
                        );
                    } else {
                        // All fine.
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::gsn::{ExtendsModule, ModuleInformation};

    use super::*;
    #[test]
    fn unknown_id() {
        let mut d = Diagnostics::default();
        let mut node = GsnNode::default();
        node.fix_node_type("X1");
        validate_type(&mut d, "", "X1", &node);
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
    fn unknown_id_no_type() {
        // validate_id is not supposed to detect that situation
        let mut d = Diagnostics::default();
        let node = GsnNode::default();
        validate_id(&mut d, "", "X1", &node);
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
        validate_id(&mut d, "", "Sn1", &node);
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
        validate_id(&mut d, "", "Sn1", &node);
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
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &Module::default(), &nodes);
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
        validate_module(&mut d, "", &Module::default(), &nodes);
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
        validate_module(
            &mut d,
            "",
            &Module {
                relative_module_path: "".to_owned(),
                meta: ModuleInformation::default(),
            },
            &nodes,
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
        validate_module(&mut d, "", &Module::default(), &nodes);
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
                undeveloped: Some(true),
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &Module::default(), &nodes);
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
        validate_module(&mut d, "", &Module::default(), &nodes);
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
                undeveloped: Some(true),
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &Module::default(), &nodes);
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
                undeveloped: Some(true),
                node_type: Some(GsnNodeType::Goal),
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
            "S1".to_owned(),
            GsnNode {
                undeveloped: Some(true),
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
        validate_module(&mut d, "", &Module::default(), &nodes);
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
        validate_module(&mut d, "", &Module::default(), &nodes);
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
                undeveloped: Some(false),
                node_type: Some(GsnNodeType::Goal),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &Module::default(), &nodes);
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
                undeveloped: Some(false),
                node_type: Some(GsnNodeType::Strategy),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &Module::default(), &nodes);
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
                undeveloped: Some(true),
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
        validate_module(&mut d, "", &Module::default(), &nodes);
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
        validate_module(
            &mut d,
            "mod",
            &Module {
                relative_module_path: "mod".to_owned(),
                meta: ModuleInformation {
                    name: "mod".to_owned(),
                    brief: Some("brief".to_owned()),
                    extends: Some(vec![ExtendsModule {
                        module: "mod2".to_owned(),
                        develops,
                    }]),
                    horizontal_index: None,
                    rank_increment: None,
                    _word_wrap: None,
                    additional: BTreeMap::new(),
                },
            },
            &nodes,
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
        validate_module(
            &mut d,
            "",
            &Module {
                relative_module_path: "".to_owned(),
                meta: ModuleInformation {
                    name: "mod".to_owned(),
                    brief: Some("brief".to_owned()),
                    extends: Some(vec![ExtendsModule {
                        module: "mod2".to_owned(),
                        develops,
                    }]),
                    horizontal_index: None,
                    rank_increment: None,
                    _word_wrap: None,
                    additional: BTreeMap::new(),
                },
            },
            &nodes,
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
}
