use svg::node::element::Element;

use crate::{
    dirgraphsvg::{util::point2d::Point2D, FontInfo},
    gsn::{GsnNode, HorizontalIndex},
};

use self::{
    away_node::{AwayNodeType, AwayType},
    box_node::BoxType,
    elliptical_node::EllipticalType,
};

use super::{escape_text, render::create_group, util::font::text_bounding_box};

mod away_node;
mod box_node;
mod elliptical_node;

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

pub struct Node {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    identifier: String,
    text: String,
    url: Option<String>,
    classes: Vec<String>,
    // FIXME pub(super)
    pub(super) rank_increment: Option<usize>,
    pub(super) horizontal_index: Option<HorizontalIndex>,
    node_type: NodeType,
}

struct SizeContext {
    width: i32,
    height: i32,
    text: String,
    text_width: i32,
    text_height: i32,
}

const PADDING_VERTICAL: i32 = 7;
const PADDING_HORIZONTAL: i32 = 7;
const OFFSET_IDENTIFIER: i32 = 5;
const MODULE_TAB_HEIGHT: i32 = 10;

impl Node {
    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_coordinates(&self, port: &Port) -> Point2D {
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
            if port == &super::Port::North {
                coords.y += MODULE_TAB_HEIGHT;
            }
        }

