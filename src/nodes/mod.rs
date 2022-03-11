use crate::FontInfo;

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

pub struct Point2D {
    pub x: u32,
    pub y: u32,
}

pub trait Node {
    fn get_id(&self) -> &str;
    fn calculate_size(&mut self, font: &FontInfo, suggested_char_wrap: u32);
    fn set_vertical_rank(&mut self, vertical_rank: usize);
    fn set_horizontal_rank(&mut self, horizontal_rank: usize);
    fn get_vertical_rank(&self) -> usize;
    fn get_horizontal_rank(&self) -> usize;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_coordinates(&self, port: Port) -> Point2D;
    fn get_forced_level(&self) -> u32;
    fn render(&mut self, x: u32, y: u32, font: &FontInfo) -> svg::node::element::Group;
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
            Port::North => x + width / 2,
            Port::East => x + width,
            Port::South => x + width / 2,
            Port::West => x,
        },
        y: match port {
            Port::North => y,
            Port::East => y + height / 2,
            Port::South => y + height,
            Port::West => y + height / 2,
        },
    }
}
