use crate::{
    diagnostics::Diagnostics,
    dirgraphsvg::edges::{EdgeType, SingleEdge},
};
use serde::{de::Visitor, Deserialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt::Display,
};

pub mod check;
pub mod validation;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum GsnNodeType {
    Goal,
    Strategy,
    Solution,
    Justification,
    Context,
    Assumption,
}

impl Display for GsnNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Debug, PartialEq)]
pub enum HorizontalIndex {
    Relative(i32),
    Absolute(u32),
}

impl<'de> Deserialize<'de> for HorizontalIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MyVisitor;
        impl<'de> Visitor<'de> for MyVisitor {
            type Value = HorizontalIndex;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "an integer (if provided with sign it is interpreted as relative index)",
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Ok(index) = i32::from_str_radix(v, 10) {
                    if v.starts_with(&['+', '-']) {
                        Ok(HorizontalIndex::Relative(index))
                    } else {
                        if let Ok(abs_index) = u32::try_from(index) {
                            Ok(HorizontalIndex::Absolute(abs_index))
                        } else {
                            Err(serde::de::Error::invalid_type(
                                serde::de::Unexpected::Str(v),
                                &self,
                            ))
                        }
                    }
                } else {
                    Err(serde::de::Error::invalid_type(
                        serde::de::Unexpected::Str(v),
                        &self,
                    ))
                }
            }
        }
        deserializer.deserialize_str(MyVisitor)
    }
}

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
    pub(crate) additional: BTreeMap<String, String>,
    #[serde(skip_deserializing)]
    pub(crate) module: String,
    pub(crate) node_type: Option<GsnNodeType>,
    pub(crate) horizontal_index: Option<HorizontalIndex>,
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

    pub fn fix_node_type(&mut self, id: &str) {
        self.node_type = if let Some(node_type) = self.node_type {
            Some(node_type)
        } else {
            get_node_type_from_text(id)
        }
    }
}

