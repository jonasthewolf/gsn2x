use crate::{
    diagnostics::Diagnostics,
    dirgraph::{DirectedGraphEdgeType, DirectedGraphNodeType},
    dirgraphsvg::edges::{EdgeType, SingleEdge},
};
use anyhow::{anyhow, Error};
use serde::{
    de::{self},
    Deserialize, Deserializer,
};
use serde_yaml::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    path::{Path, PathBuf},
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GsnEdgeType {
    SupportedBy,
    InContextOf,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HorizontalIndex {
    Relative(i32),
    Absolute(AbsoluteIndex),
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(untagged, rename_all = "camelCase", try_from = "Value")]
pub enum AbsoluteIndex {
    Number(u32),
    Last,
}

impl TryFrom<Value> for AbsoluteIndex {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let int_val = value.as_u64();
        match int_val {
            Some(x) => Ok(AbsoluteIndex::Number(x.try_into().unwrap())),
            None => {
                if value == "last" {
                    Ok(AbsoluteIndex::Last)
                } else {
                    Err(anyhow!("Either provide positive integer or \"last\""))
                }
            }
        }
    }
}

///
/// The main struct of this program
/// It describes a GSN element
///
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GsnNode {
    pub(crate) text: String,
    #[serde(default)]
    pub(crate) in_context_of: Vec<String>,
    #[serde(default)]
    pub(crate) supported_by: Vec<String>,
    pub(crate) undeveloped: Option<bool>,
    #[serde(default)]
    pub(crate) classes: Vec<String>,
    pub(crate) url: Option<String>,
    pub(crate) rank_increment: Option<usize>,
    pub(crate) horizontal_index: Option<HorizontalIndex>,
    pub(crate) node_type: Option<GsnNodeType>,
    pub(crate) word_wrap: Option<u32>,
    #[serde(default, deserialize_with = "deser_acp")]
    pub(crate) acp: BTreeMap<String, Vec<String>>,
    #[serde(flatten, deserialize_with = "deser_additional")]
    pub(crate) additional: BTreeMap<String, String>,
    #[serde(skip_deserializing)]
    pub(crate) module: String,
}

///
/// Deserialize everything that is a Map<String, Value> to a Map<String, String>
/// and ignore the rest
///
///
fn deser_additional<'de, D>(deserializer: D) -> Result<BTreeMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut result = BTreeMap::new();
    let map: Result<BTreeMap<String, Value>, D::Error> = Deserialize::deserialize(deserializer);
    if let Ok(map) = map {
        map.into_iter().for_each(|(k, v)| {
            result.insert(
                k,
                if v.is_string() {
                    v.as_str().unwrap().to_owned()
                } else {
                    serde_yaml::to_string(&v).unwrap()
                },
            ); // unwraps are ok, since deserialization from YAML just worked.
        });
    }

    Ok(result)
}

///
///
///
fn deser_acp<'de, D>(deserializer: D) -> Result<BTreeMap<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut result = BTreeMap::new();
    let map: Result<BTreeMap<String, Value>, D::Error> = Deserialize::deserialize(deserializer);
    if let Ok(map) = map {
        for (k, v) in map {
            let val = if v.is_string() {
                Ok(vec![v.as_str().unwrap().to_owned()])
            } else if v.is_sequence() {
                let seq = v.as_sequence().unwrap();
                if seq.iter().all(|c| c.is_string()) {
                    Ok(seq.iter().map(|x| x.as_str().unwrap().to_owned()).collect())
                } else {
                    Err(de::Error::invalid_type(
                        de::Unexpected::Other("Unknown"),
                        &"string",
                    ))
                }
            } else {
                Err(de::Error::invalid_type(
                    de::Unexpected::Other("Unknown"),
                    &"string of list of strings",
                ))
            }?;
            result.insert(k, val); // unwraps are ok, since deserialization from YAML just worked.
        }
        Ok(result)
    } else {
        Err(de::Error::invalid_type(
            de::Unexpected::Other("Unknown"),
            &"map of string to string or list of strings",
        ))
    }
}

impl GsnNode {
    ///
    ///
    ///
    pub fn get_edges(&self) -> Vec<(String, GsnEdgeType)> {
        let mut edges = Vec::new();
        let mut es: Vec<(String, GsnEdgeType)> = self
            .in_context_of
            .iter()
            .map(|target| (target.to_owned(), GsnEdgeType::InContextOf))
            .collect();
        edges.append(&mut es);
        let mut es: Vec<(String, GsnEdgeType)> = self
            .supported_by
            .iter()
            .map(|target| (target.to_owned(), GsnEdgeType::SupportedBy))
            .collect();
        edges.append(&mut es);
        edges
    }

