use core::fmt::Debug;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, Sub},
};

///
/// Two dimensional point representation
///
#[derive(Copy, Clone)]
pub struct Point2D<T>
where
    T: Sized + Add + Mul,
{
    pub x: T,
    pub y: T,
}

impl<T> Debug for Point2D<T>
where
    T: Sized + Add + Mul + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?},{:?}", self.x, self.y))
    }
}

impl<T> Display for Point2D<T>
where
    T: Sized + Add + Mul + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{},{}", self.x, self.y))
    }
}

impl<T> AddAssign for Point2D<T>
where
    T: Sized + Add + Mul + AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T> Add<Point2D<T>> for Point2D<T>
where
    T: Sized + Add<Output = T> + Mul,
{
    type Output = Point2D<T>;

    fn add(self, rhs: Point2D<T>) -> Self::Output {
        Point2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Add<(T, T)> for Point2D<T>
where
    T: Sized + Add<Output = T> + Mul,
{
    type Output = Point2D<T>;

    fn add(self, rhs: (T, T)) -> Self::Output {
        Point2D {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

impl<T> Sub<Point2D<T>> for Point2D<T>
where
    T: Sized + Add + Mul + Sub<Output = T>,
{
    type Output = Point2D<T>;

    fn sub(self, rhs: Point2D<T>) -> Self::Output {
        Point2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Mul<T> for Point2D<T>
where
    T: Sized + Add + Mul<Output = T> + Copy,
{
    type Output = Point2D<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Point2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> From<(T, T)> for Point2D<T>
where
    T: Sized + Add + Mul,
{
    fn from(value: (T, T)) -> Self {
        Point2D {
            x: value.0,
            y: value.1,
        }
    }
}

#[cfg(test)]
mod test {}