        coords
    }

    pub fn set_position(&mut self, pos: &Point2D) {
        self.x = pos.x;
        self.y = pos.y;
    }

    pub fn get_position(&self) -> Point2D {
        Point2D {
            x: self.x,
            y: self.y,
        }
    }

    pub fn calculate_optimal_size(&mut self, font: &FontInfo) {
        let mut min_width = i32::MAX;
        let mut min_height = i32::MAX;
        let mut min_size = SizeContext {
            width: 0,
            height: 0,
            text: "".to_owned(),
            text_width: 0,
            text_height: 0,
        };

        // Minimum number should be low.
        // However, if too low, the test in CI will fail, since size for fonts on different OSes lead to different line numbers.
        // as of 2023-05-06: 10 seems to be a good number
        for wrap in 10..70 {
            let size_context = self.calculate_size(font, wrap);
            if size_context.width > size_context.height
                || (size_context.width <= min_width && size_context.height <= min_height)
            {
                min_width = size_context.width;
                min_height = size_context.height;
                min_size = size_context;
            }
        }
        // Set minimum size
        self.width = min_size.width;
        self.height = min_size.height;
        self.text = min_size.text;
        match &mut self.node_type {
            NodeType::Box(_) => (),
            NodeType::Ellipsis(x) => {
                x.text_width = min_size.text_width;
                x.text_height = min_size.text_height;
            }
            NodeType::Away(x) => {
                (_, x.mod_height) = text_bounding_box(font, &x.module, false);
            }
        }
    }

    ///
    ///
    ///
    ///
    fn calculate_size(&mut self, font: &FontInfo, char_wrap: u32) -> SizeContext {
        let (width, height) = match &self.node_type {
            NodeType::Box(x) => x.get_minimum_size(),
            NodeType::Ellipsis(x) => x.get_minimum_size(),
            NodeType::Away(x) => x.get_minimum_size(),
        };
        let mut size_context = self.calculate_text_size(font, char_wrap);
        (size_context.width, size_context.height) = match &self.node_type {
            NodeType::Box(x) => x.calculate_size(font, width, height, &mut size_context),
            NodeType::Ellipsis(x) => x.calculate_size(font, width, height, &mut size_context),
            NodeType::Away(x) => x.calculate_size(font, width, height, &mut size_context),
        };
        size_context
    }

    ///
    /// Calculate size of text without padding
    ///
    /// Height: Identifier + Offset + Text lines (wrapped)
    /// Width: Max of Identifier or longest Text line
    ///
    ///
    fn calculate_text_size(&self, font: &FontInfo, char_wrap: u32) -> SizeContext {
        use crate::dirgraphsvg::util::wrap_words::wrap_words;

        let text = wrap_words(&self.text, char_wrap, "\n");
        // First row is identifier, thus treated differently
        let (head_width, head_height) = text_bounding_box(font, &self.identifier, true);
        let mut text_height = head_height + OFFSET_IDENTIFIER;
        let mut text_width = head_width;
        for text_line in text.lines() {
            let (line_width, line_height) = text_bounding_box(font, text_line, false);
            text_height += line_height;
            text_width = std::cmp::max(text_width, line_width);
        }
        SizeContext {
            width: 0,
            height: 0,
            text,
            text_width,
            text_height,
        }
    }

    ///
    ///
    ///
    ///
    pub fn render(&self, font: &FontInfo) -> Element {
        let (mut outer, mut context) = create_group(&self.identifier, &self.classes, &self.url);
        context = match &self.node_type {
            NodeType::Box(x) => x.render(self, font, context),
            NodeType::Ellipsis(x) => x.render(self, font, context),
            NodeType::Away(x) => x.render(self, font, context),
        };
        use svg::Node;
        outer.append(context);
        outer
    }

    ///
    ///
    ///
    fn new(
        identifier: &str,
        gsn_node: &GsnNode,
        layers: &[String],
        module_url: Option<String>,
        add_classes: &[&str],
    ) -> Self {
        // Add layer to node output
        let node_text = node_text_from_node_and_layers(gsn_node, layers);
        // Setup CSS classes
        let mut classes = node_classes_from_node(gsn_node);
        classes.push("gsnelem".to_owned());
        classes.append(
            &mut add_classes
                .iter()
                .map(|&c| c.to_owned())
                .collect::<Vec<String>>(),
        );

        Node {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            identifier: identifier.to_owned(),
            text: node_text.to_owned(),
            url: module_url,
            classes,
            horizontal_index: gsn_node.horizontal_index,
            rank_increment: gsn_node.rank_increment,
            node_type: NodeType::Box(BoxType::Context),
        }
    }

    ///
    ///
    ///
    pub fn new_assumption(identifier: &str, gsn_node: &GsnNode, layers: &[String]) -> Self {
        let mut n = Node::new(
            identifier,
            gsn_node,
            layers,
            gsn_node.url.to_owned(),
            &["gsnasmp"],
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
    ///
    ///
    pub fn new_context(identifier: &str, gsn_node: &GsnNode, layers: &[String]) -> Self {
        Node::new(
            identifier,
            gsn_node,
            layers,
            gsn_node.url.to_owned(),
            &["gsnctxt"],
        )
    }

    ///
    ///
    ///
    pub fn new_justification(identifier: &str, gsn_node: &GsnNode, layers: &[String]) -> Self {
        let mut n = Node::new(
            identifier,
            gsn_node,
            layers,
            gsn_node.url.to_owned(),
            &["gsnjust"],
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
    ///
    ///
    pub fn new_solution(identifier: &str, gsn_node: &GsnNode, layers: &[String]) -> Self {
        let mut n = Node::new(
            identifier,
            gsn_node,
            layers,
            gsn_node.url.to_owned(),
            &["gsnsltn"],
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
    ///
    ///
    ///
    pub fn new_strategy(identifier: &str, gsn_node: &GsnNode, layers: &[String]) -> Self {
        let mut n = Node::new(
            identifier,
            gsn_node,
            layers,
            gsn_node.url.to_owned(),
            &["gsnstgy"],
        );
        if gsn_node.undeveloped.unwrap_or(false) {
            n.node_type = NodeType::Box(BoxType::Undeveloped(15));
        } else {
            n.node_type = NodeType::Box(BoxType::Normal(15));
        }
        n
    }

    ///
    ///
    ///
    ///
    pub fn new_goal(identifier: &str, gsn_node: &GsnNode, layers: &[String]) -> Self {
        let undeveloped = gsn_node.undeveloped.unwrap_or(false);
        let mut classes = vec!["gsngoal"];
        if undeveloped {
            classes.push("gsn_undeveloped");
        }
        let mut n = Node::new(
            identifier,
            gsn_node,
            layers,
            gsn_node.url.to_owned(),
            &classes,
        );
        if undeveloped {
            n.node_type = NodeType::Box(BoxType::Undeveloped(0));
        } else {
            n.node_type = NodeType::Box(BoxType::Normal(0));
        }
        n
    }

    ///
    ///
    ///
    pub fn new_away_assumption(
        identifier: &str,
        gsn_node: &GsnNode,
        layers: &[String],
        module_url: Option<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            gsn_node,
            layers,
            module_url.to_owned(),
            &["gsnawayasmp"],
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
    ///
    ///
    pub fn new_away_goal(
        identifier: &str,
        gsn_node: &GsnNode,
        layers: &[String],
        module_url: Option<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            gsn_node,
            layers,
            module_url.to_owned(),
            &["gsnawaygoal"],
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
    ///
    ///
    pub fn new_away_justification(
        identifier: &str,
        gsn_node: &GsnNode,
        layers: &[String],
        module_url: Option<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            gsn_node,
            layers,
            module_url.to_owned(),
            &["gsnawayjust"],
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
    ///
    ///
    pub fn new_away_context(
        identifier: &str,
        gsn_node: &GsnNode,
        layers: &[String],
        module_url: Option<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            gsn_node,
            layers,
            module_url.to_owned(),
            &["gsnawayctxt"],
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
    ///
    ///
    pub fn new_away_solution(
        identifier: &str,
        gsn_node: &GsnNode,
        layers: &[String],
        module_url: Option<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            gsn_node,
            layers,
            module_url.to_owned(),
            &["gsnawaysltn"],
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
    ///
    ///
    ///
    pub fn new_module(
        identifier: &str,
        gsn_node: &GsnNode,
        layers: &[String],
        module_url: Option<String>,
    ) -> Self {
        let mut n = Node::new(identifier, gsn_node, layers, module_url, &["gsnmodule"]);
        n.node_type = NodeType::Box(BoxType::Module);
        n
    }
}

///
///
///
fn node_classes_from_node(gsn_node: &GsnNode) -> Vec<String> {
    let layer_classes: Option<Vec<String>> = gsn_node
        .additional
        .keys()
        .map(|k| {
            let mut t = escape_text(&k.to_ascii_lowercase());
            t.insert_str(0, "gsn_");
            Some(t.to_owned())
        })
        .collect();
    let mut mod_class = gsn_node.module.to_owned();
    mod_class.insert_str(0, "gsn_module_");
    let classes = gsn_node
        .classes
        .iter()
        .chain(layer_classes.iter())
        .flatten()
        .chain(&[mod_class])
        .cloned()
        .collect();
    classes
}

///
/// Create SVG node text from GsnNode and layer information
///
///
fn node_text_from_node_and_layers(gsn_node: &GsnNode, layers: &[String]) -> String {
    let mut node_text = gsn_node.text.to_owned();
    let mut additional_text = vec![];
    for layer in layers {
        if let Some(layer_text) = gsn_node.additional.get(layer) {
            additional_text.push(format!(
                "\n{}: {}",
                layer.to_ascii_uppercase(),
                layer_text.replace('\n', " ")
            ));
        }
    }
    if !additional_text.is_empty() {
        node_text.push_str("\n\n");
        node_text.push_str(&additional_text.join("\n"));
    }
    node_text
}

#[cfg(test)]
mod test {}
