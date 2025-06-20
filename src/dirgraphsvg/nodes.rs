use std::cell::RefCell;

use svg::{
    Node,
    node::element::{Anchor, Element, Line, Use},
};

use crate::{
    dirgraph::DirectedGraphNodeType,
    dirgraphsvg::{
        FontInfo,
        render::{ACP_BOX_SIZE, create_text},
        util::point2d::Point2D,
    },
    gsn::{GsnNode, HorizontalIndex},
};

use self::{
    away_node::{AwayNodeType, AwayType},
    box_node::BoxType,
    elliptical_node::EllipticalType,
};

use super::{
    escape_text,
    render::{PADDING_HORIZONTAL, create_group},
    util::{
        escape_url,
        font::{str_line_bounding_box, text_line_bounding_box},
        markdown::MarkdownText,
    },
};

mod away_node;
mod box_node;
mod elliptical_node;

///
/// The direction of the port where edges dock to nodes.
///
#[derive(Eq, PartialEq)]
pub enum Port {
    North,
    East,
    South,
    West,
}

enum NodeType {
    Box(BoxType),
    Ellipsis(EllipticalType),
    Away(AwayType),
}

///
/// The node type when rendering SVGs
///
pub struct SvgNode {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    identifier: String,
    text: MarkdownText,
    masked: bool,
    defeated: bool,
    url: Option<String>,
    classes: Vec<String>,
    rank_increment: Option<usize>,
    horizontal_index: Option<HorizontalIndex>,
    node_type: NodeType,
    acp: Vec<String>,
}

impl DirectedGraphNodeType<'_> for RefCell<SvgNode> {
    fn get_forced_level(&self) -> Option<usize> {
        self.borrow().rank_increment
    }

    fn get_horizontal_index(&self, current_index: usize) -> Option<usize> {
        match self.borrow().horizontal_index {
            Some(HorizontalIndex::Absolute(x)) => match x {
                crate::gsn::AbsoluteIndex::Number(num) => num.try_into().ok(),
                crate::gsn::AbsoluteIndex::Last => Some(usize::MAX),
            },
            Some(HorizontalIndex::Relative(x)) => (x + current_index as i32).try_into().ok(),
            None => None,
        }
    }
}

struct SizeContext {
    width: i32,
    height: i32,
    text_width: i32,
    text_height: i32,
}

const DEFEATED_OFFSET: i32 = 5;
const OFFSET_IDENTIFIER: i32 = 5;
const MODULE_TAB_HEIGHT: i32 = 10;

impl SvgNode {
    pub fn is_defeated(&self) -> bool {
        self.defeated
    }

    pub fn has_acp(&self) -> bool {
        !self.acp.is_empty()
    }

