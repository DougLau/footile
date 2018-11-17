## [Unreleased]
### Added
* Rgba8, Gray8 pixel formats.
* Raster::width(), height() and pixels()
### Changed
* Raster and Plotter now have pixel format type parameters.
* Plotter::color_over renamed to over.

## [0.1.1] - 2018-11-15
### Fixed
* Fixed several rendering bugs.
### Changed
* Moved fixed-point code to fixed module.
* Code cleanups.

## [0.1.0] - 2018-11-11
### Added
* Plotter::raster, Plotter::write_png
### Removed
* Plotter::add_path, Plotter::reset
### Changed
* Plotter::fill/stroke now take a PathOp iterator.
* Plotter: renamed clear to clear_mask.
* Converted SIMD code from C to rust.
* Converted benchmarks to use criterion-rs.
* Moved stroker into its own module.
* Optimized alpha blending using SIMD.

## [0.0.10] - 2017-10-25
### Added
* PathBuilder, Path2D
### Removed
* PlotterBuilder
### Changed
* Renamed Vec3 to Vec2w.

## [0.0.9] - 2017-10-20
### Added
* Transform struct
* Plotter::set_transform
### Removed
* Plotter::{scale,translate,rotate,skew_x,skew_y}
### Changed
* Implemented more accurate rendering algorithm (cumulative sum).
* Cleaned up example programs.

## [0.0.8] - 2017-10-11
### Added
* Plotter::{scale,translate,rotate,skew_x,skew_y}

## [0.0.7] - 2017-10-10
### Fixed
* Fixed some rendering glitches.
### Added
* Added C SIMD code for rendering mask.
* Added plotting benchmarks.
### Removed
* Plotter::write_png
### Changed
* Use type alias for vertex IDs.
* Moved test code into separate modules.

## [0.0.6] - 2017-10-06
### Added
* Mask struct
* Raster struct
* JoinStyle::Round
* license file
### Removed
* Removed Vec2 from public API.

## [0.0.5] - 2017-10-04
### Fixed
* Fixed stroking problems.
### Added
* Added PNG output for masks.

## [0.0.4] - 2017-10-03
### Changed
* Reworked Plotter API.

## [0.0.3] - 2017-10-02
### Added
* Added support for miter limits.
### Removed
* Removed Fig from public API.

## [0.0.2] - 2017-10-01
### Added
* Added some example programs.
### Changed
* Cleaned up public API.
* Improved documentation.

## [0.0.1] - 2017-10-01
* Initial conversion of C code to rust.