    ///
    ///
    ///
    pub fn fix_node_type(&mut self, id: &str) {
        self.node_type = if let Some(node_type) = self.node_type {
            Some(node_type)
        } else {
            get_node_type_from_text(id)
        }
    }
}

///
///
///
///
impl<'a> DirectedGraphNodeType<'a> for GsnNode {
    ///
    ///
    ///
    fn get_forced_level(&self) -> Option<usize> {
        self.rank_increment
    }

    ///
    ///
    ///
    fn get_horizontal_index(&self, current_index: usize) -> Option<usize> {
        match self.horizontal_index {
            Some(HorizontalIndex::Absolute(idx)) => match idx {
                AbsoluteIndex::Number(num) => num.try_into().ok(),
                AbsoluteIndex::Last => Some(usize::MAX),
            },
            Some(HorizontalIndex::Relative(idx)) => (current_index as i32 + idx).try_into().ok(),
            None => None,
        }
    }
}

///
///
///
impl<'a> DirectedGraphEdgeType<'a> for GsnEdgeType {
    ///
    ///
    ///
    fn is_primary_child_edge(&self) -> bool {
        match self {
            GsnEdgeType::SupportedBy => true,
            GsnEdgeType::InContextOf => false,
        }
    }

    ///
    ///
    ///
    fn is_secondary_child_edge(&self) -> bool {
        match self {
            GsnEdgeType::SupportedBy => false,
            GsnEdgeType::InContextOf => true,
        }
    }
}

///
///
///
///
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleInformation {
    pub(crate) name: String,
    pub(crate) brief: Option<String>,
    #[serde(default)]
    pub(crate) extends: Vec<ExtendsModule>,
    pub(crate) horizontal_index: Option<HorizontalIndex>,
    pub(crate) rank_increment: Option<usize>,
    pub(crate) word_wrap: Option<u32>,
    #[serde(default)]
    pub(crate) uses: Vec<String>,
    #[serde(flatten, deserialize_with = "deser_additional")]
    pub(crate) additional: BTreeMap<String, String>,
}

impl ModuleInformation {
    pub fn new(name: String) -> Self {
        ModuleInformation {
            name,
            brief: None,
            extends: vec![],
            uses: vec![],
            word_wrap: None,
            horizontal_index: None,
            rank_increment: None,
            additional: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GsnDocument {
    GsnNode(GsnNode),
    ModuleInformation(ModuleInformation),
}

#[derive(Clone, Default)]
pub enum Origin {
    #[default]
    CommandLine,
    File(String),
    Excluded,
}

impl Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Origin::CommandLine => f.write_str("command line"),
            Origin::File(file) => f.write_fmt(format_args!("file {file}")),
            Origin::Excluded => f.write_str("automatic extension by gsn2x"),
        }
    }
}

#[derive(Default)]
pub struct Module {
    pub orig_file_name: String,
    pub canonical_path: Option<PathBuf>,
    pub origin: Origin,
    pub meta: ModuleInformation,
}

pub trait FindModuleByPath {
    fn find_module_by_path(&self, module_path: &Path) -> Option<&Module>;
}

impl FindModuleByPath for BTreeMap<String, Module> {
    fn find_module_by_path(&self, module_path: &Path) -> Option<&Module> {
        self.values()
            .find(|m| m.canonical_path == Some(module_path.to_path_buf()))
    }
}

