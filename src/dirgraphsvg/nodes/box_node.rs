use svg::node::element::{path::Data, Path, Text, Title};

use crate::dirgraphsvg::FontInfo;

use super::{get_port_default_coordinates, setup_basics, Node, Point2D, Port};

const PADDING_VERTICAL: i32 = 7;
const PADDING_HORIZONTAL: i32 = 7;
const TEXT_OFFSET: i32 = 20;
const MOUDLE_TAB_HEIGHT: i32 = 10;

pub struct BoxNode {
    identifier: String,
    text: String,
    undeveloped: bool,
    skew: u32,
    url: Option<String>,
    classes: Option<Vec<String>>,
    width: i32,
    height: i32,
    lines: Vec<(i32, i32)>,
    x: i32,
    y: i32,
    is_module_node: bool,
}

impl Node for BoxNode {
    ///
    /// Width: 5 padding on each side, minimum 50, maximum line length of text or identifier
    /// Height: 5 padding on each side, minimum 30, id line height (max. 20) + height of each text line
    ///
    fn calculate_size(&mut self, font: &FontInfo, suggested_char_wrap: u32) {
        self.width = PADDING_HORIZONTAL * 2 + 70 + (self.skew * 2) as i32; // Padding of 5 on both sides
        self.height = PADDING_VERTICAL * 2 + 30; // Padding of 5 on both sides
        self.text =
            crate::dirgraphsvg::util::wordwrap::wordwrap(&self.text, suggested_char_wrap, "\n");
        let (t_width, t_height) = crate::dirgraphsvg::util::font::text_bounding_box(
            &font.font,
            &self.identifier,
            font.size,
        );
        self.lines.push((t_width, t_height));
        let mut text_height = 0;
        let mut text_width = t_width + PADDING_HORIZONTAL * 2;
        for t in self.text.lines() {
            let (width, height) =
                crate::dirgraphsvg::util::font::text_bounding_box(&font.font, t, font.size);
            self.lines.push((width, height));
            text_height += height;
            text_width = std::cmp::max(text_width, width + PADDING_HORIZONTAL * 2 + (self.skew * 2) as i32);
        }
        self.width = std::cmp::max(self.width, text_width);
        self.height = std::cmp::max(
            self.height,
            PADDING_VERTICAL * 2 + TEXT_OFFSET + text_height + 3,
        ); // +3 to make padding at bottom larger
        if self.undeveloped {
            self.height += 5;
        }
        if self.is_module_node {
            self.height += MOUDLE_TAB_HEIGHT;
        }
    }

    fn get_id(&self) -> &str {
        self.identifier.as_ref()
    }

    fn get_width(&self) -> i32 {
        self.width
    }

    fn get_height(&self) -> i32 {
        self.height
    }

    fn get_coordinates(&self, port: &Port) -> Point2D {
        let mut coords =
            get_port_default_coordinates(self.x, self.y, self.width, self.height, port);
        if port == &super::Port::East {
            coords.x -= (self.skew / 2) as i32;
        } else if port == &super::Port::West {
            coords.x += (self.skew / 2) as i32;
        }
        if port == &super::Port::North && self.is_module_node {
            coords.y += MOUDLE_TAB_HEIGHT;
        }
        coords
    }

    fn set_position(&mut self, pos: &Point2D) {
        self.x = pos.x;
        self.y = pos.y;
    }

    fn get_position(&self) -> Point2D {
        Point2D {
            x: self.x,
            y: self.y,
        }
    }

    fn render(&mut self, font: &FontInfo) -> svg::node::element::Element {
        let mut g = setup_basics(&self.identifier, &self.classes, &self.url);

        let title = Title::new().add(svg::node::Text::new(&self.identifier));

        let data = if self.is_module_node {
            Data::new()
                .move_to((self.x - self.width / 2, self.y - self.height / 2))
                .horizontal_line_by(30)
                .vertical_line_by(MOUDLE_TAB_HEIGHT)
                .line_to((
                    self.x + self.width / 2,
                    self.y - self.height / 2 + MOUDLE_TAB_HEIGHT,
                ))
                .line_to((self.x + self.width / 2, self.y + self.height / 2))
                .line_to((self.x - self.width / 2, self.y + self.height / 2))
                .close()
        } else {
            Data::new()
                .move_to((
                    self.x - self.width / 2 + self.skew as i32,
                    self.y - self.height / 2,
                ))
                .line_to((self.x + self.width / 2, self.y - self.height / 2))
                .line_to((
                    self.x + self.width / 2 - self.skew as i32,
                    self.y + self.height / 2,
                ))
                .line_to((self.x - self.width / 2, self.y + self.height / 2))
                .close()
        };

        let border = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1u32)
            .set("d", data)
            .set("class", "border");

        let id = Text::new()
            .set(
                "x",
                self.x - self.width / 2 + PADDING_HORIZONTAL + self.skew as i32,
            )
            .set(
                "y",
                self.y - self.height / 2 + PADDING_VERTICAL + self.lines.get(0).unwrap().1,
            )
            .set("textLength", self.lines.get(0).unwrap().0)
            .set("font-weight", "bold")
            .set("font-size", font.size)
            .set("font-family", font.name.as_str())
            .add(svg::node::Text::new(&self.identifier));

        use svg::Node;
        g.append(title);
        g.append(border);
        g.append(id);

        for (n, t) in self.text.lines().enumerate() {
            let text = Text::new()
                .set("x", self.x - self.width / 2 + PADDING_HORIZONTAL)
                .set(
                    "y",
                    self.y - self.height / 2
                        + PADDING_VERTICAL
                        + TEXT_OFFSET
                        + (n as i32 + 1) * self.lines.get(n + 1).unwrap().1,
                )
                .set("textLength", self.lines.get(n + 1).unwrap().0)
                .set("font-size", font.size)
                .set("font-family", font.name.as_str())
                .add(svg::node::Text::new(t));
            g.append(text);
        }

        if self.undeveloped {
            let data = Data::new()
                .move_to((self.x, self.y + self.height / 2))
                .line_by((5i32, 5i32))
                .line_by((-5i32, 5i32))
                .line_by((-5i32, -5i32))
                .close();
            let undev = Path::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 1u32)
                .set("d", data);
            g.append(undev);
        }
        g
    }

}

impl BoxNode {
    pub fn new(
        id: &str,
        text: &str,
        undeveloped: bool,
        skew: u32,
        is_module_node: bool,
        url: Option<String>,
        classes: Option<Vec<String>>,
    ) -> Self {
        BoxNode {
            identifier: id.to_owned(),
            text: text.to_owned(),
            undeveloped,
            url,
            skew,
            classes,
            width: 0,
            height: 0,
            lines: vec![],
            x: 0,
            y: 0,
            is_module_node,
        }
    }
}
