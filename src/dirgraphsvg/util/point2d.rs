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
