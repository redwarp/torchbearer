use bracket_pathfinding::prelude::{Algorithm2D, SmallVec};
use criterion::{criterion_group, criterion_main, Criterion};
use tcod::Map as TcodMap;
use torchbearer::{
    bresenham::BresenhamLine,
    path::{astar_path, astar_path_fourwaygrid, FourWayGridGraph, PathMap},
    Point,
};

const WIDTH: i32 = 20;
const HEIGHT: i32 = 20;

struct TestMap {
    width: i32,
    height: i32,
    tiles: Vec<bool>,
}

impl TestMap {
    fn new(width: i32, height: i32) -> Self {
        TestMap {
            width,
            height,
            tiles: vec![true; (width * height) as usize],
        }
    }

    fn with_walls(mut self) -> Self {
        self.build_wall((0, 3), (3, 3));
        self.build_wall((3, 3), (3, 10));
        self.build_wall((5, 3), (5, 19));
        self.build_wall((7, 0), (7, 16));
        self.build_wall((9, 1), (9, 19));
        self
    }

    fn build_wall(&mut self, from: Point, to: Point) {
        let bresenham = BresenhamLine::new(from, to);
        for (x, y) in bresenham {
            self.tiles[(x + y * self.width) as usize] = false;
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = x + y * self.width;
        self.tiles[idx as usize]
    }
}

impl PathMap for TestMap {
    fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn is_walkable(&self, (x, y): Point) -> bool {
        self.tiles[(x + y * self.width) as usize]
    }
}

/// Implementing the BaseMap like
/// https://bfnightly.bracketproductions.com/rustbook/chapter_7.html?highlight=pathfin#chasing-the-player
impl bracket_pathfinding::prelude::BaseMap for TestMap {
    fn is_opaque(&self, index: usize) -> bool {
        self.tiles[index]
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = bracket_pathfinding::prelude::Point::new(idx1 % w, idx1 / w);
        let p2 = bracket_pathfinding::prelude::Point::new(idx2 % w, idx2 / w);
        bracket_pathfinding::prelude::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl bracket_pathfinding::prelude::Algorithm2D for TestMap {
    fn dimensions(&self) -> bracket_pathfinding::prelude::Point {
        (self.width, self.height).into()
    }

    fn point2d_to_index(&self, pt: bracket_pathfinding::prelude::Point) -> usize {
        (pt.x + pt.y * self.width) as usize
    }
}

pub fn torchbearer_astar_fourwaygrid(c: &mut Criterion) {
    let map = TestMap::new(WIDTH, HEIGHT).with_walls();
    let from = (1, 4);
    let to = (15, 8);

    c.bench_function("torchbearer_astar_fourwaygrid", |bencher| {
        bencher.iter(|| astar_path_fourwaygrid(&map, from, to));
    });
}

pub fn torchbearer_astar_graph(c: &mut Criterion) {
    let map = TestMap::new(WIDTH, HEIGHT).with_walls();
    let graph = FourWayGridGraph::new(&map);
    let from = (1 + 4 * WIDTH) as usize;
    let to = (15 + 8 * WIDTH) as usize;

    c.bench_function("torchbearer_astar_graph", |bencher| {
        bencher.iter(|| astar_path(&graph, from, to));
    });
}

pub fn bracket_astar(c: &mut Criterion) {
    let map = TestMap::new(WIDTH, HEIGHT).with_walls();
    let start = map.point2d_to_index((1, 4).into());
    let end = map.point2d_to_index((15, 8).into());

    c.bench_function("bracket_astar", |bencher| {
        bencher.iter(|| bracket_pathfinding::prelude::a_star_search(start, end, &map));
    });
}

pub fn tcod_astar(c: &mut Criterion) {
    fn build_wall(map: &mut TcodMap, from: Point, to: Point) {
        let bresenham = BresenhamLine::new(from, to);
        for (x, y) in bresenham {
            map.set(x, y, false, false);
        }
    }

    let mut map = TcodMap::new(WIDTH as i32, HEIGHT as i32);
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            map.set(x, y, true, true);
        }
    }
    build_wall(&mut map, (0, 3), (3, 3));
    build_wall(&mut map, (3, 3), (3, 10));
    build_wall(&mut map, (5, 3), (5, 19));
    build_wall(&mut map, (7, 0), (7, 16));
    build_wall(&mut map, (9, 1), (9, 19));

    let mut astar = tcod::pathfinding::AStar::new_from_map(map, 0.0);
    let from = (1, 4);
    let to = (15, 8);

    c.bench_function("tcod_astar", |bencher| {
        bencher.iter(|| astar.find(from, to));
    });
}

criterion_group!(
    benches,
    torchbearer_astar_fourwaygrid,
    torchbearer_astar_graph,
    bracket_astar,
    tcod_astar
);
criterion_main!(benches);
