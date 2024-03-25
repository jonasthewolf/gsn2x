use core::fmt::Debug;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, MulAssign, Sub},
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

impl Point2D<i32> {
    ///
    /// Get the distance between two points
    ///
    pub fn distance(&self, p2: &Point2D<i32>) -> i32 {
        f64::sqrt(
            (self.x - p2.x) as f64 * (self.x - p2.x) as f64
                + (self.y - p2.y) as f64 * (self.y - p2.y) as f64,
        ) as i32
    }

    ///
    /// Get angle between self and other point
    ///
    // pub fn angle(&self, p2: &Point2D<i32>) -> f64 {
    //     f64::acos((self.x * p2.x + self.y * p2.y) as f64 / (self.norm() * p2.norm()))
    // }

    ///
    /// Get the norm of the point
    ///
    pub fn norm(&self) -> f64 {
        f64::sqrt((self.x * self.x + self.y * self.y).into())
    }

    pub fn normalize(self) -> Point2D<f64> {
        let norm = self.norm();
        Point2D {
            x: self.x as f64 / norm,
            y: self.y as f64 / norm,
        }
    }
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

// impl<T> Mul<T> for Point2D<T>
// where
//     T: Sized + Add + Mul<Output = T> + Copy,
// {
//     type Output = Point2D<T>;

//     fn mul(self, rhs: T) -> Self::Output {
//         Point2D {
//             x: self.x * rhs,
//             y: self.y * rhs,
//         }
//     }
// }

impl Mul<i32> for Point2D<i32> {
    type Output = Point2D<i32>;

    fn mul(self, rhs: i32) -> Self::Output {
        Point2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Point2D<i32>> for i32 {
    type Output = Point2D<i32>;

    fn mul(self, rhs: Point2D<i32>) -> Self::Output {
        Point2D {
            x: rhs.x * self,
            y: rhs.y * self,
        }
    }
}

impl Mul<f64> for Point2D<i32> {
    type Output = Point2D<i32>;

    fn mul(self, rhs: f64) -> Self::Output {
        Point2D {
            x: ((self.x as f64) * rhs).round() as i32,
            y: ((self.y as f64) * rhs).round() as i32,
        }
    }
}

impl Mul<Point2D<i32>> for f64 {
    type Output = Point2D<i32>;

    fn mul(self, rhs: Point2D<i32>) -> Self::Output {
        Point2D {
            x: ((rhs.x as f64) * self).round() as i32,
            y: ((rhs.y as f64) * self).round() as i32,
        }
    }
}


impl<T> Mul<Point2D<T>> for Point2D<T>
where
    T: Sized + Add<Output = T> + Mul<Output = T>,
{
    type Output = T;

    fn mul(self, rhs: Point2D<T>) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y
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
mod test {
    use super::Point2D;

    #[test]
    fn clone_copy() {
        let p = Point2D::<i32> { x: 14, y: 23 };
        let p_copy = p;
        assert_eq!(p_copy.x, p.x);
        assert_eq!(p_copy.y, p.y);
    }

    #[test]
    fn scalar() {
        let p = Point2D::<i32> { x: 14, y: 23 };
        let p_new = p * 2;
        let p_new2 = 2 * p;
        let p_new3 = 4.0 * p;
        let p_new4 = p * 3.0;
        assert_eq!(p_new.x, p.x * 2);
        assert_eq!(p_new.y, p.y * 2);
        assert_eq!(p_new2.x, p.x * 2);
        assert_eq!(p_new2.y, p.y * 2);
        assert_eq!(p_new3.x, p.x * 4);
        assert_eq!(p_new3.y, p.y * 4);
        assert_eq!(p_new4.x, p.x * 3);
        assert_eq!(p_new4.y, p.y * 3);
    }

    #[test]
    fn cross_products() {
        let p1 = Point2D::<i32> { x: 0, y: 1 };
        let p2 = Point2D::<i32> { x: 1, y: 0 };

        assert_eq!(p1 * p2, 0);
        assert_eq!(p1 * p1, 1);
    }

    #[test]
    fn basics() {
        let mut p = Point2D::<i32> { x: 2, y: 3 };
        p += p + Point2D::from((1, 1)) - (2, 2).into();
        assert_eq!(p.x, 3);
        assert_eq!(p.y, 5);
    }

    #[test]
    fn debug_display() {
        let p = Point2D::<i32> { x: 2, y: 3 };
        assert_eq!(format!("{}", p), "2,3");
        assert_eq!(format!("{:?}", p), "2,3");
    }
}