    pub fn is_away_node(&self) -> bool {
        matches!(&self.node_type, NodeType::Away(_))
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_coordinates(&self, port: Port) -> Point2D<i32> {
        let mut coords = Point2D {
            x: match port {
                Port::North => self.x,
                Port::East => self.x + (self.width / 2),
                Port::South => self.x,
                Port::West => self.x - (self.width / 2),
            },
            y: match port {
                Port::North => self.y - (self.height / 2),
                Port::East => self.y,
                Port::South => self.y + (self.height / 2),
                Port::West => self.y,
            },
        };
        if let NodeType::Box(BoxType::Module) = &self.node_type {
            if port == super::Port::North {
                coords.y += MODULE_TAB_HEIGHT;
            }
        }

        coords
    }

    pub fn set_position(&mut self, pos: &Point2D<i32>) {
        self.x = pos.x;
        self.y = pos.y;
    }

    pub fn get_position(&self) -> Point2D<i32> {
        Point2D {
            x: self.x,
            y: self.y,
        }
    }

    ///
    /// Calculate size of the SVG node based on its type.
    ///
    ///
    pub fn calculate_size(&mut self, font: &FontInfo) {
        let (min_width, min_height) = match &self.node_type {
            NodeType::Box(x) => x.get_minimum_size(),
            NodeType::Ellipsis(x) => x.get_minimum_size(),
            NodeType::Away(x) => x.get_minimum_size(),
        };
        let mut size_context = self.calculate_text_size(font);
        (size_context.width, size_context.height) = match &self.node_type {
            NodeType::Box(x) => x.calculate_size(font, min_width, min_height, &mut size_context),
            NodeType::Ellipsis(x) => {
                x.calculate_size(font, min_width, min_height, &mut size_context)
            }
            NodeType::Away(x) => x.calculate_size(font, min_width, min_height, &mut size_context),
        };
        // Set minimum size
        self.width = size_context.width;
        self.height = size_context.height;
        match &mut self.node_type {
            NodeType::Box(_) => (),
            NodeType::Ellipsis(x) => {
                x.text_width = size_context.text_width;
                x.text_height = size_context.text_height;
            }
            NodeType::Away(x) => {
                (_, x.mod_height) = str_line_bounding_box(font, &x.module, false);
            }
        }
    }

    ///
    /// Calculate size of text without padding
    ///
    /// Height: Identifier + Offset + Text lines (wrapped)
    /// Width: Max of Identifier or longest Text line
    ///
    ///
    fn calculate_text_size(&self, font: &FontInfo) -> SizeContext {
        // First row is identifier, thus treated differently
        let (head_width, head_height) = str_line_bounding_box(font, &self.identifier, true);
        let mut text_height = head_height + OFFSET_IDENTIFIER;
        let mut text_width = head_width;
        for text_line in self.text.lines() {
            let (line_width, line_height) = text_line_bounding_box(font, text_line, false);
            text_height += line_height;
            text_width = std::cmp::max(text_width, line_width);
        }
        SizeContext {
            width: 0,
            height: 0,
            text_width,
            text_height,
        }
    }

    ///
    /// Render the nodes (entry point for rendering all SVG node types)
    ///
    pub fn render(&self, font: &FontInfo, document: &mut Element) {
        let mut g = create_group(&self.identifier, &self.classes);

        let border_color = if self.masked { "lightgrey" } else { "black" };

        match &self.node_type {
            NodeType::Box(x) => x.render(self, font, &mut g, border_color),
            NodeType::Ellipsis(x) => x.render(self, font, &mut g, border_color),
            NodeType::Away(x) => x.render(self, font, &mut g, border_color),
        };

        render_acp_box(self, font, &mut g);

        // Render cross if defeated.
        if self.defeated {
            render_defeated(self, &mut g);
        }

        // Add link to self.url for complete node
        if let Some(url) = &self.url {
            let link = Anchor::new().set("href", escape_url(url.as_str())).add(g);
            document.append(link);
        } else {
            document.append(g);
        }
    }

    ///
    /// Create a new SVG node
    ///
    fn new(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        module_url: Option<String>,
        add_classes: &[&str],
        char_wrap: Option<u32>,
    ) -> Self {
        // Add layer to node output
        let node_text = node_text_from_node_and_layers(identifier, gsn_node, layers, char_wrap);
        // Setup CSS classes
        let mut classes = node_classes_from_node(identifier, gsn_node, masked);
        classes.push("gsnelem".to_owned());
        if gsn_node.defeated {
            classes.push("gsndefeated".to_owned());
        }
        classes.append(
            &mut add_classes
                .iter()
                .map(|&c| c.to_owned())
                .collect::<Vec<String>>(),
        );

        // Get all ACPs of this node that affect this node
        let acp = gsn_node
            .acp
            .iter()
            .filter_map(|(acp, ids)| {
                if ids.iter().any(|id| id == identifier) {
                    Some(acp)
                } else {
                    None
                }
            })
            .cloned()
            .collect();

        SvgNode {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            masked,
            defeated: gsn_node.defeated,
            identifier: identifier.to_owned(),
            text: node_text.into(),
            url: module_url,
            classes,
            horizontal_index: gsn_node.horizontal_index,
            rank_increment: gsn_node.rank_increment,
            node_type: NodeType::Box(BoxType::Context),
            acp,
        }
    }

    ///
    /// Create dummy node for targeting other edges.
    ///
    ///
    pub fn new_dummy(x: i32, y: i32) -> Self {
        SvgNode {
            x,
            y,
            width: 0,
            height: 0,
            identifier: "".to_owned(),
            text: MarkdownText::from(""),
            masked: false,
            defeated: false,
            url: None,
            classes: vec![],
            rank_increment: None,
            horizontal_index: None,
            node_type: NodeType::Box(BoxType::Normal(0)),
            acp: vec![],
        }
    }

    ///
    /// New Assumption.
    ///
    pub fn new_assumption(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            gsn_node.url.to_owned(),
            &["gsnasmp"],
            char_wrap,
        );
        n.node_type = NodeType::Ellipsis(EllipticalType {
            admonition: Some("A".to_owned()),
            circle: false,
            text_width: 0,
            text_height: 0,
        });
        n
    }

