use svg::node::element::{path::Data, Element, Path, Title};

use crate::dirgraphsvg::{
    nodes::OFFSET_IDENTIFIER,
    render::{create_text, PADDING_HORIZONTAL, PADDING_VERTICAL},
    util::font::{text_bounding_box, FontInfo},
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
    ///
    ///
    pub(super) fn get_minimum_size(&self) -> (i32, i32) {
        let skew = if let BoxType::Normal(x) = self { *x } else { 0 };
        (
            PADDING_HORIZONTAL * 2 + 90 + skew * 2,
            PADDING_VERTICAL * 2 + 30,
        )
    }

    ///
    ///
    ///
    pub(super) fn calculate_size(
        &self,
        _font: &FontInfo,
        min_width: i32,
        min_height: i32,
        size_context: &mut SizeContext,
    ) -> (i32, i32) {
        let mut width = std::cmp::max(min_width, size_context.text_width + 2 * PADDING_HORIZONTAL);
        let mut height = std::cmp::max(min_height, size_context.text_height + 2 * PADDING_VERTICAL);
        match &self {
            BoxType::Normal(_) => (),
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
    ///
    ///
    ///
    pub(super) fn render(
        &self,
        node: &SvgNode,
        font: &FontInfo,
        context: &mut Element,
        border_color: &str,
    ) {
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
        y += font.size as i32;
        context.append(create_text(&node.identifier, x, y, font, true));
        y += OFFSET_IDENTIFIER;

        if !node.masked {
            for text in node.text.lines() {
                let text_bb = text_bounding_box(font, text, false);
                y += text_bb.1;
                x -= skew * text_bb.1 / node.height;
                context.append(create_text(text, x, y, font, false));
            }
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
