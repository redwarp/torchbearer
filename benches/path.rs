use bracket_pathfinding::prelude::{Algorithm2D, SmallVec};
use criterion::{
    BenchmarkGroup, Criterion, criterion_group, criterion_main, measurement::WallTime,
};
#[cfg(feature = "tcod")]
use tcod::Map as TcodMap;
use torchbearer::{
    Point,
    bresenham::BresenhamLine,
    path::{FourWayGridGraph, PathMap, astar_path, astar_path_fourwaygrid},
};

const WIDTH: i32 = 100;
const HEIGHT: i32 = 100;

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
        self.build_wall((0, 15), (15, 15));
        self.build_wall((15, 15), (15, 50));
        self.build_wall((25, 15), (25, 95));
        self.build_wall((35, 0), (35, 80));
        self.build_wall((45, 5), (45, 95));
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

#[cfg(feature = "tcod")]
impl Into<TcodMap> for TestMap {
    fn into(self) -> TcodMap {
        let mut map = TcodMap::new(self.width, self.height);
        for x in 0..self.width {
            for y in 0..self.height {
                let transparent = self.is_walkable((x, y));
                map.set(x, y, transparent, transparent);
            }
        }

        map
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

pub fn torchbearer_astar_fourwaygrid(group: &mut BenchmarkGroup<WallTime>) {
    let map = TestMap::new(WIDTH, HEIGHT).with_walls();
    let from = (5, 20);
    let to = (75, 40);

    group.bench_function("torchbearer_fourwaygrid", |bencher| {
        bencher.iter(|| astar_path_fourwaygrid(&map, from, to));
    });
}

pub fn torchbearer_astar_graph(group: &mut BenchmarkGroup<WallTime>) {
    let map = TestMap::new(WIDTH, HEIGHT).with_walls();
    let graph = FourWayGridGraph::new(&map);
    let from = (5 + 20 * WIDTH) as usize;
    let to = (75 + 40 * WIDTH) as usize;

    group.bench_function("torchbearer_graph", |bencher| {
        bencher.iter(|| astar_path(&graph, from, to));
    });
}

pub fn bracket_astar(group: &mut BenchmarkGroup<WallTime>) {
    let map = TestMap::new(WIDTH, HEIGHT).with_walls();
    let start = map.point2d_to_index((5, 20).into());
    let end = map.point2d_to_index((75, 40).into());

    group.bench_function("bracket", |bencher| {
        bencher.iter(|| bracket_pathfinding::prelude::a_star_search(start, end, &map));
    });
}

#[cfg(feature = "tcod")]
pub fn tcod_astar(group: &mut BenchmarkGroup<WallTime>) {
    let map: TcodMap = TestMap::new(WIDTH, HEIGHT).with_walls().into();

    let mut astar = tcod::pathfinding::AStar::new_from_map(map, 0.0);
    let from = (5, 20);
    let to = (75, 40);

    group.bench_function("tcod", |bencher| {
        bencher.iter(|| astar.find(from, to));
    });
}

pub fn astar(c: &mut Criterion) {
    let mut group = c.benchmark_group("astar");
    torchbearer_astar_fourwaygrid(&mut group);
    torchbearer_astar_graph(&mut group);
    bracket_astar(&mut group);
    #[cfg(feature = "tcod")]
    tcod_astar(&mut group);
}

criterion_group!(benches, astar,);
criterion_main!(benches);