    ///
    /// New Context.
    ///
    pub fn new_context(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        char_wrap: Option<u32>,
    ) -> Self {
        SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            gsn_node.url.to_owned(),
            &["gsnctxt"],
            char_wrap,
        )
    }

    ///
    /// New Justification.
    ///
    pub fn new_justification(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            gsn_node.url.to_owned(),
            &["gsnjust"],
            char_wrap,
        );
        n.node_type = NodeType::Ellipsis(EllipticalType {
            admonition: Some("J".to_owned()),
            circle: false,
            text_width: 0,
            text_height: 0,
        });
        n
    }

    ///
    /// New Solution.
    ///
    pub fn new_solution(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            gsn_node.url.to_owned(),
            &["gsnsltn"],
            char_wrap,
        );
        n.node_type = NodeType::Ellipsis(EllipticalType {
            admonition: None,
            circle: true,
            text_width: 0,
            text_height: 0,
        });
        n
    }

    ///
    /// New Counter Solution.
    ///
    pub fn new_counter_solution(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            gsn_node.url.to_owned(),
            &["gsncountersltn"],
            char_wrap,
        );
        n.node_type = NodeType::Ellipsis(EllipticalType {
            admonition: None,
            circle: true,
            text_width: 0,
            text_height: 0,
        });
        n
    }

    ///
    /// New Strategy.
    ///
    pub fn new_strategy(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            gsn_node.url.to_owned(),
            &["gsnstgy"],
            char_wrap,
        );
        if gsn_node.undeveloped {
            n.node_type = NodeType::Box(BoxType::Undeveloped(15));
        } else {
            n.node_type = NodeType::Box(BoxType::Normal(15));
        }
        n
    }

    ///
    /// New Goal.
    ///
    pub fn new_goal(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        char_wrap: Option<u32>,
    ) -> Self {
        let undeveloped = gsn_node.undeveloped;
        let mut classes = vec!["gsngoal"];
        if undeveloped {
            classes.push("gsn_undeveloped");
        }
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            gsn_node.url.to_owned(),
            &classes,
            char_wrap,
        );
        if undeveloped {
            n.node_type = NodeType::Box(BoxType::Undeveloped(0));
        } else {
            n.node_type = NodeType::Box(BoxType::Normal(0));
        }
        n
    }

    ///
    /// New Counter Goal.
    ///
    pub fn new_counter_goal(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        char_wrap: Option<u32>,
    ) -> Self {
        let undeveloped = gsn_node.undeveloped;
        let mut classes = vec!["gsncountergoal"];
        if undeveloped {
            classes.push("gsn_undeveloped");
        }
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            gsn_node.url.to_owned(),
            &classes,
            char_wrap,
        );
        if undeveloped {
            n.node_type = NodeType::Box(BoxType::Undeveloped(0));
        } else {
            n.node_type = NodeType::Box(BoxType::Normal(0));
        }
        n
    }

    ///
    /// New Away Assumption.
    ///
    pub fn new_away_assumption(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        module_url: Option<String>,
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            module_url.to_owned(),
            &["gsnawayasmp"],
            char_wrap,
        );
        n.node_type = NodeType::Away(AwayType {
            module: gsn_node.module.to_owned(),
            module_url,
            away_type: AwayNodeType::Assumption,
            mod_height: 0,
        });
        n
    }

    ///
    /// New Away Goal.
    ///
    pub fn new_away_goal(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        module_url: Option<String>,
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            module_url.to_owned(),
            &["gsnawaygoal"],
            char_wrap,
        );
        n.node_type = NodeType::Away(AwayType {
            module: gsn_node.module.to_owned(),
            module_url,
            away_type: AwayNodeType::Goal,
            mod_height: 0,
        });
        n
    }

    ///
    /// There is actually no "Away Strategy" in the standard, however, to set the URL we just pretend it here.
    ///
    pub fn new_away_strategy(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        module_url: Option<String>,
        char_wrap: Option<u32>,
    ) -> Self {
        let mut cloned_strategy = gsn_node.clone();
        cloned_strategy.url = module_url;
        SvgNode::new_strategy(identifier, &cloned_strategy, masked, layers, char_wrap)
    }

    ///
    /// New Away Justification.
    ///
    pub fn new_away_justification(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        module_url: Option<String>,
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            module_url.to_owned(),
            &["gsnawayjust"],
            char_wrap,
        );
        n.node_type = NodeType::Away(AwayType {
            module: gsn_node.module.to_owned(),
            module_url,
            away_type: AwayNodeType::Justification,
            mod_height: 0,
        });
        n
    }

    ///
    /// New Away Context.
    ///
    pub fn new_away_context(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        module_url: Option<String>,
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            module_url.to_owned(),
            &["gsnawayctxt"],
            char_wrap,
        );
        n.node_type = NodeType::Away(AwayType {
            module: gsn_node.module.to_owned(),
            module_url,
            away_type: AwayNodeType::Context,
            mod_height: 0,
        });
        n
    }

    ///
    /// New Away Solution.
    ///
    pub fn new_away_solution(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        module_url: Option<String>,
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            module_url.to_owned(),
            &["gsnawaysltn"],
            char_wrap,
        );
        n.node_type = NodeType::Away(AwayType {
            module: gsn_node.module.to_owned(),
            module_url,
            away_type: AwayNodeType::Solution,
            mod_height: 0,
        });
        n
    }

    ///
    /// New Module for Architecture View.
    ///
    pub fn new_module(
        identifier: &str,
        gsn_node: &GsnNode,
        masked: bool,
        layers: &[String],
        module_url: Option<String>,
        char_wrap: Option<u32>,
    ) -> Self {
        let mut n = SvgNode::new(
            identifier,
            gsn_node,
            masked,
            layers,
            module_url,
            &["gsnmodule"],
            char_wrap,
        );
        n.node_type = NodeType::Box(BoxType::Module);
        n
    }
}

