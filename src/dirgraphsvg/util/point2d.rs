pub struct Point2D {
    pub x: u32,
    pub y: u32,
}

impl Point2D {
    pub fn move_relative(&self, x: i32, y: i32) -> Self {
        Point2D {
            x: (self.x as i32 + x) as u32,
            y: (self.y as i32 + y) as u32,
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
