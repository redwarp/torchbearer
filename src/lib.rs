#![doc = include_str!("../README.md")]

pub mod bresenham;
pub mod fov;
pub mod path;

/// A convenience type alias for a position tuple.
pub type Point = (i32, i32);
