//! Collection of utility functions to find path.

use std::{cmp::Ordering, collections::BinaryHeap};

use crate::Point;

pub type NodeId = usize;

/// Implement the Map trait to use the pathfinding functions.
pub trait PathMap {
    /// Dimension of your map, in grid size.
    fn dimensions(&self) -> (i32, i32);
    /// Wether it is possible or not to walk through the tile at position `(x, y)`.
    /// Used by pathfinding algorithm.
    fn is_walkable(&self, position: Point) -> bool;
}

/// An A* pathfinding implementation for a grid base map, where diagonal movements are disabled.
/// Returns an optional vector containing the several points on the map to walk through, including the origin and destination.
///
/// Implements the algorithm and fixes found on the
/// [redblobgames.com](https://www.redblobgames.com/pathfinding/a-star/implementation.html#python-astar).
///
/// Uses a binary heap as described in the [rust-lang](https://doc.rust-lang.org/stable/std/collections/binary_heap/) doc.
///
/// # Arguments
///
/// * `map` - a struct implementing the `Map` trait.
/// * `from` - the origin.
/// * `to` - the destination.
///
/// # Panics
///
/// Panics if `from` or `to` are out of bounds of the map.
///
/// # Examples
/// ```
/// use torchbearer::{
///     path::{astar_path_fourwaygrid, PathMap},
///     Point,
/// };
///
/// struct SampleMap {
///     width: i32,
///     height: i32,
///     walkable: Vec<bool>,
/// }
///
/// impl SampleMap {
///     fn new(width: i32, height: i32) -> Self {
///         // (…)
/// #        SampleMap {
/// #            width,
/// #            height,
/// #            walkable: vec![true; (width * height) as usize],
/// #        }
///     }
/// }
///
/// impl PathMap for SampleMap {
///     fn dimensions(&self) -> (i32, i32) {
///         (self.width, self.height)
///     }
///
///     fn is_walkable(&self, (x, y): Point) -> bool {
///         self.walkable[(x + y * self.width) as usize]
///     }
/// }
///
/// let sample_map = SampleMap::new(16, 10);
///
/// // (…) You probably want at this point to add some walls to your map.
///
/// if let Some(path) = astar_path_fourwaygrid(&sample_map, (1, 1), (3, 8)) {
///     // (…)
/// }
/// ```
pub fn astar_path_fourwaygrid<T: PathMap>(map: &T, from: Point, to: Point) -> Option<Vec<Point>> {
    fn assert_in_bounds<T: PathMap>(map: &T, (x, y): Point) {
        let (width, height) = map.dimensions();
        if x < 0 || y < 0 || x >= width || y >= height {
            panic!(
                "(x, y) should be between (0,0) and ({}, {}), got ({}, {}).",
                width, height, x, y
            );
        }
    }

    assert_in_bounds(map, from);
    assert_in_bounds(map, to);

    let graph = FourWayGridGraph::new(map);
    astar_path(&graph, graph.point_to_index(from), graph.point_to_index(to)).map(|indices| {
        indices
            .into_iter()
            .map(|index| graph.index_to_point(index))
            .collect()
    })
}

