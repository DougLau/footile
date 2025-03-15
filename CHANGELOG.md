## [Unreleased]

### Changed
* Updated `pointy` to v0.7

## [0.7.0] - 2022-06-01
### Added
* `Plotter.raster()` and `raster_mut()` (returning reference)
### Changed
* Old `Plotter.raster()` to `Plotter.into_raster()`
* Use `pointy` crate for 2D geometry

## [0.6.0] - 2020-09-19
### Added
* PathOp, FillRule, JoinStyle, etc. now implement Debug
### Changed
* Renamed PathBuilder to Path2D
### Removed
* Old Path2D (just use `Vec<PathOp>` instead)

## [0.5.0] - 2020-05-19
### Changed
* Replaced PathBuilder::new() with default() (Default impl)
* Replaced Transform::new() with default() (Default implt)
* Renamed Vec2/Vec2w to Pt/WidePt (to avoid confustion with Vec)
* Simplify Plotter API -- allow plotting directly onto provided Raster

## [0.4.0] - 2020-04-24
### Changed
* Renamed "mask" to "matte"
### Removed
* Moved Raster and supporting code to pix crate

## [0.3.1] - 2019-03-07
### Added
* Support `target_arch` = "wasm32"
* New use-simd feature

## [0.3.0] - 2019-01-21
### Added
* Rgb8 pixel format.
### Changed
* PixFmt::over mask parameter changed from &Mask to &[u8].
### Fixed
* Rendering bug (issue #15)
* Rendering bug (issue #17)

## [0.2.0] - 2018-11-20
### Added
* PixFmt, Rgba8, Gray8 pixel formats.
* Raster now has a pixel format type parameter.
* Raster::width(), height() and `as_*_slice` methods.
* RasterB (for borrowed pixels).
### Removed
* Plotter::over, raster and `write_png` methods.
* Plotter no longer has an associated Raster
### Changed
* Plotter::fill and stroke now return Mask reference.
* Raster::over clears the mask before returning.

## [0.1.1] - 2018-11-15
### Fixed
* Fixed several rendering bugs.
### Changed
* Moved fixed-point code to fixed module.
* Code cleanups.

## [0.1.0] - 2018-11-11
### Added
* Plotter::raster, `Plotter::write_png`
### Removed
* `Plotter::add_path`, Plotter::reset
### Changed
* Plotter::fill/stroke now take a PathOp iterator.
* Plotter: renamed clear to `clear_mask`.
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
* `Plotter::set_transform`
### Removed
* Plotter::{scale,translate,rotate,`skew_x`,`skew_y`}
### Changed
* Implemented more accurate rendering algorithm (cumulative sum).
* Cleaned up example programs.

## [0.0.8] - 2017-10-11
### Added
* Plotter::{scale,translate,rotate,`skew_x`,`skew_y`}

## [0.0.7] - 2017-10-10
### Fixed
* Fixed some rendering glitches.
### Added
* Added C SIMD code for rendering mask.
* Added plotting benchmarks.
### Removed
* `Plotter::write_png`
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
