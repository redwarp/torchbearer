#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]

pub mod bresenham;
pub mod fov;
pub mod path;

/// A convenience type alias for a position tuple.
pub type Point = (i32, i32);
