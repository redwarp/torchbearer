//! A set of tools to find your path in a grid based dungeon. Field of view, pathfinding...
//!
//! Inspired by [tcod-rs](https://crates.io/crates/tcod) and [bracket-pathfinding](https://crates.io/crates/bracket-pathfinding),
//! it aims to be simpler to use than tcod, without requiring a sdl2 dependency, and faster than bracket-pathfinding.
//!
//! The field of vision algorithm perform quite well, but the pathfinding one will need some serious optimizations to be
//! competitive, as tcod is fast, really fast.
//!
//! # Get started
//!
//! Implement the `Map` trait, and call field of vision or pathfinding algorithm.
//!
//! ```
//! use torchbearer::{Map, Point};
//! use torchbearer::fov::field_of_view;
//! use torchbearer::path::astar_path_fourwaygrid;
//!
//! struct SampleMap {
//!     width: i32,
//!     height: i32,
//!     transparent: Vec<bool>,
//!     walkable: Vec<bool>,
//! }
//!
//! impl SampleMap {
//!     fn new(width: i32, height: i32) -> Self {
//!          // (…)
//! #        SampleMap {
//! #            width,
//! #            height,
//! #            transparent: vec![true; (width * height) as usize],
//! #            walkable: vec![true; (width * height) as usize],
//! #        }
//!    }
//! }
//!
//! impl Map for SampleMap {
//!     fn dimensions(&self) -> (i32, i32) {
//!         (self.width, self.height)
//!     }
//!
//!     fn is_transparent(&self, x: i32, y: i32) -> bool {
//!         self.transparent[(x + y * self.width) as usize]
//!     }
//!
//!     fn is_walkable(&self, x: i32, y: i32) -> bool {
//!         self.walkable[(x + y * self.width) as usize]
//!     }
//! }
//!
//! let sample_map = SampleMap::new(16, 10);
//!
//! // (…) You probably want at this point to add some walls to your map.
//!
//! let from = (1,1);
//! let to = (3,8);
//! let radius = 5;
//!
//! for visible_position in field_of_view(&sample_map, from, radius) {
//!     // (…)
//! }
//!
//! if let Some(path) = astar_path_fourwaygrid(&sample_map, from, to) {
//!     // (…)
//! }
//! ```

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
    fn is_transparent(&self, x: i32, y: i32) -> bool;
    /// Wether it is possible or not to walk through the tile at position `(x, y)`.
    /// Used by pathfinding algorithm.
    fn is_walkable(&self, x: i32, y: i32) -> bool;
}
