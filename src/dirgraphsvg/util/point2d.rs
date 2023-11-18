use core::fmt::Debug;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, Sub},
};

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

impl AddAssign for Point2D {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add<Point2D> for Point2D {
    type Output = Point2D;

    fn add(self, rhs: Point2D) -> Self::Output {
        Point2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<(i32, i32)> for Point2D {
    type Output = Point2D;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        Point2D {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

impl Sub<Point2D> for Point2D {
    type Output = Point2D;

    fn sub(self, rhs: Point2D) -> Self::Output {
        Point2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<i32> for Point2D {
    type Output = Point2D;

    fn mul(self, rhs: i32) -> Self::Output {
        Point2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl From<(i32, i32)> for Point2D {
    fn from(value: (i32, i32)) -> Self {
        Point2D {
            x: value.0,
            y: value.1,
        }
    }
}

#[cfg(test)]
mod test {}
