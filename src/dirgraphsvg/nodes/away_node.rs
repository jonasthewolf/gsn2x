use svg::node::element::{Anchor, Element, Path, Title, Use, path::Data};

use crate::dirgraphsvg::{
    nodes::OFFSET_IDENTIFIER,
    render::{PADDING_HORIZONTAL, PADDING_VERTICAL, create_text},
    util::{
        escape_url,
        font::{str_line_bounding_box, text_line_bounding_box},
    },
};

use super::{SizeContext, SvgNode};

const MODULE_IMAGE: i32 = 20;

pub enum AwayNodeType {
    Goal,
    Solution,
    Context,
    Assumption,
    Justification,
}

pub(crate) struct AwayType {
    pub(super) module: String,
    pub(super) module_url: Option<String>,
    pub(super) away_type: AwayNodeType,
    pub(super) mod_height: i32,
}

///
///         -+----             ...........
///          |             .....         .....
///   addon  |           ...                 ...
///   height |        ....                     ....
///          |       ..                           ..
///          +---    +------------------------------+
///     Pad  |       |                              |
///  y_start +--     |  +------------------------+  |
///          |       |  |                        |  |
///          |       |  |                        |  |
///    text  |       |  |                        |  |
///   height |       |  |                        |  |
///          |       |  |                        |  |
///          |       |  |                        |  |
///          |       |  |                        |  |
///          +--     |  +------------------------+  |
///     Pad  |       |                              |
/// y_module +---    +------------------------------+
///     Pad  |       |                              |
///          +--     |  +------------------------+  |
///   mod    |       |  |                        |  |
///   height |       |  |  XXXX                  |  |
///          |       |  |                        |  |
///          +--     |  +------------------------+  |
///     Pad  |       |                              |
///         -+----   +------------------------------+
///    
///                  | Pad                      Pad |
///                  +--+------------------------+--+
///                  |           width              |
///
impl AwayType {
    ///
    /// Almost arbitrary minimum size.
    ///
    pub(super) fn get_minimum_size(&self) -> (i32, i32) {
        (PADDING_HORIZONTAL * 2 + 70, PADDING_VERTICAL * 2 + 30)
    }

    ///
    /// Calculate minimums size of Away node based on text context and padding.
    ///
    pub(super) fn calculate_size(
        &self,

        min_width: i32,
        min_height: i32,
        size_context: &mut SizeContext,
    ) -> (i32, i32) {
        // no wrapping of module names
        let (mod_width, _) = str_line_bounding_box(&self.module, false);

        let width = *[
            min_width,
            size_context.text_width + PADDING_HORIZONTAL * 2,
            mod_width + MODULE_IMAGE + PADDING_HORIZONTAL * 3, // Padding + Module Image + Padding + Module text + Padding
        ]
        .iter()
        .max()
        .unwrap();
        let addon_height = self.get_addon_height(width);
        let mut height = std::cmp::max(
            min_height,
            PADDING_VERTICAL * 2
                + size_context.text_height
                + std::cmp::max(self.mod_height, MODULE_IMAGE)
                + PADDING_VERTICAL * 2,
        );
        height += addon_height;
        (width, height)
    }

    ///
    /// Get the height add on based on type.
    ///
    ///
    fn get_addon_height(&self, width: i32) -> i32 {
        match self.away_type {
            AwayNodeType::Goal => 0,
            AwayNodeType::Solution => (width as f32 * 0.5) as i32,
            AwayNodeType::Context => (width as f32 * 0.1) as i32,
            AwayNodeType::Assumption => (width as f32 * 0.25) as i32,
            AwayNodeType::Justification => (width as f32 * 0.25) as i32,
        }
    }

