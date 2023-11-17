use core::fmt::Debug;
use std::fmt::Display;

///
/// Two dimensional point representation
///
#[derive(Copy, Clone)]
pub struct Point2D {
    pub x: i32,
    pub y: i32,
}

impl Debug for Point2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{},{}", self.x, self.y))
    }
}

impl Display for Point2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{},{}", self.x, self.y))
    }
}

impl Point2D {
    ///
    /// Move the point by `x` and `y` relatively.
    ///
    pub fn move_relative(&self, x: i32, y: i32) -> Self {
        Point2D {
            x: self.x + x,
            y: self.y + y,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn move_rel() {
        let p = Point2D { x: 0, y: 0 }.move_relative(5, 3);
        assert_eq!(p.x, 5);
        assert_eq!(p.y, 3);
    }
}
