use svg::node::element::{Ellipse, Group, Link, Text, Title};

use crate::FontInfo;

use super::{get_port_default_coordinates, Node, Point2D};

const PADDING: u32 = 5;
const TEXT_OFFSET: u32 = 20;
const MIN_SIZE: u32 = 50;

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
    text_width: u32,
    text_height: u32,
    lines: Vec<(u32, u32)>,
    x: u32,
    y: u32,
    forced_level: Option<usize>,
}

impl Node for EllipticalNode {
    ///
    ///
    fn calculate_size(&mut self, font: &FontInfo, suggested_char_wrap: u32) {
        // Wrap text
        self.text = crate::util::wordwrap::wordwrap(&self.text, suggested_char_wrap, "\n");
        // Calculate bounding box of identifier
        let (t_width, t_height) =
            crate::util::font::text_bounding_box(&font.font, &self.identifier, font.size);
        self.lines.push((t_width, t_height));
        // +3 to make padding at bottom larger
        self.text_height = t_height + TEXT_OFFSET + 3;
        self.text_width = t_width;
        for t in self.text.lines() {
            let (line_width, line_height) =
                crate::util::font::text_bounding_box(&font.font, t, font.size);
            self.lines.push((line_width, line_height));
            self.text_height += line_height;
            self.text_width = std::cmp::max(self.text_width, line_width);
        }
        if self.circle {
            let r_width = ((self.text_width * self.text_width / 4
                + self.text_height
                + self.text_height / 4) as f64)
                .sqrt();
            self.width = std::cmp::max(MIN_SIZE, (2 * PADDING + r_width as u32) * 2);
            self.height = std::cmp::max(MIN_SIZE, (2 * PADDING + r_width as u32) * 2);
        } else {
            self.width = std::cmp::max(
                MIN_SIZE,
                PADDING * 2 + ((self.text_width as f32 * 1.414) as u32),
            );
            self.height = std::cmp::max(
                self.height,
                PADDING * 2 + ((self.text_height as f32 * 1.414) as u32),
            );
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

    fn get_coordinates(&self, port: &super::Port) -> Point2D {
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
            .set("x", self.x - self.text_width / 2)
            .set(
                "y",
                self.y - self.text_height / 2 + self.lines.get(0).unwrap().1,
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

        let mut text_y = self.y - self.text_height / 2 + TEXT_OFFSET;
        for (n, t) in self.text.lines().enumerate() {
            text_y += self.lines.get(n + 1).unwrap().1;
            let text = Text::new()
                .set("x", self.x - self.text_width / 2)
                .set("y", text_y)
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
            text_width: 0,
            text_height: 0,
            lines: vec![],
            x: 0,
            y: 0,
            forced_level,
        }
    }
}
