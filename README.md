# footile
A 2D vector graphics library written in Rust.

## Documentation
[https://docs.rs/footile](https://docs.rs/footile)

## Example
```rust
use footile::{FillRule, PathBuilder, Plotter};

let fish = PathBuilder::new()
    .relative()
    .pen_width(3.0)
    .move_to(112.0, 24.0)
    .line_to(-32.0, 24.0)
    .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
    .line_to(32.0, 24.0)
    .line_to(-16.0, -40.0)
    .close()
    .build();
let mut p = Plotter::new(128, 128);
let raster = p.fill(&fish, FillRule::NonZero);
```

## Goals

* API simplicity and ergonomics
* Features comparable to other 2D graphics (Cairo, Skia, SVG)
* Anti-aliased rendering
* Image rendering for web servers
* Have fun!
