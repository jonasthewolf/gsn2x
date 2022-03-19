use svg::node::element::{Ellipse, Group, Link, Rectangle, Text, Title};

use crate::FontInfo;

use super::{get_port_default_coordinates, Node, Point2D};

const PADDING: u32 = 5;
const TEXT_OFFSET: u32 = 20;

#[derive(Clone)]
pub struct EllipticalNode {
    identifier: String,
    text: String,
    admonition: Option<String>,
    circle: bool,
    url: Option<String>,
    _classes: Option<Vec<String>>,
    width: u32,
    height: u32,
    lines: Vec<(u32, u32)>,
    x: u32,
    y: u32,
    forced_level: Option<usize>,
}

impl Node for EllipticalNode {
    ///
    /// Width: 5 padding on each side, minimum 50, maximum line length of text or identifier
    /// Height: 5 padding on each side, minimum 30, id line height (max. 20) + height of each text line
    ///
    fn calculate_size(&mut self, font: &FontInfo, suggested_char_wrap: u32) {
        self.height = PADDING * 2 + 50;
        self.width = PADDING * 2 + 50; // Padding of 5 on both sides
        self.text = crate::util::wordwrap::wordwrap(&self.text, suggested_char_wrap, "\n");
        let (t_width, t_height) =
            crate::util::font::text_bounding_box(&font.font, &self.identifier, font.size);
        self.lines.push((t_width, t_height));
        let mut text_height = 0;
        let mut text_width = t_width + PADDING * 2;
        for t in self.text.lines() {
            let (width, height) = crate::util::font::text_bounding_box(&font.font, t, font.size);
            self.lines.push((width, height));
            text_height += height;
            text_width = std::cmp::max(text_width, width + PADDING * 2);
        }
        self.width = std::cmp::max(self.width, (text_width as f32 * 1.141) as u32);
        self.height = std::cmp::max(
            self.height,
            ((PADDING * 2 + TEXT_OFFSET + text_height + 3) as f32 * 1.141) as u32,
        );
        // +3 to make padding at bottom larger
        if self.circle {
            let r_width =
                ((text_width * text_width / 4 + text_height + text_height / 4) as f64).sqrt();
            self.width = r_width as u32 * 2;
            self.height = r_width as u32 * 2;
        }
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

    fn get_id(&self) -> &str {
        self.identifier.as_ref()
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }

    fn get_forced_level(&self) -> Option<usize> {
        self.forced_level
    }

    fn set_forced_level(&mut self, level: usize) {
        self.forced_level = Some(level);
    }

    fn get_coordinates(&self, port: super::Port) -> Point2D {
        get_port_default_coordinates(self.x, self.y, self.width, self.height, port)
    }

    fn render(&mut self, font: &FontInfo) -> svg::node::element::Group {
        let mut g = Group::new(); //.set("id", "").set("class", "");
        if let Some(url) = &self.url {
            let link = Link::new();
            g = g.add(link.set("xlink:href", url.as_str()));
        }

        let title = Title::new().add(svg::node::Text::new(&self.identifier));

        let border = Ellipse::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1u32)
            .set("cx", self.x)
            .set("cy", self.y)
            .set("rx", self.width / 2)
            .set("ry", self.height / 2);

        let id = Text::new()
            .set("x", self.x - self.width / 2 + PADDING + self.width / 8)
            .set(
                "y",
                self.y - self.height / 2 + PADDING + self.lines.get(0).unwrap().1,
            )
            .set("font-weight", "bold")
            .set("font-size", font.size)
            .set("font-family", font.name.to_owned())
            .add(svg::node::Text::new(&self.identifier));

        g = g.add(title).add(border).add(id);
        if let Some(adm) = &self.admonition {
            let decorator = Text::new()
                .set("x", self.x + self.width / 2 - 5)
                .set("y", self.y + self.height / 2 - 5)
                .set("font-weight", "bold")
                .set("font-size", font.size)
                .set("font-family", font.name.to_owned())
                .add(svg::node::Text::new(adm));
            g = g.add(decorator);
        }

        for (n, t) in self.text.lines().enumerate() {
            let text = Text::new()
                .set("x", self.x - self.width / 2 + PADDING)
                .set(
                    "y",
                    self.y - self.height / 2
                        + PADDING
                        + TEXT_OFFSET
                        + (n as u32 + 1) * self.lines.get(n + 1).unwrap().1,
                )
                .set("textLength", self.lines.get(n + 1).unwrap().0)
                .set("font-size", font.size)
                .set("font-family", font.name.to_owned())
                .add(svg::node::Text::new(t));
            g = g.add(text);
        }
        g
    }
}

impl EllipticalNode {
    pub fn new(
        id: &str,
        text: &str,
        admonition: Option<String>,
        circle: bool,
        url: Option<String>,
        classes: Option<Vec<String>>,
        forced_level: Option<usize>,
    ) -> Self {
        EllipticalNode {
            identifier: id.to_string(),
            text: text.to_string(),
            admonition,
            circle,
            url,
            _classes: classes,
            width: 0,
            height: 0,
            lines: vec![],
            x: 0,
            y: 0,
            forced_level,
        }
    }
}
