use crate::{util::point2d::Point2D, FontInfo};

use self::{box_node::BoxNode, elliptical_node::EllipticalNode};

mod box_node;
pub mod context;
mod elliptical_node;
pub(crate) mod invisible_node;

pub enum Port {
    North,
    East,
    South,
    West,
}

pub trait Node {
    fn get_id(&self) -> &str;
    fn calculate_size(&mut self, font: &FontInfo, suggested_char_wrap: u32);
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn set_position(&mut self, pos: &Point2D);
    fn get_position(&self) -> Point2D;
    fn get_coordinates(&self, port: Port) -> Point2D;
    fn get_forced_level(&self) -> Option<usize>;
    fn set_forced_level(&mut self, level: usize);
    fn render(&mut self, font: &FontInfo) -> svg::node::element::Group;
}

pub(crate) fn get_port_default_coordinates(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    port: Port,
) -> Point2D {
    Point2D {
        x: match port {
            Port::North => x,
            Port::East => x + width / 2,
            Port::South => x,
            Port::West => x - width / 2,
        },
        y: match port {
            Port::North => y - height / 2,
            Port::East => y,
            Port::South => y + height / 2,
            Port::West => y,
        },
    }
}

pub fn new_assumption(
    id: &str,
    text: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
    forced_level: Option<usize>,
) -> EllipticalNode {
    EllipticalNode::new(
        id,
        text,
        Some("A".to_owned()),
        false,
        url,
        classes,
        forced_level,
    )
}

pub fn new_justification(
    id: &str,
    text: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
    forced_level: Option<usize>,
) -> EllipticalNode {
    EllipticalNode::new(
        id,
        text,
        Some("J".to_owned()),
        false,
        url,
        classes,
        forced_level,
    )
}

pub fn new_solution(
    id: &str,
    text: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
    forced_level: Option<usize>,
) -> EllipticalNode {
    EllipticalNode::new(id, text, None, true, url, classes, forced_level)
}

pub fn new_strategy(
    id: &str,
    text: &str,
    undeveloped: bool,
    url: Option<String>,
    classes: Option<Vec<String>>,
    forced_level: Option<usize>,
) -> BoxNode {
    BoxNode::new(id, text, undeveloped, 15, url, classes, forced_level)
}

pub fn new_goal(
    id: &str,
    text: &str,
    undeveloped: bool,
    url: Option<String>,
    classes: Option<Vec<String>>,
    forced_level: Option<usize>,
) -> BoxNode {
    BoxNode::new(id, text, undeveloped, 0, url, classes, forced_level)
}
