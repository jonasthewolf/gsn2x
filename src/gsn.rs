use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::io::Write;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GsnNode {
    text: String,
    pub(crate) in_context_of: Option<Vec<String>>,
    pub(crate) supported_by: Option<Vec<String>>,
}

pub fn validate(output: &mut impl Write , nodes: &BTreeMap<String, GsnNode>) {
    let mut wnodes: HashSet<String> = nodes.keys().cloned().collect();
    for (key, node) in nodes {
        // Validate if key is one of the known prefixes
        validate_id(output, key);
        // Validate if all references of node exist
        validate_reference(output, &nodes, key, &node);
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
    if wnodes.len() > 1 {
        writeln!(output,
            "Error: There is more than one unreferenced element: {}",
            wnodes.iter().map(|s| s.clone()).collect::<Vec<String>>().join(", ")
        ).unwrap();
    }
}

fn validate_id(output: &mut impl Write, id: &str) {
    // Order is important due to Sn and S
    if !(id.starts_with("Sn")
        || id.starts_with('G')
        || id.starts_with('A')
        || id.starts_with('J')
        || id.starts_with('S')
        || id.starts_with('C'))
    {
        writeln!(output,
            "Error: Elememt {} is of unknown type. Please see README for supported types",
            id
        ).unwrap();
    }
}

fn check_references(
    output: &mut impl Write,
    nodes: &BTreeMap<String, GsnNode>,
    node: &str,
    refs: &[String],
    diag: &str,
) {
    let mut set = HashSet::with_capacity(refs.len());
    let wrong_refs: Vec<&String> = refs
        .iter()
        .inspect(|&n| {
            if !set.insert(n) {
                writeln!(output,
                    "Warning: Element {} has duplicate entry {} in {}.",
                    node, n, diag
                ).unwrap();
            }
        })
        .filter(|&n| !nodes.contains_key(n))
        .collect();
    for wref in wrong_refs {
        writeln!(output, "Error: Element {} has unresolved {}: {}", node, diag, wref).unwrap();
    }
}

fn validate_reference(output: &mut impl Write, nodes: &BTreeMap<String, GsnNode>, key: &str, node: &GsnNode) {
    if let Some(context) = node.in_context_of.as_ref() {
        check_references(output, nodes, key, context, "context");
    }
    if let Some(support) = node.supported_by.as_ref() {
        check_references(output, nodes, key, support, "supported by element");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn unknown_id() {
        let mut output = Vec::<u8>::new();
        validate_id(&mut output, &"X1".to_owned());
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Elememt X1 is of unknown type. Please see README for supported types\n"
        );
    }

    #[test]
    fn known_id() {
        let mut output = Vec::<u8>::new();
        validate_id(&mut output, &"Sn1".to_owned());
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            ""
        );
    }

    #[test]
    fn unresolved_ref_context() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: Some(vec!["C1".to_owned()]),
                supported_by: None,
            },
        );
        validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Element G1 has unresolved context: C1\n"
        );
    }

    #[test]
    fn unresolved_ref_support() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["C1".to_owned()]),
            },
        );
        validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Element G1 has unresolved supported by element: C1\n"
        );
    }

    #[test]
    fn duplicate_ref_context() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: Some(vec!["C1".to_owned(), "C1".to_owned()]),
                supported_by: None,
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
            },
        );
        validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Warning: Element G1 has duplicate entry C1 in context.\n"
        );
    }

    #[test]
    fn duplicate_ref_support() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["C1".to_owned(), "C1".to_owned()]),
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
            },
        );
        validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Warning: Element G1 has duplicate entry C1 in supported by element.\n"
        );
    }

    #[test]
    fn unreferenced_id() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
            },
        );
        validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: There is more than one unreferenced element: G1, C1\n"
        );
    }
}
