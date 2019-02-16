use cgmath::*;
use num_traits::cast::NumCast;
use serde::*;

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Rect<T> {
    pub start: Point2<T>,
    pub end: Point2<T>,
}

impl<T> Rect<T> {
    pub fn new(start: Point2<T>, end: Point2<T>) -> Self {
        Self { start, end }
    }
}

impl<T: BaseNum + Ord> Rect<T> {
    // TODO: support this for floats
    /// Creates a `Rect` that is the bounding box of a set of points.
    pub fn from_points(points: &[Point2<T>]) -> Rect<T> {
        // TODO: use minmax_by_key
        Rect {
            start: Point2::from_vec(vec2(
                points.iter().min_by_key(|point| point.x).unwrap().x,
                points.iter().min_by_key(|point| point.y).unwrap().y,
            )),
            end: Point2::from_vec(vec2(
                points.iter().max_by_key(|point| point.x).unwrap().x,
                points.iter().max_by_key(|point| point.y).unwrap().y,
            )),
        }
    }
}

impl<T: BaseNum> Rect<T> {
    /// Returns the size of the `Rect`
    pub fn size(&self) -> Vector2<T> {
        self.end - self.start
    }

    /// Returns whether the `Rect` contains the given point.
    pub fn contains_point(self, point: Point2<T>) -> bool {
        self.start.x <= point.x
            && self.end.x >= point.x
            && self.start.y <= point.y
            && self.end.y >= point.y
    }
}

impl<T: NumCast + Copy> Rect<T> {
    pub fn cast<Res: NumCast>(&self) -> Option<Rect<Res>> {
        Some(Rect::new(self.start.cast()?, self.end.cast()?))
    }
}
