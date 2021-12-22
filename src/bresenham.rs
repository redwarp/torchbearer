//! Collection of bresenham implementation of lines, circles, … as Iterable.

use core::iter::Iterator;

use crate::Point;

/// Iterator-based Bresenham's line drawing algorithm.
///
/// Fork from https://github.com/mbr/bresenham-rs so that the iterator includes
/// `start` and `end`.
///
/// [Bresenham's line drawing algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm)
/// is a fast algorithm to draw a line between two points. This implements the fast
/// integer variant, using an iterator-based appraoch for flexibility. It
/// calculates coordinates without knowing anything about drawing methods or
/// surfaces.
///
/// # Example
///
/// ```rust
/// use torchbearer::bresenham::BresenhamLine;
///
/// for (x, y) in BresenhamLine::new((0, 1), (6, 4)) {
///     println!("{}, {}", x, y);
/// }
/// ```
///
/// Will print:
///
/// ```text
/// (0, 1)                      . . . . . . .
/// (1, 1)                      x # . . . . .
/// (2, 2)   corresponding to   . . # # . . .
/// (3, 2)                      . . . . # # .
/// (4, 3)                      . . . . . . x
/// (5, 3)                      . . . . . . .
/// (6, 4)
/// ```
pub struct BresenhamLine {
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
    x1: i32,
    y1: i32,
    diff: i32,
    octant: Octant,
}

struct Octant(u8);

impl Octant {
    /// adapted from http://codereview.stackexchange.com/a/95551
    #[inline]
    fn from_points(start: Point, end: Point) -> Octant {
        let mut dx = end.0 - start.0;
        let mut dy = end.1 - start.1;

        let mut octant = 0;

        if dy < 0 {
            dx = -dx;
            dy = -dy;
            octant += 4;
        }

        if dx < 0 {
            let tmp = dx;
            dx = dy;
            dy = -tmp;
            octant += 2
        }

        if dx < dy {
            octant += 1
        }

        Octant(octant)
    }

    #[inline]
    fn to_octant0(&self, p: Point) -> Point {
        match self.0 {
            0 => (p.0, p.1),
            1 => (p.1, p.0),
            2 => (p.1, -p.0),
            3 => (-p.0, p.1),
            4 => (-p.0, -p.1),
            5 => (-p.1, -p.0),
            6 => (-p.1, p.0),
            7 => (p.0, -p.1),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn from_octant0(&self, p: Point) -> Point {
        match self.0 {
            0 => (p.0, p.1),
            1 => (p.1, p.0),
            2 => (-p.1, p.0),
            3 => (-p.0, p.1),
            4 => (-p.0, -p.1),
            5 => (-p.1, -p.0),
            6 => (p.1, -p.0),
            7 => (p.0, -p.1),
            _ => unreachable!(),
        }
    }
}

impl BresenhamLine {
    /// Creates a new iterator. Yields intermediate points between `start`
    /// and `end`, inclusive.
    pub fn new(start: Point, end: Point) -> BresenhamLine {
        let octant = Octant::from_points(start, end);

        let start = octant.to_octant0(start);
        let end = octant.to_octant0(end);

        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        BresenhamLine {
            x: start.0,
            y: start.1,
            dx,
            dy,
            x1: end.0,
            y1: end.1,
            diff: dy - dx,
            octant,
        }
    }
}

impl ExactSizeIterator for BresenhamLine {}

impl Iterator for BresenhamLine {
    type Item = Point;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x == self.x1 {
            self.x += 1;
            let p = (self.x1, self.y1);
            return Some(self.octant.from_octant0(p));
        }

        if self.x > self.x1 {
            return None;
        }

        let p = (self.x, self.y);

        if self.diff >= 0 {
            self.y += 1;
            self.diff -= self.dx;
        }

        self.diff += self.dy;

        // loop inc
        self.x += 1;

        Some(self.octant.from_octant0(p))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.dx + 1) as usize;
        (len, Some(len))
    }
}