///
/// Render an element ACP box
///
///
fn render_acp_box(node: &SvgNode, font: &FontInfo, context: &mut Element) {
    if !node.acp.is_empty() {
        context.append(
            Use::new()
                .set("href", "#acp")
                .set("x", node.x - ACP_BOX_SIZE)
                .set("y", node.y + node.height / 2),
        );
        let acp_text = node.acp.join(", ");
        let acp_y = node.y + node.height / 2 + str_line_bounding_box(font, &acp_text, false).1;
        context.append(create_text(
            &acp_text.into(),
            node.x + ACP_BOX_SIZE + PADDING_HORIZONTAL,
            acp_y,
            font,
            false,
        ));
    }
}

///
/// Render cross over defeated element.
///
fn render_defeated(node: &SvgNode, context: &mut Element) {
    let l1 = Line::new()
        .set("x1", node.x - node.width / 2 - DEFEATED_OFFSET)
        .set("y1", node.y - node.height / 2 - DEFEATED_OFFSET)
        .set("x2", node.x + node.width / 2 + DEFEATED_OFFSET)
        .set("y2", node.y + node.height / 2 + DEFEATED_OFFSET)
        .set("stroke", "black");
    context.append(l1);
    let l2 = Line::new()
        .set("x1", node.x + node.width / 2 + DEFEATED_OFFSET)
        .set("y1", node.y - node.height / 2 - DEFEATED_OFFSET)
        .set("x2", node.x - node.width / 2 - DEFEATED_OFFSET)
        .set("y2", node.y + node.height / 2 + DEFEATED_OFFSET)
        .set("stroke", "black");
    context.append(l2);
}