/// An A* pathfinding implementation for a grid base map.
/// Returns an optional vector containing the several points on the map to walk through, including the origin and destination.
///
/// Implements the algorithm and fixes found on the
/// [redblobgames.com](https://www.redblobgames.com/pathfinding/a-star/implementation.html#python-astar).
///
/// Uses a binary heap as described in the [rust-lang](https://doc.rust-lang.org/stable/std/collections/binary_heap/) doc.
///
/// # Arguments
///
/// * `graph` - a struct implementing the `Graph` trait.
/// * `from_index` - the origin.
/// * `to_index` - the destination.
///
/// # Panics
///
/// Panics if `from_index` or `to_index` are out of bounds. (Meaning, a bigger index that the total node count of the graph).
///
/// # Examples
/// ```
/// use torchbearer::{
///     path::{astar_path, FourWayGridGraph, PathMap},
///     Point,
/// };
///
/// struct SampleMap {
///     width: i32,
///     height: i32,
///     walkable: Vec<bool>,
/// }
///
/// impl SampleMap {
///     fn new(width: i32, height: i32) -> Self {
///         // (…)
/// #        SampleMap {
/// #            width,
/// #            height,
/// #            walkable: vec![true; (width * height) as usize],
/// #        }
///     }
/// }
///
/// impl PathMap for SampleMap {
///     fn dimensions(&self) -> (i32, i32) {
///         (self.width, self.height)
///     }
///
///     fn is_walkable(&self, (x, y): Point) -> bool {
///         self.walkable[(x + y * self.width) as usize]
///     }
/// }
///
/// let width = 16;
/// let height = 10;
/// let sample_map = SampleMap::new(width, height);
///
/// // (…) You probably want at this point to add some walls to your map.
///
/// // Use one of the pre-made graphs (good for simple use cases), or implement your own.
/// let graph = FourWayGridGraph::new(&sample_map);
/// let from = (1 + 1 * width) as usize; // position to index
/// let to = (3 + 8 * width) as usize; // position to index
///
/// if let Some(path) = astar_path(&graph, to, from) {
///     // (…)
/// }
/// ```
pub fn astar_path<T: Graph>(
    graph: &T,
    from_index: NodeId,
    to_index: NodeId,
) -> Option<Vec<NodeId>> {
    fn assert_in_bounds<T: Graph>(graph: &T, index: NodeId) {
        if index >= graph.node_count() {
            panic!(
                "Index {} is out of bounds for a graph of size {}.",
                index,
                graph.node_count()
            );
        }
    }
    assert_in_bounds(graph, from_index);
    assert_in_bounds(graph, to_index);

    let capacity = graph.node_count() / 2;
    let mut frontier = BinaryHeap::with_capacity(capacity);

    frontier.push(State {
        cost: 0.,
        item: from_index,
    });

    let mut came_from: Vec<Option<usize>> = vec![None; graph.node_count()];
    let mut costs: Vec<Option<f32>> = vec![None; graph.node_count()];
    costs[from_index] = Some(0.);
    let mut neighboors: Vec<NodeId> = Vec::with_capacity(4);

    let mut to_cost = 0.;

    while let Some(State {
        item: current_index,
        cost: current_cost,
    }) = frontier.pop()
    {
        if current_index == to_index {
            to_cost = current_cost;
            break;
        }

        neighboors.clear();
        graph.neighboors(current_index, &mut neighboors);
        for &next_index in neighboors.iter() {
            let cost_so_far = costs[current_index].unwrap();
            let new_cost = cost_so_far + graph.cost_between(current_index, next_index);

            if costs[next_index].is_none() || new_cost < costs[next_index].unwrap() {
                let priority = new_cost + graph.heuristic(next_index, to_index);
                frontier.push(State {
                    cost: priority,
                    item: next_index,
                });
                came_from[next_index] = Some(current_index);
                costs[next_index] = Some(new_cost);
            }
        }
    }

    reconstruct_path(from_index, to_index, came_from, to_cost)
}

