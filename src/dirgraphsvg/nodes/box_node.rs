use svg::node::element::{Element, Path, Title, path::Data};

use crate::dirgraphsvg::{
    nodes::{OFFSET_IDENTIFIER, render_text},
    render::{PADDING_HORIZONTAL, PADDING_VERTICAL, create_text},
    util::font::str_line_bounding_box,
};

use super::{SizeContext, SvgNode};

const MODULE_TAB_HEIGHT: i32 = 10;
const UNDEVELOPED_DIAMOND: i32 = 5;
const CONTEXT_BUMP: i32 = 10;

pub(crate) enum BoxType {
    Normal(i32),
    Undeveloped(i32),
    Module,
    Context,
}

impl BoxType {
    ///
    /// Almost arbitrary minimum size.
    ///
    pub(super) fn get_minimum_size(&self) -> (i32, i32) {
        let skew = if let BoxType::Normal(x) = self { *x } else { 0 };
        (
            PADDING_HORIZONTAL * 2 + 90 + skew * 2,
            PADDING_VERTICAL * 2 + 30,
        )
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
        let mut width = std::cmp::max(min_width, size_context.text_width + 2 * PADDING_HORIZONTAL);
        let mut height = std::cmp::max(min_height, size_context.text_height + 2 * PADDING_VERTICAL);
        match &self {
            BoxType::Normal(skew) => {
                width += skew;
            }
            BoxType::Undeveloped(_) => {
                height += UNDEVELOPED_DIAMOND;
            }
            BoxType::Module => {
                height += MODULE_TAB_HEIGHT;
            }
            BoxType::Context => {
                width += CONTEXT_BUMP * 2;
            }
        }

        (width, height)
    }

    ///
    /// Render the node
    ///
    pub(super) fn render(&self, node: &SvgNode, context: &mut Element, border_color: &str) {
        let title = Title::new(&node.identifier);
        use svg::Node;
        context.append(title);

        let data = match &self {
            BoxType::Normal(skew) | BoxType::Undeveloped(skew) => Data::new()
                .move_to((node.x - node.width / 2 + skew / 2, node.y - node.height / 2))
                .line_to((node.x + node.width / 2 + skew / 2, node.y - node.height / 2))
                .line_to((node.x + node.width / 2 - skew / 2, node.y + node.height / 2))
                .line_to((node.x - node.width / 2 - skew / 2, node.y + node.height / 2))
                .close(),
            BoxType::Module => Data::new()
                .move_to((node.x - node.width / 2, node.y - node.height / 2))
                .horizontal_line_by(30)
                .vertical_line_by(MODULE_TAB_HEIGHT)
                .line_to((
                    node.x + node.width / 2,
                    node.y - node.height / 2 + MODULE_TAB_HEIGHT,
                ))
                .line_to((node.x + node.width / 2, node.y + node.height / 2))
                .line_to((node.x - node.width / 2, node.y + node.height / 2))
                .close(),
            BoxType::Context => Data::new()
                .move_to((
                    node.x + CONTEXT_BUMP - node.width / 2,
                    node.y - node.height / 2,
                ))
                .line_to((
                    node.x - CONTEXT_BUMP + node.width / 2,
                    node.y - node.height / 2,
                ))
                .cubic_curve_to((
                    node.x + node.width / 2,
                    node.y - node.height / 2,
                    node.x + node.width / 2,
                    node.y + node.height / 2,
                    node.x + node.width / 2 - CONTEXT_BUMP,
                    node.y + node.height / 2,
                ))
                .line_to((
                    node.x - node.width / 2 + CONTEXT_BUMP,
                    node.y + node.height / 2,
                ))
                .cubic_curve_to((
                    node.x - node.width / 2,
                    node.y + node.height / 2,
                    node.x - node.width / 2,
                    node.y - node.height / 2,
                    node.x - node.width / 2 + CONTEXT_BUMP,
                    node.y - node.height / 2,
                )),
        };

        let border = Path::new()
            .set("fill-opacity", "0")
            .set("stroke", border_color)
            .set("stroke-width", 1u32)
            .set("d", data)
            .set("class", "border");
        context.append(border);

        let skew = if let BoxType::Normal(x) = self { *x } else { 0 };
        let mut x = node.x - (node.width - skew) / 2 + PADDING_HORIZONTAL;
        if let BoxType::Context = self {
            x += CONTEXT_BUMP;
        }
        let mut y = node.y - node.height / 2 + PADDING_VERTICAL;
        if let BoxType::Module = self {
            y += MODULE_TAB_HEIGHT;
        }
        y += str_line_bounding_box("", false).1;
        context.append(create_text(&(&node.identifier).into(), x, y, true));
        y += OFFSET_IDENTIFIER;

        if !node.masked {
            render_text(&node.text, context, Some((skew, node.height)), x, y);
        }

        if let BoxType::Undeveloped(_) = self {
            let data = Data::new()
                .move_to((node.x, node.y + node.height / 2))
                .line_by((UNDEVELOPED_DIAMOND, UNDEVELOPED_DIAMOND))
                .line_by((-UNDEVELOPED_DIAMOND, UNDEVELOPED_DIAMOND))
                .line_by((-UNDEVELOPED_DIAMOND, -UNDEVELOPED_DIAMOND))
                .close();
            let undeveloped_diamond = Path::new()
                .set("fill-opacity", "0")
                .set("stroke", border_color)
                .set("stroke-width", 1u32)
                .set("d", data);
            context.append(undeveloped_diamond);
        }
    }
}
