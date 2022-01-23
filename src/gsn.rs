use crate::diagnostics::{DiagType, Diagnostics};
use crate::yaml_fix::MyMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::Deref;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GsnNode {
    text: String,
    in_context_of: Option<Vec<String>>,
    supported_by: Option<Vec<String>>,
    classes: Option<Vec<String>>,
    url: Option<String>,
    level: Option<String>,
    #[serde(flatten)]
    additional: MyMap<String, String>,
    undeveloped: Option<bool>,
}

impl GsnNode {
    pub(crate) fn get_text(&self) -> &str {
        &self.text
    }
    pub(crate) fn get_url(&self) -> &Option<String> {
        &self.url
    }
    pub(crate) fn get_layer(&self, layer: &str) -> Option<&String> {
        self.additional.get(&layer.to_owned())
    }
}

///
/// Validate all nodes
///
/// Check if key is one of the known prefixes
/// Check if all references of node exist
/// Check if there is more than one top-level node
///
/// Returns the number of validation warnings and errors
///
pub fn validate_module(diag: &mut Diagnostics, module: &str, nodes: &MyMap<String, GsnNode>) {
    let mut wnodes: HashSet<String> = nodes.keys().cloned().collect();
    for (key, node) in nodes.deref() {
        // Validate if key is one of the known prefixes
        validate_id(diag, module, key);
        // Validate if all references of node exist
        validate_reference(diag, module, nodes, key, node);
        // Remove all keys if they are referenced; used to see if there is more than one top level node
        if let Some(context) = node.in_context_of.as_ref() {
            for cnode in context {
                wnodes.remove(cnode);
            }
        }
        if let Some(support) = node.supported_by.as_ref() {
            for snode in support {
                wnodes.remove(snode);
            }
        }
    }
    match wnodes.len() {
        x if x > 1 => {
            let mut wn = wnodes.iter().cloned().collect::<Vec<String>>();
            wn.sort();
            diag.add_error(
                module,
                format!(
                    "There is more than one unreferenced element: {}.",
                    wn.join(", ")
                ),
            );
        }
        x if x == 1 => {
            let rootn = wnodes.iter().next().unwrap();
            if !rootn.starts_with('G') {
                diag.add_error(
                    module,
                    format!(
                        "The root element should be a goal, but {} was found.",
                        rootn
                    ),
                );
            }
        }
        _ => {
            // Ignore empty document.
        }
    }
}

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
            module,
            format!(
                "Elememt {} is of unknown type. Please see README for supported types",
                id
            ),
        );
    }
}

///
/// Check all references.
///
/// - Check if node does not reference itself.
/// - Check if all references exist.
/// - Check if a list of references only contains unique values.
/// - Check if a reference in the correct list i.e., inContextOf or supportedBy
///
fn check_references(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &MyMap<String, GsnNode>,
    node: &str,
    refs: &[String],
    diag_str: &str,
    valid_refs: &[&str],
) {
    let mut set = HashSet::with_capacity(refs.len());
    let wrong_refs: Vec<&String> = refs
        .iter()
        .filter(|&n| {
            let isself = n == node;
            if isself {
                diag.add_error(
                    module,
                    format!("Element {} references itself in {}.", node, diag_str),
                );
            }
            let doubled = !set.insert(n);
            if doubled {
                diag.add_warning(
                    module,
                    format!(
                        "Element {} has duplicate entry {} in {}.",
                        node, n, diag_str
                    ),
                );
            }
            let wellformed = valid_refs.iter().any(|&r| n.starts_with(r));
            if !wellformed {
                diag.add_error(
                    module,
                    format!(
                        "Element {} has invalid type of reference {} in {}.",
                        node, n, diag_str
                    ),
                );
            }
            !isself && !doubled && wellformed
        })
        .filter(|&n| !nodes.contains_key(n))
        .collect();
    for wref in wrong_refs {
        diag.add_error(
            module,
            format!("Element {} has unresolved {}: {}", node, diag_str, wref),
        );
    }
}

