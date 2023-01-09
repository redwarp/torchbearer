//! Collection of utility function to calculate field of vision.

use crate::{
    bresenham::{BresenhamLine, ThickBresenhamCircle},
    Point,
};

/// Implement the VisionMap trait to use the field of view function.
pub trait VisionMap {
    /// Dimension of your map, in grid size.
    fn dimensions(&self) -> (i32, i32);
    /// Wether it is possible or not to see through the tile at position `(x, y)`.
    /// Used by field of view algorithm.
    fn is_transparent(&self, position: Point) -> bool;
}

/// An implementation of the field of view algorithm using basic raycasting.
/// Returns a vector containing all points visible from the starting position, including the starting position.
///
/// Implement the algorithm found on the [visibility determination](https://sites.google.com/site/jicenospam/visibilitydetermination).
/// For a comparison of the different raycasting types, advantages and disavantages, see
/// [roguebasin's comparison](http://www.roguebasin.com/index.php?title=Comparative_study_of_field_of_view_algorithms_for_2D_grid_based_worlds)
///
/// # Arguments
///
/// * `map` - A struct implementing the `VisionMap` trait.
/// * `from` - The origin/center of the field of vision.
/// * `radius` - How far the vision should go. Should be higher or equal to 0 (If 0, you only see yourself).
///
/// # Examples
/// ```
/// use torchbearer::{
///     fov::{field_of_view, VisionMap},
///     Point,
/// };
///
/// struct SampleMap {
///     width: i32,
///     height: i32,
///     transparent: Vec<bool>,
/// }
///
/// impl SampleMap {
///     fn new(width: i32, height: i32) -> Self {
///         // (…)
/// #        SampleMap {
/// #            width,
/// #            height,
/// #            transparent: vec![true; (width * height) as usize],
/// #        }
///     }
/// }
///
/// impl VisionMap for SampleMap {
///     fn dimensions(&self) -> (i32, i32) {
///         (self.width, self.height)
///     }
///
///     fn is_transparent(&self, (x, y): Point) -> bool {
///         self.transparent[(x + y * self.width) as usize]
///     }
/// }
///
/// let sample_map = SampleMap::new(16, 10);
///
/// // (…) You probably want at this point to add some walls to your map.
/// let from = (1, 1);
/// let radius = 5;
/// let visible_positions = field_of_view(&sample_map, from, radius);
///
/// for visible_position in visible_positions {
///     // (…)
/// }
/// ```
pub fn field_of_view<T: VisionMap>(map: &T, from: Point, radius: i32) -> Vec<(i32, i32)> {
    let (x, y) = from;
    assert_in_bounds(map, x, y);
    if radius < 0 {
        panic!("A radius >= 0 is required, you used {}", radius);
    }

    if radius < 1 {
        return vec![(x, y)];
    }

    let (width, height) = map.dimensions();

    let minx = (x - radius).max(0);
    let miny = (y - radius).max(0);
    let maxx = (x + radius).min(width - 1);
    let maxy = (y + radius).min(height - 1);

    if maxx - minx == 0 || maxy - miny == 0 {
        // Well, no area to check.
        return vec![];
    }

    let (sub_width, sub_height) = (maxx - minx + 1, maxy - miny + 1);
    let (offset_x, offset_y) = (minx, miny);

    let mut visibles = vec![false; (sub_width * sub_height) as usize];
    // Set origin as visible.
    visibles[(x - offset_x + (y - offset_y) * sub_width) as usize] = true;

    for point in ThickBresenhamCircle::new(from, radius) {
        cast_ray(
            map,
            &mut visibles,
            sub_width,
            from,
            point,
            (offset_x, offset_y),
        );
    }

    visibles
        .into_iter()
        .enumerate()
        .filter_map(|(index, visible)| {
            if visible {
                Some((
                    index as i32 % sub_width + offset_x,
                    index as i32 / sub_width + offset_y,
                ))
            } else {
                None
            }
        })
        .collect()
}

fn is_out_of_bounds<M: VisionMap>(map: &M, x: i32, y: i32) -> bool {
    let (width, height) = map.dimensions();
    x < 0 || y < 0 || x >= width || y >= height
}

fn assert_in_bounds<M: VisionMap>(map: &M, x: i32, y: i32) {
    let (width, height) = map.dimensions();
    if is_out_of_bounds(map, x, y) {
        panic!(
            "(x, y) should be between (0,0) and ({}, {}), got ({}, {})",
            width, height, x, y
        );
    }
}

