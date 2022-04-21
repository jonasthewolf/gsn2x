use std::{cell::RefCell, rc::Rc};

use svg::node::element::Group;

use crate::dirgraphsvg::util::point2d::Point2D;

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

    fn calculate_size(&mut self, _: &crate::dirgraphsvg::FontInfo, _: u32) {
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

    fn get_coordinates(&self, _: &super::Port) -> crate::dirgraphsvg::util::point2d::Point2D {
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

    fn render(&mut self, _: &crate::dirgraphsvg::FontInfo) -> svg::node::element::Element {
        Group::new().into() // Empty groups are not rendered.
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

impl<'a> From<&Rc<RefCell<dyn Node>>> for InvisibleNode {
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

#[cfg(test)]
mod test {

    use rusttype::Font;

    use super::*;
    use crate::dirgraphsvg::FontInfo;

    #[test]
    fn justify() {
        let font_bytes = vec![
            0, 1, 0, 0, 0, 10, 0, 128, 0, 3, 0, 32, 100, 117, 109, 49, 0, 0, 0, 0, 0, 0, 0, 172, 0,
            0, 0, 2, 99, 109, 97, 112, 0, 12, 0, 96, 0, 0, 0, 176, 0, 0, 0, 44, 103, 108, 121, 102,
            53, 115, 99, 161, 0, 0, 0, 220, 0, 0, 0, 20, 104, 101, 97, 100, 7, 157, 81, 54, 0, 0,
            0, 240, 0, 0, 0, 54, 104, 104, 101, 97, 0, 164, 3, 249, 0, 0, 1, 40, 0, 0, 0, 36, 104,
            109, 116, 120, 4, 68, 0, 10, 0, 0, 1, 76, 0, 0, 0, 8, 108, 111, 99, 97, 0, 10, 0, 0, 0,
            0, 1, 84, 0, 0, 0, 6, 109, 97, 120, 112, 0, 4, 0, 3, 0, 0, 1, 92, 0, 0, 0, 32, 110, 97,
            109, 101, 0, 68, 16, 175, 0, 0, 1, 124, 0, 0, 0, 56, 100, 117, 109, 50, 0, 0, 0, 0, 0,
            0, 1, 180, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 3, 0, 1, 0, 0, 0, 12, 0, 4, 0, 32, 0,
            0, 0, 4, 0, 4, 0, 1, 0, 0, 0, 45, 255, 255, 0, 0, 0, 45, 255, 255, 255, 212, 0, 1, 0,
            0, 0, 0, 0, 1, 0, 10, 0, 0, 0, 58, 0, 56, 0, 2, 0, 0, 51, 35, 53, 58, 48, 56, 0, 1, 0,
            0, 0, 1, 0, 0, 23, 194, 213, 22, 95, 15, 60, 245, 0, 11, 0, 64, 0, 0, 0, 0, 207, 21,
            56, 6, 0, 0, 0, 0, 217, 38, 219, 189, 0, 10, 0, 0, 0, 58, 0, 56, 0, 0, 0, 6, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 76, 255, 236, 0, 18, 4, 0, 0, 10, 0, 10, 0, 58, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 4, 0, 0, 0, 0, 68, 0, 10, 0, 0, 0, 0, 0,
            10, 0, 0, 0, 1, 0, 0, 0, 2, 0, 3, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        // The following functions must be implemented for a trait, but cannot be used for Invisible node.
        // Thus, this test tests nothing.
        let mut n = InvisibleNode {
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            id: "".to_owned(),
        };
        n.calculate_size(
            &FontInfo {
                font: Font::try_from_vec(font_bytes).unwrap(),
                name: "".to_owned(),
                size: 12.0,
            },
            0,
        );
        n.set_forced_level(1);
        assert_eq!(n.get_forced_level(), None);
    }

    #[test]
    fn test_set_size() {
        let mut node = InvisibleNode {
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            id: "".to_owned(),
        };
        node.set_size(3, 5);
        assert_eq!(node.get_width(), 3);
        assert_eq!(node.get_height(), 5);
    }
}