///
///
///
///
fn validate_reference(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &MyMap<String, GsnNode>,
    key: &str,
    node: &GsnNode,
) {
    if let Some(context) = node.in_context_of.as_ref() {
        let mut valid_refs = vec![];
        // Only goals and strategies can have contexts, assumptions and goals
        if key.starts_with('S') || key.starts_with('G') {
            valid_refs.append(&mut vec!["J", "A", "C"]);
        }
        check_references(diag, module, nodes, key, context, "context", &valid_refs);
    }
    if let Some(support) = node.supported_by.as_ref() {
        let mut valid_refs = vec![];
        // Only goals and strategies can have other goals, strategies and solutions
        if key.starts_with('S') || key.starts_with('G') {
            valid_refs.append(&mut vec!["G", "Sn", "S"]);
        }
        check_references(
            diag,
            module,
            nodes,
            key,
            support,
            "supported by element",
            &valid_refs,
        );
        if Some(true) == node.undeveloped {
            diag.add_error(
                module,
                format!("Undeveloped element {} has supporting arguments.", key),
            );
        }
    } else if (key.starts_with('S') && !key.starts_with("Sn") || key.starts_with('G'))
        && (Some(false) == node.undeveloped || node.undeveloped.is_none())
    {
        // No "supported by" entries, but Strategy and Goal => undeveloped
        diag.add_warning(module, format!("Element {} is undeveloped.", key));
    }
}

///
/// Gathers all different 'level' attributes from all nodes.
/// Levels are used to create "{rank=same; x; y; z;}" statements.
///
pub fn get_levels(nodes: &MyMap<String, GsnNode>) -> Vec<String> {
    let mut levels = HashSet::<String>::new();
    for (_, v) in nodes.iter() {
        if let Some(l) = &v.level {
            levels.insert(l.trim().to_owned());
        }
    }
    levels.into_iter().collect()
}