fn cast_ray<T: VisionMap>(
    map: &T,
    visibles: &mut [bool],
    width: i32,
    origin: Point,
    destination: Point,
    offset: (i32, i32),
) {
    // We skip the first item as it is the origin position.
    let bresenham = BresenhamLine::new(origin, destination).skip(1);
    for (x, y) in bresenham {
        let (off_x, off_y) = (x - offset.0, y - offset.1);
        if off_x < 0 || off_y < 0 {
            // No need to continue the ray, we are out of bounds.
            return;
        }
        visibles[(off_x + off_y * width) as usize] = true;

        if !map.is_transparent((x, y)) {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{prelude::StdRng, Rng, SeedableRng};
    use std::fmt::Debug;

    use crate::Point;

    use super::{field_of_view, VisionMap};
    const WIDTH: i32 = 45;
    const HEIGHT: i32 = 45;
    const POSITION_X: i32 = 22;
    const POSITION_Y: i32 = 22;
    const RADIUS: i32 = 24;
    const RANDOM_WALLS: i32 = 10;

    pub struct SampleMap {
        /// Vector to store the transparent tiles.
        transparent: Vec<bool>,
        /// Vector to store the computed field of vision.
        vision: Vec<bool>,
        /// The width of the map
        width: i32,
        /// The height of the map
        height: i32,
        /// The last position where the field of view was calculated. If never calculated, initialized to (-1, -1).
        last_origin: (i32, i32),
    }

    impl VisionMap for SampleMap {
        fn dimensions(&self) -> (i32, i32) {
            (self.width, self.height)
        }

        fn is_transparent(&self, (x, y): Point) -> bool {
            let index = (x + y * self.width) as usize;
            self.transparent[index]
        }
    }

    impl SampleMap {
        pub fn new(width: i32, height: i32) -> Self {
            if width <= 0 && height <= 0 {
                panic!("Width and height should be > 0, got ({},{})", width, height);
            }
            SampleMap {
                transparent: vec![true; (width * height) as usize],
                vision: vec![false; (width * height) as usize],
                width,
                height,
                last_origin: (-1, -1),
            }
        }
        /// Flag a tile as transparent or visible.
        pub fn set_transparent(&mut self, x: i32, y: i32, is_transparent: bool) {
            self.transparent[(x + y * self.width) as usize] = is_transparent;
        }

        pub fn calculate_fov(&mut self, x: i32, y: i32, radius: i32) {
            for see in self.vision.iter_mut() {
                *see = false;
            }

            let visibles = field_of_view(self, (x, y), radius);

            for (x, y) in visibles {
                self.vision[(x + y * self.width) as usize] = true
            }
            self.last_origin = (x, y);
        }

        pub fn calculate_fov2(&mut self, x: i32, y: i32, radius: i32) {
            for see in self.vision.iter_mut() {
                *see = false;
            }

            let visibles = field_of_view(self, (x, y), radius);

            for (x, y) in visibles {
                self.vision[(x + y * self.width) as usize] = true
            }
            self.last_origin = (x, y);
        }
    }

    impl Debug for SampleMap {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let (width, _height) = self.dimensions();

            let last_origin_index = if self.last_origin.0 >= 0 && self.last_origin.1 >= 0 {
                Some((self.last_origin.0 + self.last_origin.1 * width) as usize)
            } else {
                None
            };

            let mut display_string = String::from("+");
            display_string.push_str("-".repeat(self.width as usize).as_str());
            display_string.push_str("+\n");
            for index in 0..self.vision.len() {
                if index % self.width as usize == 0 {
                    display_string.push('|');
                }

                let is_last_origin = if let Some(last_origin_index) = last_origin_index {
                    last_origin_index == index
                } else {
                    false
                };
                let tile = match (is_last_origin, self.transparent[index], self.vision[index]) {
                    (true, _, _) => '*',
                    (_, true, true) => ' ',
                    (_, false, true) => '□',
                    _ => '?',
                };
                display_string.push(tile);
                if index > 0 && (index + 1) % self.width as usize == 0 {
                    display_string.push_str("|\n");
                }
            }
            display_string.truncate(display_string.len() - 1);
            display_string.push('\n');
            display_string.push('+');
            display_string.push_str("-".repeat(self.width as usize).as_str());
            display_string.push('+');

            write!(f, "{}", display_string)
        }
    }

    #[test]
    fn fov_with_sample_map() {
        let mut fov = SampleMap::new(10, 10);
        for x in 1..10 {
            fov.set_transparent(x, 3, false);
        }
        for y in 0..10 {
            fov.set_transparent(9, y, false);
        }
        fov.calculate_fov(3, 2, 10);

        println!("{:?}", fov);
    }

    #[test]
    fn fov_with_sample_map2() {
        let mut fov = SampleMap::new(10, 10);
        for x in 1..10 {
            fov.set_transparent(x, 3, false);
        }
        for y in 0..10 {
            fov.set_transparent(9, y, false);
        }
        fov.calculate_fov2(3, 2, 10);

        println!("{:?}", fov);
    }

    #[test]
    fn fov_to_vector() {
        let mut fov = SampleMap::new(WIDTH, HEIGHT);

        fov.calculate_fov(POSITION_X, POSITION_Y, RADIUS);
    }

    #[test]
    fn fov_with_wall_to_vector() {
        let mut fov = SampleMap::new(WIDTH, HEIGHT);
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..RANDOM_WALLS {
            let (x, y) = (rng.gen_range(0..WIDTH), rng.gen_range(0..HEIGHT));
            fov.set_transparent(x, y, false);
        }
        fov.set_transparent(POSITION_X, POSITION_Y, true);

        fov.calculate_fov(POSITION_X, POSITION_Y, RADIUS);

        println!("{:?}", fov);
    }
}