fn reconstruct_path(
    from: NodeId,
    to: NodeId,
    came_from: Vec<Option<NodeId>>,
    cost: f32,
) -> Option<Vec<NodeId>> {
    let mut current = Some(to);
    let target_index = from;

    let mut path = Vec::with_capacity((cost.floor() + 2.0) as usize);

    while current != Some(target_index) {
        if let Some(position) = current {
            path.push(position);
            current = if let Some(entry) = came_from[position] {
                Some(entry)
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    path.push(target_index);

    Some(path.into_iter().rev().collect())
}

struct State<C: PartialOrd, T> {
    cost: C,
    item: T,
}
impl<C: PartialOrd, T> PartialEq for State<C, T> {
    fn eq(&self, other: &Self) -> bool {
        self.cost.eq(&other.cost)
    }
}

impl<C: PartialOrd, T> Eq for State<C, T> {}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl<C: PartialOrd, T> Ord for State<C, T> {
    fn cmp(&self, other: &State<C, T>) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

// `PartialOrd` needs to be implemented as well.
impl<C: PartialOrd, T> PartialOrd for State<C, T> {
    fn partial_cmp(&self, other: &State<C, T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A graph for the A* algorithm. This is intended for a grid based representation, where each
/// node would be a square on the map.
pub trait Graph {
    /// The amount of nodes in the graph, used to create correctly sized vectors.
    fn node_count(&self) -> usize;

    /// The cost between two points. A higher cost could represent a hard to cross terrain.
    /// If normal terrain would cost 1 to go from a to be, climbing a mountain side could cost 2.
    fn cost_between(&self, a: NodeId, b: NodeId) -> f32;

    /// How close we are from our target.
    /// See <https://www.redblobgames.com/pathfinding/a-star/introduction.html#greedy-best-first>
    /// for more details about how it is useful.
    fn heuristic(&self, a: NodeId, b: NodeId) -> f32;

    /// From point a, where can you go. Create a list of all possible neighboors.
    /// No need to filter the walkable ones, or the one in bounds: the algorithm
    /// does it later for optimisation purposes.
    ///
    /// # Arguments
    ///
    /// * `a` - the position whose neighboors you are looking for.
    /// * `into` - push the neighboors into this vector.
    ///   No need to clear explicitely, as `clear()` is called before each call to this method.
    fn neighboors(&self, a: NodeId, into: &mut Vec<NodeId>);
}

/// A wrapper around a Map, representing the graph for a four way grid type of Map, where
/// it's possible to go north, east, south and west, but not in diagonal.
pub struct FourWayGridGraph<'a, T: PathMap> {
    map: &'a T,
    width: i32,
    height: i32,
}

impl<'a, T: PathMap> FourWayGridGraph<'a, T> {
    pub fn new(map: &'a T) -> Self {
        let (width, height) = map.dimensions();
        FourWayGridGraph { map, width, height }
    }

    /// Is the node at position (x, y) walkable.
    fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.map.is_walkable((x, y))
    }

    fn point_to_index(&self, (x, y): Point) -> usize {
        (x + y * self.width) as usize
    }

    fn index_to_point(&self, index: usize) -> Point {
        (index as i32 % self.width, index as i32 / self.width)
    }
}

impl<'a, T: PathMap> Graph for FourWayGridGraph<'a, T> {
    fn node_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    fn cost_between(&self, a: NodeId, b: NodeId) -> f32 {
        let basic = 1.;
        let (x1, y1) = self.index_to_point(a);
        let (x2, y2) = self.index_to_point(b);
        // Why the nudge? Check https://www.redblobgames.com/pathfinding/a-star/implementation.html#troubleshooting-ugly-path
        // For a path in a 4 way grid, going up 3 times then left 3 times is the same cost as
        // going up then left then up then... So we add a small nudge to the cost to make sure
        // the algorithm doesn't follow straight path when it could go diagonal.
        let nudge = if ((x1 + y1) % 2 == 0 && x2 != x1) || ((x1 + y1) % 2 == 1 && y2 != y1) {
            1.
        } else {
            0.
        };
        basic + 0.001 * nudge
    }

    fn heuristic(&self, a: NodeId, b: NodeId) -> f32 {
        let (xa, ya) = self.index_to_point(a);
        let (xb, yb) = self.index_to_point(b);

        ((xa - xb).abs() + (ya - yb).abs()) as f32
    }

    fn neighboors(&self, a: NodeId, into: &mut Vec<NodeId>) {
        let (x, y) = self.index_to_point(a);

        fn add_to_neighboors_if_qualified<'a, T: PathMap>(
            graph: &FourWayGridGraph<'a, T>,
            (x, y): Point,
            into: &mut Vec<NodeId>,
        ) {
            if x < 0 || y < 0 || x >= graph.width || y >= graph.height || !graph.is_walkable(x, y) {
                return;
            }
            into.push(graph.point_to_index((x, y)));
        }

        add_to_neighboors_if_qualified(self, (x, y + 1), into);
        add_to_neighboors_if_qualified(self, (x, y - 1), into);
        add_to_neighboors_if_qualified(self, (x - 1, y), into);
        add_to_neighboors_if_qualified(self, (x + 1, y), into);
    }
}

#[cfg(test)]
mod tests {
    use crate::{bresenham::BresenhamLine, path::astar_path, Point};

    use super::{astar_path_fourwaygrid, FourWayGridGraph, PathMap};

    struct SampleMap {
        width: i32,
        height: i32,
        walkable: Vec<bool>,
    }

    impl SampleMap {
        fn new(width: i32, height: i32) -> Self {
            SampleMap {
                width,
                height,
                walkable: vec![true; (width * height) as usize],
            }
        }

        fn build_wall(&mut self, from: Point, to: Point) {
            let bresenham = BresenhamLine::new(from, to);
            for (x, y) in bresenham {
                self.walkable[(x + y * self.width) as usize] = false;
            }
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

    #[test]
    fn astar_find_path() {
        let mut map = SampleMap::new(10, 10);
        map.build_wall((3, 3), (3, 6));
        map.build_wall((0, 3), (3, 3));

        let from = (0, 4);
        let to = (5, 4);

        let path = astar_path_fourwaygrid(&map, from, to);
        assert!(path.is_some());

        if let Some(path) = path {
            assert_eq!(from, path[0]);
            assert_eq!(to, path[path.len() - 1]);

            assert_eq!(
                path,
                [
                    (0, 4),
                    (0, 5),
                    (1, 5),
                    (1, 6),
                    (2, 6),
                    (2, 7),
                    (3, 7),
                    (4, 7),
                    (5, 7),
                    (5, 6),
                    (5, 5),
                    (5, 4)
                ]
            );
        }
    }

    #[test]
    fn astar_no_path() {
        let mut map = SampleMap::new(10, 10);
        map.build_wall((3, 3), (3, 6));
        map.build_wall((0, 3), (3, 3));
        map.build_wall((0, 6), (3, 6));

        let from = (0, 4);
        let to = (5, 4);

        let path = astar_path_fourwaygrid(&map, from, to);
        assert!(path.is_none());
    }

    #[test]
    #[should_panic(expected = "Index 120 is out of bounds for a graph of size 100.")]
    fn astar_path_out_of_bounds_index_panics() {
        let map = SampleMap::new(10, 10);
        let graph = FourWayGridGraph::new(&map);

        astar_path(&graph, 0, 120);
    }

    #[test]
    #[should_panic(expected = "(x, y) should be between (0,0) and (10, 10), got (0, 12).")]
    fn astar_fourway_out_of_bounds_index_panics() {
        let map = SampleMap::new(10, 10);

        astar_path_fourwaygrid(&map, (0, 0), (0, 12));
    }
}