///
/// Checks if the layers handed in via command line parameters
/// are actually used at at least one node.
/// Also checks if no reserved words are used, like 'level' or 'text'
///
pub fn check_layers(
    diag: &mut Diagnostics,
    module: &str,
    nodes: &MyMap<String, GsnNode>,
    layers: &[&str],
) {
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
                module,
                format!("{} is a reserved attribute and cannot be used as layer.", l),
            );
            continue;
        }
        let mut found = false;
        for (_, n) in nodes.iter() {
            if n.additional.contains_key(l.to_owned()) {
                found = true;
                break;
            }
        }
        if !found {
            diag.add_warning(
                module,
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
    #[test]
    fn unknown_id() {
        let mut d = Diagnostics::default();
        validate_id(&mut d, "module1", &"X1".to_owned());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Elememt X1 is of unknown type. Please see README for supported types"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn known_id() {
        let mut d = Diagnostics::default();
        validate_id(&mut d, "module1", &"Sn1".to_owned());
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_context() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                in_context_of: Some(vec!["C1".to_owned()]),
                ..Default::default()
            },
        );
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element C1 references itself in context."
        );
        assert_eq!(d.messages[1].module, "module1");
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "Element C1 has invalid type of reference C1 in context."
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_support() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: Some(vec!["G1".to_owned()]),
                ..Default::default()
            },
        );
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element G1 references itself in supported by element."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_wrong_context() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                supported_by: Some(vec!["C1".to_owned()]),
                ..Default::default()
            },
        );
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element C1 references itself in supported by element."
        );
        assert_eq!(d.messages[1].module, "module1");
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "Element C1 has invalid type of reference C1 in supported by element."
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_wrong_support() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                in_context_of: Some(vec!["G1".to_owned()]),
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element G1 references itself in context."
        );
        assert_eq!(d.messages[1].module, "module1");
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "Element G1 has invalid type of reference G1 in context."
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
    }

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
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(d.messages[0].msg, "Element G1 has unresolved context: C1");
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
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element G1 has unresolved supported by element: G2"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn duplicate_ref_context() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
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
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[0].msg,
            "Element G1 has duplicate entry C1 in context."
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
    }

    #[test]
    fn duplicate_ref_support() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
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
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[0].msg,
            "Element G1 has duplicate entry G2 in supported by element."
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
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
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "There is more than one unreferenced element: C1, G1."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn wrong_ref_context() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
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
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 3);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element G1 has invalid type of reference G2 in context."
        );
        assert_eq!(d.messages[1].module, "module1");
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "Element G1 has invalid type of reference S1 in context."
        );
        assert_eq!(d.messages[2].module, "module1");
        assert_eq!(d.messages[2].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[2].msg,
            "Element G1 has invalid type of reference Sn1 in context."
        );
        assert_eq!(d.errors, 3);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn wrong_ref_support() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
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
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 3);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element G1 has invalid type of reference C1 in supported by element."
        );
        assert_eq!(d.messages[1].module, "module1");
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "Element G1 has invalid type of reference J1 in supported by element."
        );
        assert_eq!(d.messages[2].module, "module1");
        assert_eq!(d.messages[2].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[2].msg,
            "Element G1 has invalid type of reference A1 in supported by element."
        );
        assert_eq!(d.errors, 3);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn undeveloped_goal() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert("G1".to_owned(), GsnNode::default());
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                undeveloped: Some(false),
                ..Default::default()
            },
        );
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 3);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(d.messages[0].msg, "Element G1 is undeveloped.");
        assert_eq!(d.messages[1].module, "module1");
        assert_eq!(d.messages[1].diag_type, DiagType::Warning);
        assert_eq!(d.messages[1].msg, "Element G2 is undeveloped.");
        assert_eq!(d.messages[2].module, "module1");
        assert_eq!(d.messages[2].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[2].msg,
            "There is more than one unreferenced element: G1, G2."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 2);
    }

    #[test]
    fn undeveloped_strategy() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert("S1".to_owned(), GsnNode::default());
        nodes.insert(
            "S2".to_owned(),
            GsnNode {
                undeveloped: Some(false),
                ..Default::default()
            },
        );
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 3);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(d.messages[0].msg, "Element S1 is undeveloped.");
        assert_eq!(d.messages[1].module, "module1");
        assert_eq!(d.messages[1].diag_type, DiagType::Warning);
        assert_eq!(d.messages[1].msg, "Element S2 is undeveloped.");
        assert_eq!(d.messages[2].module, "module1");
        assert_eq!(d.messages[2].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[2].msg,
            "There is more than one unreferenced element: S1, S2."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 2);
    }

    #[test]
    fn wrong_undeveloped() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                supported_by: Some(vec!["Sn2".to_owned()]),
                undeveloped: Some(true),
                ..Default::default()
            },
        );
        nodes.insert("Sn2".to_owned(), GsnNode::default());
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Undeveloped element G1 has supporting arguments."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn wrong_root() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert("Sn1".to_owned(), GsnNode::default());
        validate_module(&mut d, "module1", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "The root element should be a goal, but Sn1 was found."
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
        check_layers(&mut d, "module1", &nodes, &["layer1"]);
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn layer_does_not_exist() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();

        nodes.insert("Sn1".to_owned(), GsnNode::default());
        check_layers(&mut d, "module1", &nodes, &["layer1"]);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
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
        check_layers(&mut d, "module1", &nodes, &["layer1", "layer2"]);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "module1");
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
        check_layers(&mut d, "module1", &nodes, &["inContextOf", "layer2"]);
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, "module1");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "inContextOf is a reserved attribute and cannot be used as layer."
        );
        assert_eq!(d.messages[1].module, "module1");
        assert_eq!(d.messages[1].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[1].msg,
            "Layer layer2 is not used in file. No additional output will be generated."
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 1);
    }

    #[test]
    fn no_level_exists() {
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert("Sn1".to_owned(), Default::default());
        let output = get_levels(&nodes);
        assert!(output.is_empty());
    }

    #[test]
    fn two_levels_exist() {
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                level: Some("x1".to_owned()),
                ..Default::default()
            },
        );
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                level: Some("x2".to_owned()),
                ..Default::default()
            },
        );
        let output = get_levels(&nodes);
        assert_eq!(output.len(), 2);
        assert!(output.contains(&"x1".to_owned()));
        assert!(output.contains(&"x2".to_owned()));
    }
}
