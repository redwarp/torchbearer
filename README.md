[![Docs](https://docs.rs/torchbearer/badge.svg)](https://docs.rs/torchbearer)
[![Crates.io](https://img.shields.io/crates/d/torchbearer.svg)](https://crates.io/crates/torchbearer)
[![Crates.io](https://img.shields.io/crates/v/torchbearer.svg)](https://crates.io/crates/torchbearer)

A set of tools to find your path in a grid based dungeon. Field of view, pathfinding...

Inspired by [tcod-rs](https://crates.io/crates/tcod) and [bracket-pathfinding](https://crates.io/crates/bracket-pathfinding), 
it aims to be simpler to use than tcod, without requiring a sdl2 dependency, and faster than bracket-pathfinding.

# Get started

Implement the `Map` trait, and call field of vision or pathfinding algorithm.

```rust
use torchbearer::{Map, Point};
use torchbearer::fov::field_of_view;
use torchbearer::path::astar_path_fourwaygrid;

struct SampleMap {
    width: i32,
    height: i32,
    transparent: Vec<bool>,
    walkable: Vec<bool>,
}

impl SampleMap {
    fn new(width: i32, height: i32) -> Self {
         // (…)
#        SampleMap {
#            width,
#            height,
#            transparent: vec![true; (width * height) as usize],
#            walkable: vec![true; (width * height) as usize],
#        }
   }
}

impl Map for SampleMap {
    fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn is_transparent(&self, (x, y): Point) -> bool {
        self.transparent[(x + y * self.width) as usize]
    }

    fn is_walkable(&self, (x, y): Point) -> bool {
        self.walkable[(x + y * self.width) as usize]
    }
}

let sample_map = SampleMap::new(16, 10);

// (…) You probably want at this point to add some walls to your map.

let from = (1,1);
let to = (3,8);
let radius = 5;

for visible_position in field_of_view(&sample_map, from, radius) {
    // (…)
}

if let Some(path) = astar_path_fourwaygrid(&sample_map, from, to) {
    // (…)
}
```