#[derive(Clone, Debug, Deserialize)]
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
    modules: &BTreeMap<String, Module>,
) {
    for (module_name, module_info) in modules {
        for ext in &module_info.meta.extends {
            if !modules.contains_key(&ext.module) {
                diags.add_error(
                    Some(module_name),
                    format!(
                        "C09: Module {} is not found, but is supposed to be extended by module {}.",
                        ext.module, module_name
                    ),
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
                        foreign_node.supported_by = local_ids.to_vec();
                        foreign_node.undeveloped = Some(false);
                    }
                } else {
                    diags.add_error(
                        Some(module_name),
                        format!(
                            "C10: Element {} does not exist, but is supposed to be extended by {}.",
                            foreign_id,
                            local_ids.join(",")
                        ),
                    );
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
        for cnode in &node.in_context_of {
            root_nodes.remove(cnode);
        }
        for snode in &node.supported_by {
            root_nodes.remove(snode);
        }
    }
    Vec::from_iter(root_nodes)
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
        if !v.supported_by.is_empty() {
            add_dependencies(&v.supported_by, nodes, v, &mut res, SingleEdge::SupportedBy);
        }
        if !v.in_context_of.is_empty() {
            add_dependencies(
                &v.in_context_of,
                nodes,
                v,
                &mut res,
                SingleEdge::InContextOf,
            );
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
    use anyhow::{anyhow, Result};
    #[test]
    fn serde_hor_index() -> Result<()> {
        let res: HorizontalIndex = serde_yaml::from_str("!relative 5")?;
        assert_eq!(res, HorizontalIndex::Relative(5));
        let res: HorizontalIndex = serde_yaml::from_str("!relative -3")?;
        assert_eq!(res, HorizontalIndex::Relative(-3));
        let res: HorizontalIndex = serde_yaml::from_str("!relative 0")?;
        assert_eq!(res, HorizontalIndex::Relative(0));
        let res: HorizontalIndex = serde_yaml::from_str("!absolute 0")?;
        assert_eq!(res, HorizontalIndex::Absolute(AbsoluteIndex::Number(0)));
        let res: HorizontalIndex = serde_yaml::from_str("!absolute 7")?;
        assert_eq!(res, HorizontalIndex::Absolute(AbsoluteIndex::Number(7)));
        let res: HorizontalIndex = serde_yaml::from_str("!absolute last")?;
        assert_eq!(res, HorizontalIndex::Absolute(AbsoluteIndex::Last));
        Ok(())
    }

    #[test]
    fn serde_hor_invalid_index() -> Result<()> {
        let res: Result<HorizontalIndex, _> = serde_yaml::from_str("+asdf");
        assert!(res.is_err());
        let res: Result<HorizontalIndex, _> = serde_yaml::from_str("");
        assert!(res.is_err());
        // 2**32 + 1
        let res: Result<HorizontalIndex, _> = serde_yaml::from_str("4294967297");
        assert!(res.is_err());
        let res: Result<HorizontalIndex, _> = serde_yaml::from_str("+4294967297");
        assert!(res.is_err());
        let res: Result<HorizontalIndex, _> = serde_yaml::from_str("-4294967297");
        assert!(res.is_err());
        let res: Result<HorizontalIndex, _> = serde_yaml::from_str("-x");
        assert!(res.is_err());
        let res: Result<HorizontalIndex, _> = serde_yaml::from_str("bslkdf");
        assert!(res.is_err());
        let res: Result<HorizontalIndex, _> = serde_yaml::from_str("!absolute null");
        assert!(res.is_err());
        let res: Result<HorizontalIndex, _> = serde_yaml::from_str("!absolute");
        assert!(res.is_err());
        let res: Result<HorizontalIndex, _> = serde_yaml::from_str("null");
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn serde_back() -> Result<()> {
        let goal = r#"
text: test
undeveloped: true
horizontalIndex:
  relative: -23
"#;
        let res: GsnDocument = serde_yaml::from_str(goal)?;
        if let GsnDocument::GsnNode(node) = res {
            assert_eq!(node.horizontal_index, Some(HorizontalIndex::Relative(-23)));
            Ok(())
        } else {
            Err(anyhow!("no GsnNode deserialized"))
        }
    }

    #[test]
    fn nodetype() -> Result<()> {
        let nt: GsnNodeType = serde_yaml::from_str("Solution")?;
        assert_eq!(nt, GsnNodeType::Solution);

        let gsn = r#"
G1:
  text: Goal1
  supportedBy: [C1]

C1:
  text: Solution1
  nodeType: Solution
"#;
        let res: BTreeMap<String, GsnDocument> = serde_yaml::from_str(gsn)?;
        if let Some(GsnDocument::GsnNode(n)) = res.get("C1") {
            assert_eq!(n.node_type, Some(GsnNodeType::Solution));
            Ok(())
        } else {
            Err(anyhow!("Serialization did not work"))
        }
    }

    #[test]
    fn deser_acp1() {
        let gsn = r#"
G1:
  text: Goal1
  acp:
    - C1
    - G2
"#;
        assert!(serde_yaml::from_str::<BTreeMap<String, GsnDocument>>(gsn).is_err());
    }

    #[test]
    fn deser_acp2() {
        let gsn = r#"
G1:
  text: Goal1
  acp:
    ACP1: true
"#;
        assert!(serde_yaml::from_str::<BTreeMap<String, GsnDocument>>(gsn).is_err());
    }

    #[test]
    fn deser_acp3() {
        let gsn = r#"
G1:
  text: Goal1
  acp:
    ACP1: [true, 123]
"#;
        assert!(serde_yaml::from_str::<BTreeMap<String, GsnDocument>>(gsn).is_err());
    }

    #[test]
    fn edge_type_copy_clone() {
        let edge = GsnEdgeType::SupportedBy;
        let edge_copy = edge;
        assert_eq!(edge, edge_copy);
    }

    #[test]
    fn edge_type_debug() {
        assert_eq!(format!("{:?}", GsnEdgeType::SupportedBy), "SupportedBy");
        assert_eq!(format!("{:?}", GsnEdgeType::InContextOf), "InContextOf");
    }
}
