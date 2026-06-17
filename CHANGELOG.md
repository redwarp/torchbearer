# Changelog

## [0.7.0] - 2026-06-17

### Bug Fixes

* Remove unecessary vec allocation

### Miscellaneous Tasks

* Migrate to the Rust 2024 edition

### Performance

* Speed up A* search loop
* [**breaking**] Yield neighbours with their cost from the Graph trait

## [0.6.1] - 2023-01-10

### Bug Fixes

* Detect panic early with explicit messages.

## [0.6.0] - 2023-01-09

### Features

* In FOV, Cast ray around a circle instead of square (#1)

## [0.5.3] - 2022-06-06

### Bug Fixes

* Apply clippy's suggestions

## [0.5.2] - 2022-01-07

### Bug Fixes

* Remove useless main.rs

## [0.5.1] - 2022-01-07

### Bug Fixes

* Cleanup BresenhamLine, speeding up fov.

## [0.5.0] - 2021-12-30

### Features

* [**breaking**] Split the Map trait into VisionMap and PathMap

## [0.4.0] - 2021-12-30
