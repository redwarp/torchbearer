use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::StdRng, Rng, SeedableRng};
use tcod::Map as TcodMap;
use torchbearer::Map;

const WIDTH: i32 = 45;
const HEIGHT: i32 = 45;
const POSITION_X: i32 = 22;
const POSITION_Y: i32 = 22;
const RADIUS: i32 = 12;
const RANDOM_WALLS: i32 = 10;

pub struct SampleMap {
    /// Vector to store the transparent tiles.
    transparent: Vec<bool>,
    /// The width of the map
    width: i32,
    /// The height of the map
    height: i32,
}

impl Map for SampleMap {
    fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn is_transparent(&self, x: i32, y: i32) -> bool {
        let index = (x + y * self.width) as usize;
        self.transparent[index]
    }

    fn is_walkable(&self, _x: i32, _y: i32) -> bool {
        false
    }
}

impl bracket_pathfinding::prelude::BaseMap for SampleMap {
    fn is_opaque(&self, index: usize) -> bool {
        !self.transparent[index]
    }
}

impl bracket_pathfinding::prelude::Algorithm2D for SampleMap {
    fn dimensions(&self) -> bracket_pathfinding::prelude::Point {
        (self.width as i32, self.height as i32).into()
    }
}

impl SampleMap {
    pub fn new(width: i32, height: i32) -> Self {
        if width <= 0 && height <= 0 {
            panic!("Width and height should be > 0, got ({},{})", width, height);
        }
        SampleMap {
            transparent: vec![true; (width * height) as usize],
            width,
            height,
        }
    }

    pub fn randomize_walls(mut self) -> Self {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..RANDOM_WALLS {
            let (x, y) = (rng.gen_range(0..WIDTH), rng.gen_range(0..HEIGHT));
            self.set_transparent(x, y, false);
        }
        self.set_transparent(POSITION_X, POSITION_Y, true);
        self
    }

    /// Flag a tile as transparent or visible.
    pub fn set_transparent(&mut self, x: i32, y: i32, is_transparent: bool) {
        self.transparent[(x + y * self.width) as usize] = is_transparent;
    }
}

pub fn torchbearer_fov_no_walls(c: &mut Criterion) {
    let map = SampleMap::new(WIDTH, HEIGHT);

    c.bench_function("torchbearer_fov_no_walls", |bencher| {
        bencher.iter(|| torchbearer::fov::field_of_view(&map, (POSITION_X, POSITION_Y), RADIUS));
    });
}

pub fn torchbearer_fov_random_walls(c: &mut Criterion) {
    let map = SampleMap::new(WIDTH, HEIGHT).randomize_walls();

    c.bench_function("torchbearer_fov_random_walls", |bencher| {
        bencher.iter(|| torchbearer::fov::field_of_view(&map, (POSITION_X, POSITION_Y), RADIUS));
    });
}

pub fn tcod_fov_no_walls(c: &mut Criterion) {
    let mut map = TcodMap::new(WIDTH as i32, HEIGHT as i32);
    for x in 0..WIDTH as i32 {
        for y in 0..HEIGHT as i32 {
            map.set(x, y, true, true);
        }
    }

    let x = POSITION_X as i32;
    let y = POSITION_Y as i32;
    let radius = RADIUS as i32;
    c.bench_function("tcod_fov_no_walls", |bencher| {
        bencher.iter(|| map.compute_fov(x, y, radius, true, tcod::map::FovAlgorithm::Basic));
    });
}

pub fn tcod_fov_random_walls(c: &mut Criterion) {
    let mut map = TcodMap::new(WIDTH as i32, HEIGHT as i32);
    for x in 0..WIDTH as i32 {
        for y in 0..HEIGHT as i32 {
            map.set(x, y, true, true);
        }
    }

    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..RANDOM_WALLS {
        let (x, y) = (rng.gen_range(0..WIDTH), rng.gen_range(0..HEIGHT));
        map.set(x as i32, y as i32, false, false);
    }
    map.set(POSITION_X as i32, POSITION_Y as i32, true, true);

    let x = POSITION_X as i32;
    let y = POSITION_Y as i32;
    let radius = RADIUS as i32;
    c.bench_function("tcod_fov_random_walls", |bencher| {
        bencher.iter(|| map.compute_fov(x, y, radius, true, tcod::map::FovAlgorithm::Basic));
    });
}

pub fn bracket_fov_no_walls(c: &mut Criterion) {
    let map = SampleMap::new(WIDTH, HEIGHT);

    c.bench_function("bracket_fov_no_walls", |bencher| {
        bencher.iter(|| {
            bracket_pathfinding::prelude::field_of_view(
                (POSITION_X, POSITION_Y).into(),
                RADIUS,
                &map,
            )
        });
    });
}

pub fn bracket_fov_random_walls(c: &mut Criterion) {
    let map = SampleMap::new(WIDTH, HEIGHT).randomize_walls();

    c.bench_function("bracket_fov_random_walls", |bencher| {
        bencher.iter(|| {
            bracket_pathfinding::prelude::field_of_view(
                (POSITION_X, POSITION_Y).into(),
                RADIUS,
                &map,
            )
        });
    });
}

criterion_group!(
    benches,
    torchbearer_fov_no_walls,
    torchbearer_fov_random_walls,
    tcod_fov_no_walls,
    tcod_fov_random_walls,
    bracket_fov_no_walls,
    bracket_fov_random_walls
);
criterion_main!(benches);
