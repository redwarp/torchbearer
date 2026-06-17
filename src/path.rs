//! Collection of utility functions to find path.

use std::{cmp::Ordering, collections::BinaryHeap};

use crate::Point;

/// Identifies a node in a [`Graph`] (for a grid, typically the tile's row-major
/// index `x + y * width`). Used to index the algorithm's internal buffers, so a
/// graph's ids should be a contiguous range `0..node_count`.
pub type NodeId = usize;

/// The cost of moving between nodes (e.g. a step's price, or a heuristic estimate).
/// Higher means more expensive terrain.
pub type Cost = f32;

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
///     Point,
///     path::{PathMap, astar_path_fourwaygrid},
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
///     Point,
///     path::{FourWayGridGraph, PathMap, astar_path},
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
        cost_from_start: 0.,
        item: from_index,
    });

    // `usize::MAX` is the "no predecessor" sentinel, avoiding the 16-byte
    // `Option<usize>` (usize has no niche) and halving this allocation.
    let mut came_from: Vec<NodeId> = vec![usize::MAX; graph.node_count()];
    // `f32::INFINITY` is the "not visited yet" sentinel: any real cost compares
    // less than it, so `new_cost < costs[next]` handles the unvisited case too.
    let mut costs: Vec<Cost> = vec![f32::INFINITY; graph.node_count()];
    costs[from_index] = 0.;
    let mut neighbors: Vec<(NodeId, Cost)> = Vec::with_capacity(4);

    let mut to_cost = 0.;

    while let Some(State {
        item: current_index,
        cost_from_start,
        ..
    }) = frontier.pop()
    {
        if current_index == to_index {
            to_cost = cost_from_start;
            break;
        }

        // Skip stale duplicates: the heap emulates decrease-key by re-pushing a
        // node whenever a cheaper route to it is found, leaving the older, pricier
        // entries behind. If this entry's cost is worse than the best recorded for
        // the node, a better one already superseded it, so expanding it is wasted
        // work.
        if cost_from_start > costs[current_index] {
            continue;
        }

        neighbors.clear();
        graph.neighbors(current_index, &mut neighbors);
        for &(next_index, step_cost) in neighbors.iter() {
            let new_cost = cost_from_start + step_cost;

            if new_cost < costs[next_index] {
                let priority = new_cost + graph.heuristic(next_index, to_index);
                frontier.push(State {
                    cost: priority,
                    cost_from_start: new_cost,
                    item: next_index,
                });
                came_from[next_index] = current_index;
                costs[next_index] = new_cost;
            }
        }
    }

    reconstruct_path(from_index, to_index, came_from, to_cost)
}

fn reconstruct_path(
    from: NodeId,
    to: NodeId,
    came_from: Vec<NodeId>,
    cost: Cost,
) -> Option<Vec<NodeId>> {
    let mut current = to;
    let target_index = from;

    let mut path = Vec::with_capacity((cost.floor() + 2.0) as usize);

    while current != target_index {
        path.push(current);
        let entry = came_from[current];
        if entry == usize::MAX {
            // No predecessor recorded: `to` was never reached.
            return None;
        }
        current = entry;
    }
    path.push(target_index);
    path.reverse();
    Some(path)
}

struct State<C: PartialOrd, T> {
    /// Heap ordering key: the priority (cost from start + heuristic).
    cost: C,
    /// The cost from the start to this node, carried alongside so a popped entry
    /// can be checked against the best known cost without recomputing the heuristic.
    cost_from_start: C,
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

    /// How close we are from our target.
    /// See <https://www.redblobgames.com/pathfinding/a-star/introduction.html#greedy-best-first>
    /// for more details about how it is useful.
    fn heuristic(&self, a: NodeId, b: NodeId) -> Cost;

    /// From node `a`, where can you go, and at what cost to step there. Push each
    /// reachable neighbour together with the cost of moving onto it; a higher cost
    /// could represent harder terrain (normal terrain might cost 1, climbing a
    /// mountain side 2). Only push neighbours that are actually walkable and in
    /// bounds — the algorithm trusts every entry and does not re-check.
    ///
    /// # Arguments
    ///
    /// * `a` - the node whose neighbours you are looking for.
    /// * `into` - push the `(neighbour, cost)` pairs into this vector.
    ///   No need to clear explicitely, as `clear()` is called before each call to this method.
    fn neighbors(&self, a: NodeId, into: &mut Vec<(NodeId, Cost)>);
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

    fn heuristic(&self, a: NodeId, b: NodeId) -> Cost {
        let (xa, ya) = self.index_to_point(a);
        let (xb, yb) = self.index_to_point(b);

        ((xa - xb).abs() + (ya - yb).abs()) as f32
    }

    fn neighbors(&self, a: NodeId, into: &mut Vec<(NodeId, Cost)>) {
        let (x, y) = self.index_to_point(a);
        // Parity of the source tile, computed once for all four edges.
        let source_even = (x + y) % 2 == 0;

        // Why the nudge? Check https://www.redblobgames.com/pathfinding/a-star/implementation.html#troubleshooting-ugly-path
        // For a path in a 4 way grid, going up 3 times then left 3 times is the same cost as
        // going up then left then up then... So we add a small nudge to the cost to make sure
        // the algorithm doesn't follow straight path when it could go diagonal.
        // The nudge applies when the source-tile parity matches the move direction.
        fn add_if_qualified<'a, T: PathMap>(
            graph: &FourWayGridGraph<'a, T>,
            (x, y): Point,
            source_even: bool,
            moves_horizontally: bool,
            into: &mut Vec<(NodeId, Cost)>,
        ) {
            if x < 0 || y < 0 || x >= graph.width || y >= graph.height || !graph.is_walkable(x, y) {
                return;
            }
            let nudge = if source_even == moves_horizontally {
                1.
            } else {
                0.
            };
            into.push((graph.point_to_index((x, y)), 1. + 0.001 * nudge));
        }

        add_if_qualified(self, (x, y + 1), source_even, false, into);
        add_if_qualified(self, (x, y - 1), source_even, false, into);
        add_if_qualified(self, (x - 1, y), source_even, true, into);
        add_if_qualified(self, (x + 1, y), source_even, true, into);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Point, bresenham::BresenhamLine, path::astar_path};

    use super::{FourWayGridGraph, PathMap, astar_path_fourwaygrid};

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
