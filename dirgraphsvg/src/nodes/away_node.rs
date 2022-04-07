use svg::node::element::{path::Data, Path, Rectangle, Text, Title, Use};

use crate::FontInfo;

use super::{get_port_default_coordinates, setup_basics, Node, Point2D, Port};

const PADDING_VERTICAL: u32 = 7;
const PADDING_HORIZONTAL: u32 = 7;
const TEXT_OFFSET: u32 = 20;
const MODULE_IMAGE: u32 = 20;

pub enum AwayType {
    Goal,
    Solution,
    Context,
    Assumption,
    Justification,
}

pub struct AwayNode {
    identifier: String,
    text: String,
    module: String,
    node_type: AwayType,
    url: Option<String>,
    classes: Option<Vec<String>>,
    width: u32,
    height: u32,
    lines: Vec<(u32, u32)>,
    x: u32,
    y: u32,
    forced_level: Option<usize>,
    mod_width: u32,
    mod_height: u32,
    addon_height: u32,
}

impl Node for AwayNode {
    ///
    /// Width: 5 padding on each side, minimum 50, maximum line length of text or identifier
    /// Height: 5 padding on each side, minimum 30, id line height (max. 20) + height of each text line
    ///
    fn calculate_size(&mut self, font: &FontInfo, suggested_char_wrap: u32) {
        self.width = 70; // Padding of 5 on both sides
        self.height = PADDING_VERTICAL * 2 + 30; // Padding of 5 on both sides
        self.text = crate::util::wordwrap::wordwrap(&self.text, suggested_char_wrap, "\n");
        let (t_width, t_height) =
            crate::util::font::text_bounding_box(&font.font, &self.identifier, font.size);
        self.lines.push((t_width, t_height));
        let mut text_height = 0;
        let mut text_width = t_width;
        for t in self.text.lines() {
            let (width, height) = crate::util::font::text_bounding_box(&font.font, t, font.size);
            self.lines.push((width, height));
            text_height += height;
            text_width = std::cmp::max(text_width, width);
        }
        let (mod_width, mod_height) =
            crate::util::font::text_bounding_box(&font.font, &self.module, font.size);
        self.mod_width = mod_width;
        self.mod_height = mod_height;
        self.width = *[
            self.width,
            text_width,
            mod_width + MODULE_IMAGE + PADDING_HORIZONTAL,
        ]
        .iter()
        .max()
        .unwrap()
            + PADDING_HORIZONTAL * 2;
        self.height = std::cmp::max(
            self.height,
            PADDING_VERTICAL * 4 + TEXT_OFFSET + text_height + 3 + self.mod_height,
        ); // +3 to make padding at bottom larger
        self.addon_height = match self.node_type {
            AwayType::Goal => 0,
            AwayType::Solution => (self.width as f32 * 0.5) as u32,
            AwayType::Context => (self.width as f32 * 0.1) as u32,
            AwayType::Assumption => (self.width as f32 * 0.25) as u32,
            AwayType::Justification => (self.width as f32 * 0.25) as u32,
        };
        self.height += self.addon_height;
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

    fn get_coordinates(&self, port: &Port) -> Point2D {
        get_port_default_coordinates(self.x, self.y, self.width, self.height, port)
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

    fn render(&mut self, font: &FontInfo) -> svg::node::element::Group {
        let mut g = setup_basics(&self.identifier, &self.classes, &self.url);

        let title = Title::new().add(svg::node::Text::new(&self.identifier));

        let start_y = self.y + self.height / 2 - 2 * PADDING_VERTICAL - self.mod_height;
        let start_id = self.y + self.addon_height - self.height / 2 + PADDING_VERTICAL;

        let data = match self.node_type {
            AwayType::Goal => Data::new()
                .move_to((self.x - self.width / 2, start_y))
                .vertical_line_to(self.y - self.height / 2)
                .horizontal_line_to(self.x + self.width / 2)
                .vertical_line_to(start_y),
            AwayType::Solution | AwayType::Assumption | AwayType::Justification => Data::new()
                .move_to((self.x - self.width / 2, start_y))
                .vertical_line_to(self.y - self.height / 2 + self.addon_height)
                .elliptical_arc_to((
                    self.width / 2,
                    self.addon_height,
                    0,
                    0,
                    1,
                    self.x + self.width / 2,
                    self.y - self.height / 2 + self.addon_height,
                ))
                .vertical_line_to(start_y),
            AwayType::Context => Data::new()
                .move_to((self.x - self.width / 2, start_y))
                .vertical_line_to(self.y - self.height / 2 + self.addon_height)
                .cubic_curve_to((
                    self.x - self.width / 2,
                    self.y - self.height / 2 + self.addon_height / 2,
                    self.x - self.width / 2 + self.addon_height / 2,
                    self.y - self.height / 2,
                    self.x - self.width / 2 + self.addon_height,
                    self.y - self.height / 2,
                ))
                .horizontal_line_by(self.width - 2 * self.addon_height)
                .cubic_curve_to((
                    self.x + self.width / 2 - self.addon_height / 2,
                    self.y - self.height / 2,
                    self.x + self.width / 2,
                    self.y - self.height / 2 + self.addon_height / 2,
                    self.x + self.width / 2,
                    self.y - self.height / 2 + self.addon_height,
                ))
                .vertical_line_to(start_y),
        };

        let upper_line = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1u32)
            .set("d", data)
            .set("class", "border");

        let module_box = Rectangle::new()
            .set("x", self.x - self.width / 2)
            .set(
                "y",
                self.y + self.height / 2 - 2 * PADDING_VERTICAL - self.mod_height,
            )
            .set("width", self.width)
            .set("height", 2 * PADDING_VERTICAL + self.mod_height)
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1u32);

        let module_text = Text::new()
            .set(
                "x",
                self.x - self.width / 2 + 2 * PADDING_HORIZONTAL + MODULE_IMAGE,
            )
            .set("y", self.y + self.height / 2 - PADDING_VERTICAL)
            .set("textLength", self.mod_width)
            .set("font-weight", "bold")
            .set("font-size", font.size)
            .set("font-family", font.name.as_str())
            .add(svg::node::Text::new(&self.module));

        let id = Text::new()
            .set("x", self.x - self.width / 2 + PADDING_HORIZONTAL)
            .set("y", start_id + self.lines.get(0).unwrap().1)
            .set("font-weight", "bold")
            .set("font-size", font.size)
            .set("font-family", font.name.as_str())
            .add(svg::node::Text::new(&self.identifier));

        g = g
            .add(title)
            .add(module_box)
            .add(module_text)
            .add(upper_line)
            .add(id)
            .add(
                Use::new()
                    .set("href", "#module_icon")
                    .set("x", self.x - self.width / 2 + PADDING_HORIZONTAL)
                    .set(
                        "y",
                        self.y + self.height / 2 - self.mod_height - PADDING_VERTICAL,
                    ),
            );

        let admonition = match self.node_type {
            AwayType::Assumption => Some("A"),
            AwayType::Justification => Some("J"),
            _ => None,
        };
        if let Some(adm) = admonition {
            let decorator = Text::new()
                .set("x", self.x + self.width / 2 - PADDING_HORIZONTAL)
                .set("y", self.y - self.height / 2 + self.mod_height)
                .set("font-weight", "bold")
                .set("font-size", font.size)
                .set("font-family", font.name.as_str())
                .add(svg::node::Text::new(adm));
            g = g.add(decorator);
        }

        for (n, t) in self.text.lines().enumerate() {
            let text = Text::new()
                .set("x", self.x - self.width / 2 + PADDING_HORIZONTAL)
                .set(
                    "y",
                    start_id + TEXT_OFFSET + (n as u32 + 1) * self.lines.get(n + 1).unwrap().1,
                )
                .set("textLength", self.lines.get(n + 1).unwrap().0)
                .set("font-size", font.size)
                .set("font-family", font.name.as_str())
                .add(svg::node::Text::new(t));
            g = g.add(text);
        }

        g
    }

    fn get_forced_level(&self) -> Option<usize> {
        self.forced_level
    }

    fn set_forced_level(&mut self, level: usize) {
        self.forced_level = Some(level);
    }
}

impl AwayNode {
    pub fn new(
        id: &str,
        text: &str,
        module: &str,
        node_type: AwayType,
        url: Option<String>,
        classes: Option<Vec<String>>,
    ) -> Self {
        AwayNode {
            identifier: id.to_owned(),
            text: text.to_owned(),
            url,
            classes,
            width: 0,
            height: 0,
            lines: vec![],
            x: 0,
            y: 0,
            forced_level: None,
            module: module.to_owned(),
            node_type,
            mod_width: 0,
            mod_height: 0,
            addon_height: 0,
        }
    }
}
