use std::{cell::RefCell, rc::Rc};

use svg::node::element::Group;

use crate::util::point2d::Point2D;

use super::Node;

pub struct InvisibleNode {
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    id: String,
}

impl Node for InvisibleNode {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn calculate_size(&mut self, _: &crate::FontInfo, _: u32) {
        // Intentionally left empty
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
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

    fn get_coordinates(&self, _: &super::Port) -> crate::util::point2d::Point2D {
        Point2D {
            x: self.x,
            y: self.y,
        }
    }

    fn get_forced_level(&self) -> Option<usize> {
        None
    }

    fn set_forced_level(&mut self, _: usize) {
        // Intentionally left emtpy
    }

    fn render(&mut self, _: &crate::FontInfo) -> svg::node::element::Group {
        Group::new() // Empty groups are not rendered.
    }
}

impl InvisibleNode {
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_owned();
    }
}

impl From<&Rc<RefCell<dyn Node>>> for InvisibleNode {
    fn from(n: &Rc<RefCell<dyn Node>>) -> Self {
        let n = n.borrow();
        InvisibleNode {
            id: if n.get_id().starts_with("__invisible__node__") {
                let (node_name, num) = n.get_id().rsplit_once('-').unwrap_or((n.get_id(), "0"));
                format!("{}-{}", node_name, num.parse::<u32>().unwrap() + 1)
            } else {
                format!("__invisible__node__{}", n.get_id())
            },
            width: n.get_width(),
            height: n.get_height(),
            x: n.get_position().x,
            y: n.get_position().y,
        }
    }
}
