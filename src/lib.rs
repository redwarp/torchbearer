#![doc = include_str!("../README.md")]

pub mod bresenham;
pub mod fov;
pub mod path;

/// A convenience type alias for a position tuple.
pub type Point = (i32, i32);

/// Implement the Map trait to use the field of view and pathfinding functions.
pub trait Map {
    /// Dimension of your map, in grid size.
    fn dimensions(&self) -> (i32, i32);
    /// Wether it is possible or not to see through the tile at position `(x, y)`.
    /// Used by field of view algorithm.
    fn is_transparent(&self, position: Point) -> bool;
    /// Wether it is possible or not to walk through the tile at position `(x, y)`.
    /// Used by pathfinding algorithm.
    fn is_walkable(&self, position: Point) -> bool;
}
