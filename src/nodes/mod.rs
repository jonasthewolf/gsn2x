use crate::{util::point2d::Point2D, FontInfo};

pub mod assumption;
mod box_node;
pub mod context;
mod elliptical_node;
pub mod goal;
pub mod justification;
pub mod solution;
pub mod strategy;

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
    fn get_forced_level(&self) -> Option<u32>;
    fn set_forced_level(&mut self, level: u32);
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
