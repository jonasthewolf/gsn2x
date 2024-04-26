use std::ops::{Add, Mul};

use super::point2d::Point2D;

pub struct CubicBezierCurve<T>
where
    T: Sized + Add + Mul,
{
    p0: Point2D<T>,
    p1: Point2D<T>,
    p2: Point2D<T>,
    p3: Point2D<T>,
}

impl CubicBezierCurve<i32> {
    pub fn new(p0: Point2D<i32>, p1: Point2D<i32>, p2: Point2D<i32>, p3: Point2D<i32>) -> Self {
        CubicBezierCurve { p0, p1, p2, p3 }
    }

    ///
    /// B'' == 0
    /// (1-t)*(P2-2*P1+P0) = t*(P3-2*P2+P1)
    /// (P2-2*P1+P0) - t*(P2-2*P1+P0) = t*(P3-2*P2+P1)
    /// (P2-2*P1+P0) = t*(P2-2*P1+P0 + P3-2*P2+P1)
    /// (P2-2*P1+P0) = t*(-P2-P1+P0+P3)
    /// (P2-2*P1+P0) * (P3-P2-P1+P0)^-1 = t
    /// (P0-2*P1+P2) * (P0-P1-P2+P3)^-1 = t
    ///
    ///
    // pub fn get_turning_point(&self) -> f64 {
    //     let num = self.p0.x - 2 * self.p1.x + self.p2.x;
    //     let denom = self.p0.x - self.p1.x - self.p2.x + self.p3.x;

    //     num as f64 / denom as f64
    // }

    ///
    /// Get the x,y coordinates for parameter t of cubic Bezier curve.
    ///
    pub fn get_coordinates_for_t(&self, t: f64) -> Point2D<i32> {
        (1.0 - t).powf(3.0) * self.p0
            + 3.0 * (1.0 - t).powf(2.0) * t * self.p1
            + 3.0 * (1.0 - t) * t * t * self.p2
            + (t * t * t) * self.p3
    }

    ///
    /// Get the first derivative at t for cubic Bezier curve in x,y coordinate system.
    ///
    pub fn get_first_derivative_for_t(&self, t: f64) -> Point2D<i32> {
        3.0 * (1.0 - t).powf(2.0) * (self.p1 - self.p0)
            + 6.0 * (1.0 - t) * t * (self.p2 - self.p1)
            + 3.0 * (t * t) * (self.p3 - self.p2)
    }
}
