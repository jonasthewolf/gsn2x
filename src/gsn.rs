use crate::yaml_fix::MyMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;
use std::ops::{AddAssign, Deref};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GsnNode {
    text: String,
    in_context_of: Option<Vec<String>>,
    supported_by: Option<Vec<String>>,
    url: Option<String>,
    undeveloped: Option<bool>,
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
pub fn validate(output: &mut impl Write, nodes: &MyMap<String, GsnNode>) -> Result<Diagnostics> {
    let mut wnodes: HashSet<String> = nodes.keys().cloned().collect();
    let mut diag = Diagnostics::default();
    for (key, node) in nodes.deref() {
        // Validate if key is one of the known prefixes
        diag += validate_id(output, &key)?;
        // Validate if all references of node exist
        diag += validate_reference(output, &nodes, &key, &node)?;
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
            writeln!(
                output,
                "Error: There is more than one unreferenced element: {}.",
                wn.join(", ")
            )?;
            diag.errors += 1;
        }
        x if x == 1 => {
            let rootn = wnodes.iter().next().unwrap();
            if !rootn.starts_with('G') {
                writeln!(
                    output,
                    "Error: The root element should be a goal, but {} was found.",
                    rootn
                )?;
                diag.errors += 1;
            }
        }
        _ => {
            // Ignore empty document.
        }
    }

    Ok(diag)
}

fn validate_id(output: &mut impl Write, id: &str) -> Result<Diagnostics> {
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
        )?;
        Ok(Diagnostics {
            warnings: 0,
            errors: 1,
        })
    } else {
        Ok(Diagnostics {
            warnings: 0,
            errors: 0,
        })
    }
}

fn check_references(
    mut output: &mut impl Write,
    nodes: &MyMap<String, GsnNode>,
    node: &str,
    refs: &[String],
    diag_str: &str,
    valid_refs: &[&str],
) -> Result<Diagnostics> {
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
    Ok(diag)
}