///
/// Add CSS classes for node.
/// What is added as class?
///  - All additional layers (the name of the layer)
///  - The name of the module the node belongs to
///  - The associated ACPs
///
/// The class for type of the node is added when they are created.
///
fn node_classes_from_node(identifier: &str, gsn_node: &GsnNode, masked: bool) -> Vec<String> {
    let layer_classes: Vec<String> = gsn_node
        .additional
        .keys()
        .map(|k| {
            let mut t = escape_text(&k.to_lowercase());
            t.insert_str(0, "gsn_");
            t.to_owned()
        })
        .collect();
    let mod_class = [format!(
        "gsn_module_{}",
        escape_text(&gsn_node.module).to_lowercase()
    )];
    let masked_class = if masked {
        vec!["gsn_masked".to_owned()]
    } else {
        vec![]
    };
    let acp_classes: Vec<String> = gsn_node
        .acp
        .iter()
        .filter_map(|(acp, ids)| {
            if ids.iter().any(|id| id == identifier) {
                Some(format!("acp_{}", escape_text(acp).to_lowercase()))
            } else {
                None
            }
        })
        .collect();
    gsn_node
        .classes
        .iter()
        .chain(layer_classes.iter())
        .chain(mod_class.iter())
        .chain(masked_class.iter())
        .chain(acp_classes.iter())
        .cloned()
        .collect()
}

///
/// Create SVG node text from GsnNode and layer information
///
///
fn node_text_from_node_and_layers(
    identifier: &str,
    gsn_node: &GsnNode,
    layers: &[String],
    char_wrap: Option<u32>,
) -> String {
    use crate::dirgraphsvg::util::wrap_words::wrap_words;

    let mut node_text = if let Some(char_wrap) = gsn_node.char_wrap.or(char_wrap) {
        let id_len = identifier.len() as u32;
        let new_wrap = if char_wrap < id_len {
            id_len
        } else {
            char_wrap
        };
        wrap_words(&gsn_node.text, new_wrap, "\n")
    } else {
        gsn_node.text.to_owned()
    };
    let mut additional_text = vec![];
    for layer in layers {
        if let Some(layer_text) = gsn_node.additional.get(layer) {
            additional_text.push(format!("\n{}:", layer.to_ascii_uppercase()));
            let wrapped_layer_line = if let Some(char_wrap) = gsn_node.char_wrap.or(char_wrap) {
                wrap_words(layer_text, char_wrap, "\n")
            } else {
                layer_text.to_owned()
            };
            additional_text.push(wrapped_layer_line);
        }
    }
    if !additional_text.is_empty() {
        node_text.push('\n');
        node_text.push_str(&additional_text.join("\n"));
    }
    node_text
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::gsn::{GsnNode, GsnNodeType};

    use super::node_text_from_node_and_layers;

    #[test]
    fn node_text_layers() {
        let n1 = GsnNode {
            text: "test text".to_owned(),
            undeveloped: true,
            node_type: Some(crate::gsn::GsnNodeType::Goal),
            additional: BTreeMap::from([("layer1".to_owned(), "text for layer1".to_owned())]),
            ..Default::default()
        };
        let res = node_text_from_node_and_layers("id", &n1, &["layer1".to_owned()], None);
        assert_eq!(res, "test text\n\nLAYER1:\ntext for layer1");
    }

    #[test]
    fn undeveloped_counter_goal() {
        // No assertion. Code is the same as for goal, thus, works.
        let _ = super::SvgNode::new_counter_goal(
            "test",
            &GsnNode {
                node_type: Some(GsnNodeType::CounterGoal),
                undeveloped: true,
                ..Default::default()
            },
            false,
            &[],
            None,
        );
    }
}
