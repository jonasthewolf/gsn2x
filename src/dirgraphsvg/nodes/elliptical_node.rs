use svg::node::element::{path::Data, Element, Path, Title};

use crate::dirgraphsvg::{
    nodes::{add_text, OFFSET_IDENTIFIER},
    util::font::FontInfo,
};

use super::{Node, SizeContext, PADDING_HORIZONTAL};

pub(crate) struct EllipticalType {
    pub(super) admonition: Option<String>,
    pub(super) circle: bool,
    pub(super) text_width: i32,
    pub(super) text_height: i32,
}

impl EllipticalType {
    ///
    ///
    ///
    ///
    pub(super) fn get_minimum_size(&self) -> (i32, i32) {
        (PADDING_HORIZONTAL * 2 + 40, PADDING_HORIZONTAL * 2 + 40)
    }

    ///
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
        if self.circle {
            let r_width = (((size_context.text_width * size_context.text_width
                + size_context.text_height * size_context.text_height)
                as f64)
                .sqrt()
                / 2.0) as i32;
            (
                std::cmp::max(min_width, (2 * PADDING_HORIZONTAL + r_width) * 2),
                std::cmp::max(min_height, (2 * PADDING_HORIZONTAL + r_width) * 2),
            )
        } else {
            (
                std::cmp::max(
                    min_width,
                    PADDING_HORIZONTAL * 4 + ((size_context.text_width as f32 * 1.414) as i32),
                ),
                std::cmp::max(
                    min_height,
                    PADDING_HORIZONTAL * 4 + ((size_context.text_height as f32 * 1.414) as i32),
                ),
            )
        }
    }

    ///
    ///
    ///
    ///
    pub(super) fn render(&self, node: &Node, font: &FontInfo, mut context: Element) -> Element {
        let title = Title::new().add(svg::node::Text::new(&node.identifier));

        let data = Data::new()
            .move_to((node.x - node.width / 2, node.y))
            .elliptical_arc_by((
                node.width / 2,  // rx
                node.height / 2, // ry
                0,               // x-axis-rotation
                1,               // large-arc-flag
                0,               // sweep-flag
                node.width,
                0,
            ))
            .elliptical_arc_by((
                node.width / 2,  // rx
                node.height / 2, // ry
                0,               // x-axis-rotation
                1,               // large-arc-flag
                0,               // sweep-flag
                - node.width,
                0,
            ))
            .close();

        let border = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1u32)
            .set("d", data)
            .set("class", "border");

        use svg::Node;
        context.append(title);
        context.append(border);

        let x = node.x - self.text_width / 2;
        let mut y = node.y - self.text_height / 2 + PADDING_HORIZONTAL;
        context = add_text(context, &node.identifier, x, y, font, true);

        y += OFFSET_IDENTIFIER;
        for text in node.text.lines() {
            y += font.size as i32;
            context = add_text(context, text, x, y, font, false);
        }

        if let Some(adm) = &self.admonition {
            context = add_text(
                context,
                adm,
                node.x + node.width / 2 - 5,
                node.y + node.height / 2 - 5,
                font,
                true,
            );
        }

        context
    }
}