fn validate_reference(
    output: &mut impl Write,
    nodes: &MyMap<String, GsnNode>,
    key: &str,
    node: &GsnNode,
) -> Result<Diagnostics> {
    let mut diag = Diagnostics::default();
    if let Some(context) = node.in_context_of.as_ref() {
        let mut valid_refs = vec![];
        // Only goals and strategies can have contexts, assumptions and goals
        if key.starts_with('S') || key.starts_with('G') {
            valid_refs.append(&mut vec!["J", "A", "C"]);
        }
        diag += check_references(output, nodes, key, context, "context", &valid_refs)?;
    }
    if let Some(support) = node.supported_by.as_ref() {
        let mut valid_refs = vec![];
        // Only goals and strategies can have other goals, strategies and solutions
        if key.starts_with('S') || key.starts_with('G') {
            valid_refs.append(&mut vec!["G", "Sn", "S"]);
        }
        diag += check_references(
            output,
            nodes,
            key,
            support,
            "supported by element",
            &valid_refs,
        )?;
        if Some(true) == node.undeveloped {
            writeln!(
                output,
                "Error: Undeveloped element {} has supporting arguments.",
                key
            )?;
            diag += Diagnostics {
                errors: 1,
                warnings: 0,
            };
        }
    } else if (key.starts_with('S') && !key.starts_with("Sn") || key.starts_with('G'))
        && (Some(false) == node.undeveloped || node.undeveloped.is_none())
    {
        // No "supported by" entries, but Strategy and Goal => undeveloped
        writeln!(output, "Warning: Element {} is undeveloped.", key)?;
        diag += Diagnostics {
            errors: 0,
            warnings: 1,
        };
    }
    Ok(diag)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn unknown_id() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let d = validate_id(&mut output, &"X1".to_owned())?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Elememt X1 is of unknown type. Please see README for supported types\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
        Ok(())
    }

    #[test]
    fn known_id() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let d = validate_id(&mut output, &"Sn1".to_owned())?;
        assert_eq!(std::str::from_utf8(&output).unwrap(), "");
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 0);
        Ok(())
    }

    #[test]
    fn self_ref_context() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: Some(vec!["C1".to_owned()]),
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Element C1 references itself in context.\nError: Element C1 has invalid type of reference C1 in context.\n"
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
        Ok(())
    }

    #[test]
    fn self_ref_support() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["G1".to_owned()]),
                url: None,
                undeveloped: None,
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Element G1 references itself in supported by element.\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
        Ok(())
    }

    #[test]
    fn self_ref_wrong_context() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["C1".to_owned()]),
                url: None,
                undeveloped: None,
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            concat!(
                "Error: Element C1 references itself in supported by element.\n",
                "Error: Element C1 has invalid type of reference C1 in supported by element.\n"
            )
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
        Ok(())
    }

    #[test]
    fn self_ref_wrong_support() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: Some(vec!["G1".to_owned()]),
                supported_by: None,
                url: None,
                undeveloped: Some(true),
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            concat!(
                "Error: Element G1 references itself in context.\n",
                "Error: Element G1 has invalid type of reference G1 in context.\n"
            )
        );
        assert_eq!(d.errors, 2);
        assert_eq!(d.warnings, 0);
        Ok(())
    }

    #[test]
    fn unresolved_ref_context() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: Some(vec!["C1".to_owned()]),
                supported_by: None,
                url: None,
                undeveloped: Some(true),
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Element G1 has unresolved context: C1\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
        Ok(())
    }

    #[test]
    fn unresolved_ref_support() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["G2".to_owned()]),
                url: None,
                undeveloped: None,
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Element G1 has unresolved supported by element: G2\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
        Ok(())
    }

    #[test]
    fn duplicate_ref_context() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: Some(vec!["C1".to_owned(), "C1".to_owned()]),
                supported_by: Some(vec!["Sn1".to_owned()]),
                url: None,
                undeveloped: None,
            },
        );
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Warning: Element G1 has duplicate entry C1 in context.\n"
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
        Ok(())
    }

    #[test]
    fn duplicate_ref_support() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["G2".to_owned(), "G2".to_owned()]),
                url: None,
                undeveloped: None,
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: Some(true),
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Warning: Element G1 has duplicate entry G2 in supported by element.\n"
        );
        assert_eq!(d.errors, 0);
        assert_eq!(d.warnings, 1);
        Ok(())
    }

    #[test]
    fn unreferenced_id() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: Some(true),
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: There is more than one unreferenced element: C1, G1.\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
        Ok(())
    }

    #[test]
    fn wrong_ref_context() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: Some(vec!["G2".to_owned(), "S1".to_owned(), "Sn1".to_owned()]),
                supported_by: None,
                url: None,
                undeveloped: Some(true),
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: Some(true),
            },
        );
        nodes.insert(
            "S1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: Some(true),
            },
        );
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        let d = validate(&mut output, &nodes)?;
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
        Ok(())
    }

    #[test]
    fn wrong_ref_support() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["C1".to_owned(), "J1".to_owned(), "A1".to_owned()]),
                url: None,
                undeveloped: None,
            },
        );
        nodes.insert(
            "C1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        nodes.insert(
            "J1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        nodes.insert(
            "A1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        let d = validate(&mut output, &nodes)?;
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
        Ok(())
    }

    #[test]
    fn undeveloped_goal() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        nodes.insert(
            "G2".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: Some(false),
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Warning: Element G1 is undeveloped.\nWarning: Element G2 is undeveloped.\nError: There is more than one unreferenced element: G1, G2.\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 2);
        Ok(())
    }

    #[test]
    fn undeveloped_strategy() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "S1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        nodes.insert(
            "S2".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: Some(false),
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Warning: Element S1 is undeveloped.\nWarning: Element S2 is undeveloped.\nError: There is more than one unreferenced element: S1, S2.\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 2);
        Ok(())
    }

    #[test]
    fn wrong_undeveloped() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "G1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: Some(vec!["Sn2".to_owned()]),
                url: None,
                undeveloped: Some(true),
            },
        );
        nodes.insert(
            "Sn2".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: Undeveloped element G1 has supporting arguments.\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
        Ok(())
    }

    #[test]
    fn wrong_root() -> Result<()> {
        let mut output = Vec::<u8>::new();
        let mut nodes = MyMap::<String, GsnNode>::new();
        nodes.insert(
            "Sn1".to_owned(),
            GsnNode {
                text: "".to_owned(),
                in_context_of: None,
                supported_by: None,
                url: None,
                undeveloped: None,
            },
        );
        let d = validate(&mut output, &nodes)?;
        assert_eq!(
            std::str::from_utf8(&output).unwrap(),
            "Error: The root element should be a goal, but Sn1 was found.\n"
        );
        assert_eq!(d.errors, 1);
        assert_eq!(d.warnings, 0);
        Ok(())
    }
}
