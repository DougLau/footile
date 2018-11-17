# footile
A 2D vector graphics library written in Rust.

## Goals
* API simplicity and ergonomics
* Features comparable to other 2D graphics (Cairo, Skia, SVG)
* Anti-aliased rendering
* Image rendering for web servers
* (Someday) GPU acceleration (hopefully using SPIR-V as rust compile target!)
* Have fun!

## Example
```rust
use footile::{FillRule,PathBuilder,Plotter,Rgba8};

let fish = PathBuilder::new().relative().pen_width(3.0)
                       .move_to(112.0, 24.0)
                       .line_to(-32.0, 24.0)
                       .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
                       .line_to(32.0, 24.0)
                       .line_to(-16.0, -40.0)
                       .close().build();
let mut p = Plotter::new(128, 128);
p.fill(&fish, FillRule::NonZero).over(Rgba8::rgb(127, 96, 96));
p.stroke(&fish).over(Rgba8::rgb(255, 208, 208));
p.write_png("./fishy.png")?;
```