fn get_node_type_from_text(text: &str) -> Option<GsnNodeType> {
    // Order is important due to Sn and S
    match text {
        id if id.starts_with('G') => Some(GsnNodeType::Goal),
        id if id.starts_with("Sn") => Some(GsnNodeType::Solution),
        id if id.starts_with('S') => Some(GsnNodeType::Strategy),
        id if id.starts_with('C') => Some(GsnNodeType::Context),
        id if id.starts_with('A') => Some(GsnNodeType::Assumption),
        id if id.starts_with('J') => Some(GsnNodeType::Justification),
        _ => None,
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleInformation {
    pub(crate) name: String,
    pub(crate) brief: Option<String>,
    pub(crate) extends: Option<Vec<ExtendsModule>>,
    #[serde(flatten)]
    pub(crate) additional: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GsnDocumentNode {
    GsnNode(GsnNode),
    ModuleInformation(ModuleInformation),
}

#[derive(Default)]
pub struct Module {
    pub relative_module_path: String,
    pub meta: ModuleInformation,
}

#[derive(Debug, Deserialize)]
pub struct ExtendsModule {
    pub module: String,
    pub develops: BTreeMap<String, Vec<String>>,
}

///
///
///
///
pub fn extend_modules(
    diags: &mut Diagnostics,
    nodes: &mut BTreeMap<String, GsnNode>,
    modules: &HashMap<String, Module>,
) {
    for (module_name, module_info) in modules {
        if let Some(extensions) = &module_info.meta.extends {
            for ext in extensions {
                if !modules.contains_key(&ext.module) {
                    diags.add_error(
                            Some(module_name),
                            format!("C09: Module {} is not found, but is supposed to be extended by module {}.", ext.module, module_name),
                        );
                }
                for (foreign_id, local_ids) in &ext.develops {
                    if let Some(foreign_node) = nodes.get_mut(foreign_id) {
                        if foreign_node.module != ext.module {
                            diags.add_error(
                                    Some(module_name),
                                    format!("C10: Element {} does not exist in module {}, but is supposed to be extended by {}.", foreign_id, ext.module, local_ids.join(",")),
                                );
                        } else if foreign_node.undeveloped != Some(true) {
                            diags.add_error(
                                    Some(module_name),
                                    format!("C10: Element {} is not undeveloped, but is supposed to be extended by {}.", foreign_id, local_ids.join(",")),
                                );
                        } else {
                            foreign_node.supported_by = Some(local_ids.to_vec());
                            foreign_node.undeveloped = Some(false);
                        }
                    } else {
                        diags.add_error(
                                Some(module_name),
                                format!("C10: Element {} does not exist, but is supposed to be extended by {}.", foreign_id, local_ids.join(",")),
                            );
                    }
                }
            }
        }
    }
}

///
/// Get root nodes
/// These are the unreferenced nodes.
///
fn get_root_nodes(nodes: &BTreeMap<String, GsnNode>) -> Vec<String> {
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
pub fn get_levels(nodes: &BTreeMap<String, GsnNode>) -> BTreeMap<&str, Vec<&str>> {
    let mut levels = BTreeMap::<&str, Vec<&str>>::new();
    for (id, node) in nodes.iter() {
        if let Some(l) = &node.level {
            levels.entry(l.trim()).or_default().push(id);
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
    nodes: &BTreeMap<String, GsnNode>,
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
///
fn add_dependencies(
    children: &Vec<String>,
    nodes: &BTreeMap<String, GsnNode>,
    cur_node: &GsnNode,
    dependencies: &mut BTreeMap<String, BTreeMap<String, EdgeType>>,
    dep_type: SingleEdge,
) {
    for child in children {
        // Unwrap is ok, since node names in `nodes` always exist
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
                // What about both true? Cannot happen, since we covered this in the match statement below.
                // Here, both are false
                let e = dependencies.entry(cur_node.module.to_owned()).or_default();
                e.entry(other_module.to_owned())
                    .or_insert(EdgeType::OneWay(dep_type));
            }
            // unwrap is ok, since oneway_module is either newly inserted (else-case above),
            // or found in `dependencies` before the if-else if-else.
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
    use anyhow::Result;
    #[test]
    fn derser_hor_index() -> Result<()> {
        let res: HorizontalIndex = serde_yaml::from_str("+5")?;
        assert_eq!(res, HorizontalIndex::Relative(5));
        let res: HorizontalIndex = serde_yaml::from_str("-3")?;
        assert_eq!(res, HorizontalIndex::Relative(-3));
        let res: HorizontalIndex = serde_yaml::from_str("0")?;
        assert_eq!(res, HorizontalIndex::Absolute(0));
        Ok(())
    }

    #[test]
    fn derser_hor_invalid_index() -> Result<()> {
        let res: Result<HorizontalIndex,_> = serde_yaml::from_str("+asdf");
        assert!(res.is_err());
        let res: Result<HorizontalIndex,_> = serde_yaml::from_str("");
        assert!(res.is_err());
        // 2**32 + 1
        let res: Result<HorizontalIndex,_> = serde_yaml::from_str("4294967297");
        assert!(res.is_err());
        let res: Result<HorizontalIndex,_> = serde_yaml::from_str("+4294967297");
        assert!(res.is_err());
        let res: Result<HorizontalIndex,_> = serde_yaml::from_str("-4294967297");
        assert!(res.is_err());
        let res: Result<HorizontalIndex,_> = serde_yaml::from_str("-x");
        assert!(res.is_err());
        let res: Result<HorizontalIndex,_> = serde_yaml::from_str("bslkdf");
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn no_level_exists() {
        let mut nodes = BTreeMap::<String, GsnNode>::new();
        nodes.insert("Sn1".to_owned(), Default::default());
        let output = get_levels(&nodes);
        assert!(output.is_empty());
    }

    #[test]
    fn two_levels_exist() {
        let mut nodes = BTreeMap::<String, GsnNode>::new();
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
