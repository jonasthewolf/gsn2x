use crate::diagnostics::{DiagType, Diagnostics};
use crate::yaml_fix::MyMap;
use dirgraphsvg::edges::EdgeType;
use dirgraphsvg::nodes::{
    new_assumption, new_context, new_goal, new_justification, new_solution, new_strategy,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;

///
/// The main struct of this program
/// It describes a GSN element
///
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
    #[serde(skip_deserializing)]
    pub module: String,
}

impl GsnNode {
    pub fn get_edges(&self) -> Vec<(String, EdgeType)> {
        let mut edges = Vec::new();
        if let Some(c_nodes) = &self.in_context_of {
            let mut es: Vec<(String, EdgeType)> = c_nodes
                .iter()
                .map(|target| (target.to_owned(), EdgeType::InContextOf))
                .collect();
            edges.append(&mut es);
        }
        if let Some(s_nodes) = &self.supported_by {
            let mut es: Vec<(String, EdgeType)> = s_nodes
                .iter()
                .map(|target| (target.to_owned(), EdgeType::SupportedBy))
                .collect();
            edges.append(&mut es);
        }
        edges
    }
}

// TODO Add layer as class 
pub fn from_gsn_node(
    id: &str,
    gsn_node: &GsnNode,
) -> Rc<RefCell<dyn dirgraphsvg::nodes::Node>> {
    match id {
        id if id.starts_with('G') => new_goal(
            id,
            &gsn_node.text,
            gsn_node.undeveloped.unwrap_or(false),
            gsn_node.url.to_owned(),
            gsn_node.classes.to_owned(),
        ),
        id if id.starts_with("Sn") => new_solution(
            id,
            &gsn_node.text,
            gsn_node.url.to_owned(),
            gsn_node.classes.to_owned(),
        ),
        id if id.starts_with('S') => new_strategy(
            id,
            &gsn_node.text,
            gsn_node.undeveloped.unwrap_or(false),
            gsn_node.url.to_owned(),
            gsn_node.classes.to_owned(),
        ),
        id if id.starts_with('C') => new_context(
            id,
            &gsn_node.text,
            gsn_node.url.to_owned(),
            gsn_node.classes.to_owned(),
        ),
        id if id.starts_with('A') => new_assumption(
            id,
            &gsn_node.text,
            gsn_node.url.to_owned(),
            gsn_node.classes.to_owned(),
        ),
        id if id.starts_with('J') => new_justification(
            id,
            &gsn_node.text,
            gsn_node.url.to_owned(),
            gsn_node.classes.to_owned(),
        ),
        _ => unreachable!(),
    }
}

