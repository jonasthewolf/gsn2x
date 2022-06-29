use super::GsnNode;
use crate::diagnostics::{DiagType, Diagnostics};
use std::collections::{BTreeMap, HashSet};

///
/// Validate all ids and nodes
///
///
pub fn validate_module(diag: &mut Diagnostics, module: &str, nodes: &BTreeMap<String, GsnNode>) {
    for (id, node) in nodes.iter().filter(|(_, n)| n.module == module) {
        // Validate if key is one of the known prefixes
        validate_id(diag, module, id);
        // Validate all references of node
        validate_references(diag, module, id, node);
    }
}

///
/// Validate id
///
/// Check if node id starts with a know prefix
///
fn validate_id(diag: &mut Diagnostics, module: &str, id: &str) {
    // Order is important due to Sn and S
    if !(id.starts_with("Sn")
        || id.starts_with('G')
        || id.starts_with('A')
        || id.starts_with('J')
        || id.starts_with('S')
        || id.starts_with('C'))
    {
        diag.add_msg(
            DiagType::Error,
            Some(module),
            format!(
                "V01: Element {} is of unknown type. Please see README for supported types",
                id
            ),
        );
    }
}

///
/// Validate all references
///
/// - Check in_context references for wellformedness
/// - Check supported_by references for wellformedness
/// - Check if undeveloped is correctly set
///
fn validate_references(diag: &mut Diagnostics, module: &str, id: &str, node: &GsnNode) {
    if let Some(in_context) = node.in_context_of.as_ref() {
        let mut valid_refs = vec![];
        // Only goals and strategies can have contexts, assumptions and goals
        if id.starts_with('S') || id.starts_with('G') {
            valid_refs.append(&mut vec!["J", "A", "C"]);
        }
        validate_reference(diag, module, id, in_context, "context", &valid_refs);
    }
    if let Some(support) = node.supported_by.as_ref() {
        let mut valid_refs = vec![];
        // Only goals and strategies can have other goals, strategies and solutions
        if id.starts_with('S') || id.starts_with('G') {
            valid_refs.append(&mut vec!["G", "Sn", "S"]);
        }
        validate_reference(
            diag,
            module,
            id,
            support,
            "supported by element",
            &valid_refs,
        );
        if Some(true) == node.undeveloped {
            diag.add_error(
                Some(module),
                format!("V03: Undeveloped element {} has supporting arguments.", id),
            );
        }
    } else if (id.starts_with('S') && !id.starts_with("Sn") || id.starts_with('G'))
        && (Some(false) == node.undeveloped || node.undeveloped.is_none())
    {
        // No "supported by" entries, but Strategy and Goal => undeveloped
        diag.add_warning(Some(module), format!("V02: Element {} is undeveloped.", id));
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
    node: &str,
    refs: &[String],
    diag_str: &str,
    valid_refs: &[&str],
) {
    // HashSet ok, since order is never important.
    let mut set = HashSet::with_capacity(refs.len());
    for n in refs {
        if n == node {
            diag.add_error(
                Some(module),
                format!("V06: Element {} references itself in {}.", node, diag_str),
            );
        }
        if !set.insert(n) {
            diag.add_warning(
                Some(module),
                format!(
                    "V05: Element {} has duplicate entry {} in {}.",
                    node, n, diag_str
                ),
            );
        }
        if !valid_refs.iter().any(|&r| n.starts_with(r)) {
            diag.add_error(
                Some(module),
                format!(
                    "V04: Element {} has invalid type of reference {} in {}.",
                    node, n, diag_str
                ),
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn unknown_id() {
        let mut d = Diagnostics::default();
        validate_id(&mut d, "", "X1");
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, Some("".to_owned()));
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "V01: Element X1 is of unknown type. Please see README for supported types"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn known_id() {
        let mut d = Diagnostics::default();
        validate_id(&mut d, "", "Sn1");
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_context() {
        let mut d = Diagnostics::default();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                in_context_of: Some(vec!["C1".to_owned()]),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &nodes);
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
                supported_by: Some(vec!["G1".to_owned()]),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &nodes);
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
                supported_by: Some(vec!["C1".to_owned()]),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &nodes);
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
                in_context_of: Some(vec!["G1".to_owned()]),
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &nodes);
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
                in_context_of: Some(vec!["C1".to_owned(), "C1".to_owned()]),
                supported_by: Some(vec!["Sn1".to_owned()]),
                ..Default::default()
            },
        );
        nodes.insert("Sn1".to_owned(), GsnNode::default());
        nodes.insert("C1".to_owned(), GsnNode::default());
        validate_module(&mut d, "", &nodes);
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
                supported_by: Some(vec!["G2".to_owned(), "G2".to_owned()]),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &nodes);
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
                in_context_of: Some(vec!["G2".to_owned(), "S1".to_owned(), "Sn1".to_owned()]),
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        nodes.insert(
            "S1".to_owned(),
            GsnNode {
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        nodes.insert("Sn1".to_owned(), GsnNode::default());
        validate_module(&mut d, "", &nodes);
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
                supported_by: Some(vec!["C1".to_owned(), "J1".to_owned(), "A1".to_owned()]),
                ..Default::default()
            },
        );
        nodes.insert("C1".to_owned(), GsnNode::default());
        nodes.insert("J1".to_owned(), GsnNode::default());
        nodes.insert("A1".to_owned(), GsnNode::default());
        validate_module(&mut d, "", &nodes);
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
        nodes.insert("G1".to_owned(), GsnNode::default());
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                undeveloped: Some(false),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &nodes);
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
        nodes.insert("S1".to_owned(), GsnNode::default());
        nodes.insert(
            "S2".to_owned(),
            GsnNode {
                undeveloped: Some(false),
                ..Default::default()
            },
        );
        validate_module(&mut d, "", &nodes);
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
                supported_by: Some(vec!["Sn2".to_owned()]),
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        nodes.insert("Sn2".to_owned(), GsnNode::default());
        validate_module(&mut d, "", &nodes);
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
}
