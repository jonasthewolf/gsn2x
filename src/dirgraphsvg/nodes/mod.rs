use svg::node::element::{Anchor, Element, Group};

use crate::{
    dirgraphsvg::{util::point2d::Point2D, FontInfo},
    gsn::HorizontalIndex,
};

use self::{
    away_node::{AwayNodeType, AwayType},
    box_node::BoxType,
    elliptical_node::EllipticalType,
};

use super::util::{escape_node_id, escape_url, font::text_bounding_box};

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

    pub fn render(&self, font: &FontInfo) -> Element {
        let mut context = setup_basics(&self.identifier, &self.classes, &self.url);
        context = match &self.node_type {
            NodeType::Box(x) => x.render(self, font, context),
            NodeType::Ellipsis(x) => x.render(self, font, context),
            NodeType::Away(x) => x.render(self, font, context),
        };
        context
    }

    fn new(
        identifier: &str,
        text: &str,
        url: Option<String>,
        classes: Vec<String>,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        add_class: &str,
    ) -> Self {
        let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), add_class.to_owned()];
        new_classes.append(&mut classes.to_vec());

        Node {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            identifier: identifier.to_owned(),
            text: text.to_owned(),
            url,
            classes: new_classes,
            horizontal_index,
            rank_increment,
            node_type: NodeType::Box(BoxType::Context),
        }
    }

    ///
    ///
    pub fn new_assumption(
        identifier: &str,
        text: &str,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnasmp",
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
    ///
    pub fn new_away_assumption(
        identifier: &str,
        text: &str,
        module: &str,
        module_url: Option<String>,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnawayasmp",
        );
        n.node_type = NodeType::Away(AwayType {
            module: module.to_owned(),
            module_url,
            away_type: AwayNodeType::Assumption,
            mod_height: 0,
        });
        n
    }

    ///
    ///
    ///
    ///
    pub fn new_justification(
        identifier: &str,
        text: &str,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnjust",
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
    ///
    ///
    pub fn new_away_justification(
        identifier: &str,
        text: &str,
        module: &str,
        module_url: Option<String>,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnawayjust",
        );
        n.node_type = NodeType::Away(AwayType {
            module: module.to_owned(),
            module_url,
            away_type: AwayNodeType::Justification,
            mod_height: 0,
        });
        n
    }

    ///
    ///
    ///
    ///
    pub fn new_solution(
        identifier: &str,
        text: &str,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnsltn",
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
    pub fn new_away_solution(
        identifier: &str,
        text: &str,
        module: &str,
        module_url: Option<String>,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnawaysltn",
        );
        n.node_type = NodeType::Away(AwayType {
            module: module.to_owned(),
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
    pub fn new_strategy(
        identifier: &str,
        text: &str,
        undeveloped: bool,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnstgy",
        );
        if undeveloped {
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
    pub fn new_goal(
        identifier: &str,
        text: &str,
        undeveloped: bool,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut classes = classes;
        if undeveloped {
            classes.push("gsn_undeveloped".to_owned());
        }
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsngoal",
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
    ///
    pub fn new_away_goal(
        identifier: &str,
        text: &str,
        module: &str,
        module_url: Option<String>,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnawaygoal",
        );
        n.node_type = NodeType::Away(AwayType {
            module: module.to_owned(),
            module_url,
            away_type: AwayNodeType::Goal,
            mod_height: 0,
        });
        n
    }

    ///
    ///
    ///
    ///
    pub fn new_context(
        identifier: &str,
        text: &str,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnctxt",
        )
    }

    ///
    ///
    ///
    ///
    pub fn new_away_context(
        identifier: &str,
        text: &str,
        module: &str,
        module_url: Option<String>,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnawayctxt",
        );
        n.node_type = NodeType::Away(AwayType {
            module: module.to_owned(),
            module_url,
            away_type: AwayNodeType::Context,
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
        text: &str,
        horizontal_index: Option<HorizontalIndex>,
        rank_increment: Option<usize>,
        url: Option<String>,
        classes: Vec<String>,
    ) -> Self {
        let mut n = Node::new(
            identifier,
            text,
            url,
            classes,
            horizontal_index,
            rank_increment,
            "gsnmodule",
        );
        n.node_type = NodeType::Box(BoxType::Module);
        n
    }
}

///
///
///
///
///
pub(crate) fn setup_basics(id: &str, classes: &[String], url: &Option<String>) -> Element {
    let mut g = Group::new().set("id", escape_node_id(id));
    g = g.set("class", classes.join(" "));
    if let Some(url) = &url {
        let link = Anchor::new();
        link.set("href", escape_url(url.as_str())).add(g).into()
    } else {
        g.into()
    }
}

///
///
///
///
pub(crate) fn add_text(
    mut context: Element,
    text: &str,
    x: i32,
    y: i32,
    font: &FontInfo,
    bold: bool,
) -> Element {
    use svg::node::element::Text;
    let mut text = Text::new()
        .set("x", x)
        .set("y", y)
        .set("font-size", font.size)
        .set("font-family", font.name.as_str())
        .add(svg::node::Text::new(text));
    if bold {
        text = text.set("font-weight", "bold");
    }
    use svg::Node;
    context.append(text);
    context
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_setup_basics() {
        let b = setup_basics("my_id", &[], &None);
        assert_eq!(
            b.get_attributes()["id"].to_string(),
            "node_my_id".to_owned()
        );
    }
}