/// Iterator-based Bresenham's circle drawing algorithm.
///
/// [Bresenham's circle drawing algorithm](http://members.chello.at/~easyfilter/bresenham.html)
/// is a fast algorithm to draw the circumference of a circle.
///
/// # Example
///
/// ```
/// use torchbearer::bresenham::BresenhamCircle;
///
/// let center = (0, 0);
/// let radius = 2;
/// for (x,y) in BresenhamCircle::new(center, radius) {
///     println!("{}, {}", x, y);
/// }
/// ```
///
/// Will print
///
/// ```text
/// (2, 0)
/// (2, 1)
/// (1, 2)
/// (0, 2)                       . . . . . . .
/// (-1, 2)                      . . # # # . .
/// (-2, 1)                      . # . . . # .
/// (-2, 0)   corresponding to   . # . x . # .
/// (-2, -1)                     . # . . . # .
/// (-1, -2)                     . . # # # . .
/// (0, -2)                      . . . . . . .
/// (1, -2)
/// (2, -1)
/// ```
pub struct BresenhamCircle {
    center: Point,
    original_radius: i32,
    radius: i32,
    x: i32,
    y: i32,
    err: i32,
    current_quadrant: i32,
}

impl BresenhamCircle {
    /// Create new iterator. Yield all points on the circumference of the circle
    /// of center `center` and radius `radius`.
    pub fn new(center: Point, radius: i32) -> Self {
        BresenhamCircle {
            center,
            original_radius: radius,
            radius,
            x: -radius,
            y: 0,
            err: 2 - 2 * radius,
            current_quadrant: 0,
        }
    }
}

impl Iterator for BresenhamCircle {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= 0 {
            None
        } else {
            let point = match self.current_quadrant {
                0 => (self.center.0 - self.x, self.center.1 + self.y),
                1 => (self.center.0 - self.y, self.center.1 - self.x),
                2 => (self.center.0 + self.x, self.center.1 - self.y),
                3 => (self.center.0 + self.y, self.center.1 + self.x),
                _ => unreachable!(),
            };

            // We went through the points of 4 quadrants, moving on.
            self.radius = self.err;
            if self.radius <= self.y {
                self.y += 1;
                self.err += self.y * 2 + 1;
            }
            if self.radius > self.x || self.err > self.y {
                self.x += 1;
                self.err += self.x * 2 + 1;
            }

            if self.x >= 0 && self.current_quadrant < 3 {
                self.current_quadrant = (self.current_quadrant + 1) % 4;
                // Reset for next quadrant
                self.radius = self.original_radius;
                self.x = -self.original_radius;
                self.y = 0;
                self.err = 2 - 2 * self.original_radius;
            }

            Some(point)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BresenhamCircle;
    use super::BresenhamLine;
    use std::vec::Vec;

    #[test]
    fn test_wp_example() {
        let bi = BresenhamLine::new((0, 1), (6, 4));
        let len = bi.len();
        let res: Vec<_> = bi.collect();

        assert_eq!(
            res,
            [(0, 1), (1, 1), (2, 2), (3, 2), (4, 3), (5, 3), (6, 4)]
        );
        assert_eq!(len, 7);
    }

    #[test]
    fn test_inverse_wp() {
        let bi = BresenhamLine::new((6, 4), (0, 1));
        let len = bi.len();
        let res: Vec<_> = bi.collect();

        assert_eq!(
            res,
            [(6, 4), (5, 4), (4, 3), (3, 3), (2, 2), (1, 2), (0, 1)]
        );
        assert_eq!(len, 7);
    }

    #[test]
    fn test_straight_hline() {
        let bi = BresenhamLine::new((2, 3), (5, 3));
        let len = bi.len();
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(2, 3), (3, 3), (4, 3), (5, 3)]);
        assert_eq!(len, 4);
    }

    #[test]
    fn test_straight_vline() {
        let bi = BresenhamLine::new((2, 3), (2, 6));
        let len = bi.len();
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(2, 3), (2, 4), (2, 5), (2, 6)]);
        assert_eq!(len, 4);
    }

    #[test]
    fn circle_contiguous() {
        let circle = BresenhamCircle::new((0, 0), 2);

        let res: Vec<_> = circle.collect();

        assert_eq!(
            res,
            [
                (2, 0),
                (2, 1),
                (1, 2),
                (0, 2),
                (-1, 2),
                (-2, 1),
                (-2, 0),
                (-2, -1),
                (-1, -2),
                (0, -2),
                (1, -2),
                (2, -1)
            ]
        );
    }
}
