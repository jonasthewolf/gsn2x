use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::ops::AddAssign;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GsnNode {
    text: String,
    in_context_of: Option<Vec<String>>,
    supported_by: Option<Vec<String>>,
    url: Option<String>,
}

#[derive(Debug, Default)]
pub struct Diagnostics {
    pub warnings: usize,
    pub errors: usize,
}

impl AddAssign for Diagnostics {
    fn add_assign(&mut self, rhs: Diagnostics) {
        self.warnings += rhs.warnings;
        self.errors += rhs.errors;
    }
}

/// Returns the number of validation warnings and errors
pub fn validate(output: &mut impl Write, nodes: &BTreeMap<String, GsnNode>) -> Diagnostics {
    let mut wnodes: HashSet<String> = nodes.keys().cloned().collect();
    let mut diag = Diagnostics::default();
    for (key, node) in nodes {
        // Validate if key is one of the known prefixes
        diag += validate_id(output, key);
        // Validate if all references of node exist
        diag += validate_reference(output, &nodes, key, &node);
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
        let mut wn = wnodes.iter().cloned().collect::<Vec<String>>();
        wn.sort();
        writeln!(
            output,
            "Error: There is more than one unreferenced element: {}",
            wn.join(", ")
        )
        .unwrap();
        diag.errors += 1;
    }
    diag
}

fn validate_id(output: &mut impl Write, id: &str) -> Diagnostics {
    // Order is important due to Sn and S
    if !(id.starts_with("Sn")
        || id.starts_with('G')
        || id.starts_with('A')
        || id.starts_with('J')
        || id.starts_with('S')
        || id.starts_with('C'))
    {
        writeln!(
            output,
            "Error: Elememt {} is of unknown type. Please see README for supported types",
            id
        )
        .unwrap();
        Diagnostics {
            warnings: 0,
            errors: 1,
        }
    } else {
        Diagnostics {
            warnings: 0,
            errors: 0,
        }
    }
}

fn check_references(
    mut output: &mut impl Write,
    nodes: &BTreeMap<String, GsnNode>,
    node: &str,
    refs: &[String],
    diag_str: &str,
    valid_refs: &[&str],
) -> Diagnostics {
    let mut diag = Diagnostics::default();
    let mut set = HashSet::with_capacity(refs.len());
    let wrong_refs: Vec<&String> = refs
        .iter()
        .filter(|&n| {
            let isself = n == node;
            if isself {
                writeln!(
                    &mut output,
                    "Error: Element {} references itself in {}.",
                    node, diag_str
                )
                .unwrap();
                diag.errors += 1;
            }
            let doubled = !set.insert(n);
            if doubled {
                writeln!(
                    &mut output,
                    "Warning: Element {} has duplicate entry {} in {}.",
                    node, n, diag_str
                )
                .unwrap();
                diag.warnings += 1;
            }
            let wellformed = valid_refs.iter().any(|&r| n.starts_with(r));
            if !wellformed {
                writeln!(
                    &mut output,
                    "Error: Element {} has invalid type of reference {} in {}.",
                    node, n, diag_str
                )
                .unwrap();
                diag.errors += 1;
            }
            !isself && !doubled && wellformed
        })
        .filter(|&n| !nodes.contains_key(n))
        .collect();
    for wref in wrong_refs {
        writeln!(
            &mut output,
            "Error: Element {} has unresolved {}: {}",
            node, diag_str, wref
        )
        .unwrap();
        diag.errors += 1;
    }
    diag
}

fn validate_reference(
    output: &mut impl Write,
    nodes: &BTreeMap<String, GsnNode>,
    key: &str,
    node: &GsnNode,
) -> Diagnostics {
    let mut diag = Diagnostics::default();
    if let Some(context) = node.in_context_of.as_ref() {
        let valid_refs = vec!["J", "A", "C"];
        diag += check_references(output, nodes, key, context, "context", &valid_refs);
    }
    if let Some(support) = node.supported_by.as_ref() {
        let valid_refs = vec!["G", "Sn", "S"];
        diag += check_references(
            output,
            nodes,
            key,
            support,
            "supported by element",
            &valid_refs,
        );
    }
    diag
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn unknown_id() {
        let mut output = Vec::<u8>::new();
        let d = validate_id(&mut output, &"X1".to_owned());
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Elememt X1 is of unknown type. Please see README for supported types\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn known_id() {
        let mut output = Vec::<u8>::new();
        let d = validate_id(&mut output, &"Sn1".to_owned());
        assert_eq!(std::str::from_utf8(&output).unwrap(), "");
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_context() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: Some(vec!["C1".to_owned()]),
                supported_by: None,
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Element C1 references itself in context.\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_support() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["G1".to_owned()]),
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Element G1 references itself in supported by element.\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_wrong_context() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["C1".to_owned()]),
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            concat!(
                "Error: Element C1 references itself in supported by element.\n",
                "Error: Element C1 has invalid type of reference C1 in supported by element.\n"
            )
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn self_ref_wrong_support() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: Some(vec!["G1".to_owned()]),
                supported_by: None,
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            concat!(
                "Error: Element G1 references itself in context.\n",
                "Error: Element G1 has invalid type of reference G1 in context.\n"
            )
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
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
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Element G1 has unresolved context: C1\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
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
                supported_by: Some(vec!["G2".to_owned()]),
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Element G1 has unresolved supported by element: G2\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
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
                url: None,
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Warning: Element G1 has duplicate entry C1 in context.\n"
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
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
                supported_by: Some(vec!["G2".to_owned(), "G2".to_owned()]),
                url: None,
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Warning: Element G1 has duplicate entry G2 in supported by element.\n"
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
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
                url: None,
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: There is more than one unreferenced element: C1, G1\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn wrong_ref_context() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: Some(vec!["G2".to_owned(), "S1".to_owned(), "Sn1".to_owned()]),
                supported_by: None,
                url: None,
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
            },
        );
        nodes.insert(
            "S1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
            },
        );
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            concat!(
                "Error: Element G1 has invalid type of reference G2 in context.\n",
                "Error: Element G1 has invalid type of reference S1 in context.\n",
                "Error: Element G1 has invalid type of reference Sn1 in context.\n"
            )
        );
        assert_eq!(d.errors, 3);
        assert_eq!(d.warnings, 0);
    }

    #[test]
    fn wrong_ref_support() {
        let mut output = Vec::<u8>::new();
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["C1".to_owned(), "J1".to_owned(), "A1".to_owned()]),
                url: None,
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
            },
        );
        nodes.insert(
            "J1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
            },
        );
        nodes.insert(
            "A1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
            },
        );
        let d = validate(&mut output, &nodes);
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            concat!(
                "Error: Element G1 has invalid type of reference C1 in supported by element.\n",
                "Error: Element G1 has invalid type of reference J1 in supported by element.\n",
                "Error: Element G1 has invalid type of reference A1 in supported by element.\n"
            )
        );
        assert_eq!(d.errors, 3);
        assert_eq!(d.warnings, 0);
    }
}
