use crate::dirgraphsvg::edges::{EdgeType, SingleEdge};
use crate::yaml_fix::MyMap;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

pub mod check;
pub mod validation;

///
/// The main struct of this program
/// It describes a GSN element
///
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GsnNode {
    pub(crate) text: String,
    pub(crate) in_context_of: Option<Vec<String>>,
    pub(crate) supported_by: Option<Vec<String>>,
    pub(crate) undeveloped: Option<bool>,
    pub(crate) classes: Option<Vec<String>>,
    pub(crate) url: Option<String>,
    pub(crate) level: Option<String>,
    #[serde(flatten)]
    pub(crate) additional: MyMap<String, String>,
    #[serde(skip_deserializing)]
    pub(crate) module: String,
}

impl GsnNode {
    pub fn get_edges(&self) -> Vec<(String, EdgeType)> {
        let mut edges = Vec::new();
        if let Some(c_nodes) = &self.in_context_of {
            let mut es: Vec<(String, EdgeType)> = c_nodes
                .iter()
                .map(|target| (target.to_owned(), EdgeType::OneWay(SingleEdge::InContextOf)))
                .collect();
            edges.append(&mut es);
        }
        if let Some(s_nodes) = &self.supported_by {
            let mut es: Vec<(String, EdgeType)> = s_nodes
                .iter()
                .map(|target| (target.to_owned(), EdgeType::OneWay(SingleEdge::SupportedBy)))
                .collect();
            edges.append(&mut es);
        }
        edges
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleInformation {
    pub(crate) name: String,
    pub(crate) brief: Option<String>,
    #[serde(flatten)]
    pub(crate) additional: MyMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GsnDocumentNode {
    GsnNode(GsnNode),
    ModuleInformation(ModuleInformation),
}

pub struct Module {
    pub filename: String,
    pub meta: Option<ModuleInformation>,
}

///
/// Get root nodes
/// These are the unreferenced nodes.
///
fn get_root_nodes(nodes: &MyMap<String, GsnNode>) -> Vec<String> {
    // Usage of BTreeSet, since root nodes might be used in output and that should be deterministic
    let mut root_nodes: BTreeSet<String> = nodes.keys().cloned().collect();
    for node in nodes.values() {
        // Remove all keys if they are referenced; used to see if there is more than one top level node
        if let Some(context) = node.in_context_of.as_ref() {
            for cnode in context {
                root_nodes.remove(cnode);
            }
        }
        if let Some(support) = node.supported_by.as_ref() {
            for snode in support {
                root_nodes.remove(snode);
            }
        }
    }
    Vec::from_iter(root_nodes)
}

///
/// Gathers all different 'level' attributes from all nodes.
///
pub fn get_levels(nodes: &MyMap<String, GsnNode>) -> BTreeMap<&str, Vec<&str>> {
    let mut levels = BTreeMap::<&str, Vec<&str>>::new();
    for (id, node) in nodes.iter() {
        if let Some(l) = &node.level {
            levels.entry(l.trim()).or_insert(Vec::new()).push(id);
        }
    }
    levels
}

///
/// Calculate module dependencies
/// Check if a dependency in one direction is already known, then only modify the existing one.
///
///
pub fn calculate_module_dependencies(
    nodes: &MyMap<String, GsnNode>,
) -> BTreeMap<String, BTreeMap<String, EdgeType>> {
    let mut res = BTreeMap::<String, BTreeMap<String, EdgeType>>::new();

    for v in nodes.values() {
        if let Some(sups) = &v.supported_by {
            add_dependencies(sups, nodes, v, &mut res, SingleEdge::SupportedBy);
        }
        if let Some(ctxs) = &v.in_context_of {
            add_dependencies(ctxs, nodes, v, &mut res, SingleEdge::InContextOf);
        }
    }

    // Create empty dummies for other modules.
    for n in nodes.values() {
        if let std::collections::btree_map::Entry::Vacant(e) = res.entry(n.module.to_owned()) {
            e.insert(BTreeMap::new());
        }
    }
    res
}

///
///
/// TODO Continue here
/// Checking the modules one after the other does not work.
///
fn add_dependencies(
    children: &Vec<String>,
    nodes: &MyMap<String, GsnNode>,
    cur_node: &GsnNode,
    dependencies: &mut BTreeMap<String, BTreeMap<String, EdgeType>>,
    dep_type: SingleEdge,
) {
    for child in children {
        let other_module = &nodes.get(child).unwrap().module;
        if &cur_node.module != other_module {
            let oneway = dependencies
                .get(&cur_node.module)
                .map(|es| es.contains_key(other_module))
                .unwrap_or(false);
            let otherway = dependencies
                .get(other_module)
                .map(|es| es.contains_key(&cur_node.module))
                .unwrap_or(false);
            let mut oneway_module = &cur_node.module;
            let mut otherway_module = other_module;
            let mut normal_dir = true;
            if oneway && !otherway {
                // Default assignment
            } else if otherway && !oneway {
                oneway_module = other_module;
                otherway_module = &cur_node.module;
                normal_dir = false;
            } else {
                // What about both true? Cannot happen anymore
                // Both none
                let e = dependencies
                    .entry(cur_node.module.to_owned())
                    .or_insert(BTreeMap::new());
                e.entry(other_module.to_owned())
                    .or_insert(EdgeType::OneWay(dep_type));
            }
            let e = dependencies.get_mut(oneway_module).unwrap();
            e.entry(otherway_module.to_owned())
                .and_modify(|x| {
                    *x = match &x {
                        EdgeType::OneWay(et) if normal_dir => EdgeType::OneWay(*et | dep_type),
                        EdgeType::OneWay(et) if !normal_dir => EdgeType::TwoWay((dep_type, *et)),
                        EdgeType::TwoWay((te, et)) if normal_dir => {
                            EdgeType::TwoWay((*te, *et | dep_type))
                        }
                        EdgeType::TwoWay((te, et)) if !normal_dir => {
                            EdgeType::TwoWay((*te | dep_type, *et))
                        }
                        _ => *x,
                    }
                })
                .or_insert(EdgeType::OneWay(dep_type));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
