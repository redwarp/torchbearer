[![Docs](https://docs.rs/torchbearer/badge.svg)](https://docs.rs/torchbearer)
[![Crates.io](https://img.shields.io/crates/d/torchbearer.svg)](https://crates.io/crates/torchbearer)
[![Crates.io](https://img.shields.io/crates/v/torchbearer.svg)](https://crates.io/crates/torchbearer)

A set of tools to find your path in a grid based dungeon. Field of view, pathfinding...

Inspired by [tcod-rs](https://crates.io/crates/tcod) and [bracket-pathfinding](https://crates.io/crates/bracket-pathfinding), 
it aims to be simpler to use than tcod, without requiring a sdl2 dependency, and faster than bracket-pathfinding.

# Get started

Implement the `VisionMap` trait to use the field of vision algorithms, or the `PathMap` trait to use the pathfinding algorithms.

```rust
use torchbearer::Point;
use torchbearer::fov::{field_of_view, VisionMap};
use torchbearer::path::{astar_path_fourwaygrid, PathMap};

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

impl VisionMap for SampleMap {
    fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn is_transparent(&self, (x, y): Point) -> bool {
        self.transparent[(x + y * self.width) as usize]
    }
}

impl PathMap for SampleMap {
    fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
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

Even though torchbearer provides by default a pathfinding simple function for fourway grid maps (meaning, you can go north, south, east and west), you can also implement your own graph, to support other types of movements (8 ways, north, ne, east, se, south, sw, west, nw), or teleportation, or...
You can also simply decide you want to tweak your heuristics, so that moving along roads is easier than climbing a mountain for instance.

```rust
use torchbearer::path::{astar_path, Graph, NodeId};

struct MyMap {}

impl Graph for MyMap {
    // (..)
#    fn node_count(&self) -> usize {
#        3
#    }
#
#    fn cost_between(&self, a: NodeId, b: NodeId) -> f32 {
#        1.0
#    }
#
#   fn heuristic(&self, a: NodeId, b: NodeId) -> f32 {
#        1.0
#    }
#
#    fn neighboors(&self, a: NodeId, into: &mut Vec<NodeId>) {}
}

let my_map = MyMap {};

let from = (1) as NodeId;
let to = (2) as NodeId;

if let Some(path) = astar_path(&my_map, to, from) {
    // (…)
}
```