///
/// Validate all nodes
///
/// Check if key is one of the known prefixes
/// Check if all references of node exist
/// Check if there is more than one top-level node
///
///
pub fn validate_module(diag: &mut Diagnostics, module: &str, nodes: &MyMap<String, GsnNode>) {
    let mut wnodes: HashSet<String> = nodes.keys().cloned().collect();
    for (key, node) in nodes.deref() {
        if node.module == module {
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
        } else {
            wnodes.remove(key);
        }
    }
    match wnodes.len() {
        x if x > 1 => {
            let mut wn = wnodes.iter().cloned().collect::<Vec<String>>();
            wn.sort();
            diag.add_warning(
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
/// Validate all references
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
///
pub fn get_levels(nodes: &MyMap<String, GsnNode>) -> BTreeMap<&str, Vec<&str>> {
    let mut levels = BTreeMap::<&str, Vec<&str>>::new();
    for (id, node) in nodes.iter() {
        if let Some(l) = &node.level {
            levels
                .entry(l.trim())
                .or_insert(Vec::new())
                .push(id);
        }
    }
    levels
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

#[derive(Debug, PartialEq, Serialize)]
pub enum ModuleDependency {
    SupportedBy,
    InContextOf,
    Both,
}

///
/// Calculate module dependencies
///
///
pub fn calculate_module_dependencies(
    nodes: &MyMap<String, GsnNode>,
) -> BTreeMap<String, BTreeMap<String, ModuleDependency>> {
    let mut res = BTreeMap::<String, BTreeMap<String, ModuleDependency>>::new();
    for v in nodes.values() {
        res.insert(v.module.to_owned(), BTreeMap::new());
    }
    for v in nodes.deref().values() {
        if let Some(sups) = &v.supported_by {
            for sup in sups {
                let other_module = &nodes.get(sup).unwrap().module;
                if &v.module != other_module {
                    let e = res.get_mut(&v.module).unwrap();
                    e.entry(other_module.to_owned())
                        .and_modify(|x| {
                            if *x == ModuleDependency::InContextOf {
                                *x = ModuleDependency::Both
                            }
                        })
                        .or_insert(ModuleDependency::SupportedBy);
                }
            }
        }
        if let Some(ctxs) = &v.in_context_of {
            for ctx in ctxs {
                let other_module = &nodes.get(ctx).unwrap().module;
                if &v.module != other_module {
                    let e = res.get_mut(&v.module).unwrap();
                    e.entry(other_module.to_owned())
                        .and_modify(|x| {
                            if *x == ModuleDependency::SupportedBy {
                                *x = ModuleDependency::Both
                            }
                        })
                        .or_insert(ModuleDependency::InContextOf);
                }
            }
        }
    }
    res
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn unknown_id() {
        let mut d = Diagnostics::default();
        validate_id(&mut d, "", &"X1".to_owned());
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
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
        validate_id(&mut d, "", &"Sn1".to_owned());
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, "");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element C1 references itself in context."
        );
        assert_eq!(d.messages[1].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, "");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element C1 references itself in supported by element."
        );
        assert_eq!(d.messages[1].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, "");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element G1 references itself in context."
        );
        assert_eq!(d.messages[1].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[0].msg,
            "There is more than one unreferenced element: C1, G1."
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 3);
        assert_eq!(d.messages[0].module, "");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element G1 has invalid type of reference G2 in context."
        );
        assert_eq!(d.messages[1].module, "");
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "Element G1 has invalid type of reference S1 in context."
        );
        assert_eq!(d.messages[2].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 3);
        assert_eq!(d.messages[0].module, "");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "Element G1 has invalid type of reference C1 in supported by element."
        );
        assert_eq!(d.messages[1].module, "");
        assert_eq!(d.messages[1].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[1].msg,
            "Element G1 has invalid type of reference J1 in supported by element."
        );
        assert_eq!(d.messages[2].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 3);
        assert_eq!(d.messages[0].module, "");
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(d.messages[0].msg, "Element G1 is undeveloped.");
        assert_eq!(d.messages[1].module, "");
        assert_eq!(d.messages[1].diag_type, DiagType::Warning);
        assert_eq!(d.messages[1].msg, "Element G2 is undeveloped.");
        assert_eq!(d.messages[2].module, "");
        assert_eq!(d.messages[2].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[2].msg,
            "There is more than one unreferenced element: G1, G2."
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 3);
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 3);
        assert_eq!(d.messages[0].module, "");
        assert_eq!(d.messages[0].diag_type, DiagType::Warning);
        assert_eq!(d.messages[0].msg, "Element S1 is undeveloped.");
        assert_eq!(d.messages[1].module, "");
        assert_eq!(d.messages[1].diag_type, DiagType::Warning);
        assert_eq!(d.messages[1].msg, "Element S2 is undeveloped.");
        assert_eq!(d.messages[2].module, "");
        assert_eq!(d.messages[2].diag_type, DiagType::Warning);
        assert_eq!(
            d.messages[2].msg,
            "There is more than one unreferenced element: S1, S2."
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 3);
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
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
        validate_module(&mut d, "", &nodes);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
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
        check_layers(&mut d, "", &nodes, &["layer1"]);
        assert_eq!(d.messages.len(), 0);
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn layer_does_not_exist() {
        let mut d = Diagnostics::default();
        let mut nodes = MyMap::<String, GsnNode>::new();

        nodes.insert("Sn1".to_owned(), GsnNode::default());
        check_layers(&mut d, "", &nodes, &["layer1"]);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
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
        check_layers(&mut d, "", &nodes, &["layer1", "layer2"]);
        assert_eq!(d.messages.len(), 1);
        assert_eq!(d.messages[0].module, "");
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
        check_layers(&mut d, "", &nodes, &["inContextOf", "layer2"]);
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[0].module, "");
        assert_eq!(d.messages[0].diag_type, DiagType::Error);
        assert_eq!(
            d.messages[0].msg,
            "inContextOf is a reserved attribute and cannot be used as layer."
        );
        assert_eq!(d.messages[1].module, "");
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
        assert!(output.contains_key(&"x1"));
        assert!(output.contains_key(&"x2"));
    }
}