    ///
    /// Render the node
    ///
    pub(super) fn render(&self, node: &SvgNode, context: &mut Element, border_color: &str) {
        let title = Title::new(&node.identifier);

        use svg::Node;
        context.append(title);

        let addon_height = self.get_addon_height(node.width);

        let y_module = node.y + node.height / 2 - 2 * PADDING_VERTICAL - self.mod_height;
        let y_id = node.y - node.height / 2 + addon_height + PADDING_VERTICAL;

        let data = match self.away_type {
            AwayNodeType::Goal => Data::new()
                .move_to((node.x - node.width / 2, y_module))
                .vertical_line_to(node.y - node.height / 2)
                .horizontal_line_to(node.x + node.width / 2)
                .vertical_line_to(y_module),
            AwayNodeType::Solution | AwayNodeType::Assumption | AwayNodeType::Justification => {
                Data::new()
                    .move_to((node.x - node.width / 2, y_module))
                    .vertical_line_to(node.y - node.height / 2 + addon_height)
                    .elliptical_arc_to((
                        node.width / 2,
                        addon_height,
                        0,
                        0,
                        1,
                        node.x + node.width / 2,
                        node.y - node.height / 2 + addon_height,
                    ))
                    .vertical_line_to(y_module)
            }
            AwayNodeType::Context => Data::new()
                .move_to((node.x - node.width / 2, y_module))
                .vertical_line_to(node.y - node.height / 2 + addon_height)
                .cubic_curve_to((
                    node.x - node.width / 2,
                    node.y - node.height / 2 + addon_height / 2,
                    node.x - node.width / 2 + addon_height / 2,
                    node.y - node.height / 2,
                    node.x - node.width / 2 + addon_height,
                    node.y - node.height / 2,
                ))
                .horizontal_line_by(node.width - 2 * addon_height)
                .cubic_curve_to((
                    node.x + node.width / 2 - addon_height / 2,
                    node.y - node.height / 2,
                    node.x + node.width / 2,
                    node.y - node.height / 2 + addon_height / 2,
                    node.x + node.width / 2,
                    node.y - node.height / 2 + addon_height,
                ))
                .vertical_line_to(y_module),
        };

        let upper_line = Path::new()
            .set("fill-opacity", "0")
            .set("stroke", border_color)
            .set("stroke-width", 1u32)
            .set("d", data)
            .set("class", "border");
        context.append(upper_line);

        let x = node.x - node.width / 2 + PADDING_HORIZONTAL;
        let mut y = y_id + str_line_bounding_box("", false).1;
        // Identifier
        context.append(create_text(&(&node.identifier).into(), x, y, true));
        y += OFFSET_IDENTIFIER;

        // Text
        if !node.masked {
            for text in node.text.lines() {
                y += text_line_bounding_box(text, false).1;
                context.append(create_text(&text.into(), x, y, false));
            }
        }

        // It is a box to be able to add a link to it
        let module_box_data = Data::new()
            .move_to((
                node.x - node.width / 2,
                node.y + node.height / 2 - (2 * PADDING_VERTICAL + self.mod_height),
            ))
            .horizontal_line_by(node.width)
            .vertical_line_by(2 * PADDING_VERTICAL + self.mod_height)
            .horizontal_line_by(-node.width)
            .close();

        let module_box = Path::new()
            .set("fill-opacity", "0")
            .set("stroke", border_color)
            .set("stroke-width", 1u32)
            .set("d", module_box_data)
            .set("class", "border");

        // Module text and links
        let module_text = if node.masked {
            None
        } else {
            Some(create_text(
                &(&self.module).into(),
                node.x - node.width / 2 + PADDING_HORIZONTAL + MODULE_IMAGE + PADDING_HORIZONTAL,
                node.y + node.height / 2 - PADDING_VERTICAL,
                true,
            ))
        };
        if let Some(module_url) = &self.module_url {
            let mut module_link = Anchor::new()
                .set("href", escape_url(module_url.as_str()))
                .add(module_box);
            if let Some(module_text) = module_text {
                module_link.append(module_text);
            }
            context.append(module_link);
        } else {
            if let Some(module_text) = module_text {
                context.append(module_text);
            }
            context.append(module_box);
        }
        // Module icon
        context.append(
            Use::new()
                .set("href", "#module_icon")
                .set("x", node.x - node.width / 2 + PADDING_HORIZONTAL)
                .set(
                    "y",
                    node.y + node.height / 2 - self.mod_height - PADDING_VERTICAL,
                ),
        );

        // Add admonition letter
        let admonition = match self.away_type {
            AwayNodeType::Assumption => Some("A"),
            AwayNodeType::Justification => Some("J"),
            _ => None,
        };
        if let Some(adm) = admonition {
            context.append(create_text(
                &adm.into(),
                node.x + node.width / 2 - PADDING_HORIZONTAL,
                node.y - node.height / 2,
                true,
            ));
        }
    }
}
