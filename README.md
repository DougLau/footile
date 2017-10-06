# footile
A 2D vector graphics library written in Rust

## Example
```rust
use footile::{FillRule, PlotterBuilder, Raster};

let mut p = PlotterBuilder::new().width(128).height(128).build();
let mut r = Raster::new(p.width(), p.height());
p.pen_width(3f32, false);
p.move_to(112f32, 24f32);
p.line_to(-32f32, 24f32);
p.cubic_to(-96f32, -48f32, -96f32, 80f32, 0f32, 32f32);
p.line_to(32f32, 24f32);
p.line_to(-16f32, -40f32);
p.close();
p.fill(FillRule::EvenOdd);
r.composite(p.mask(), [127u8, 96u8, 96u8]);
p.clear();
p.stroke();
r.composite(p.mask(), [255u8, 208u8, 208u8]);
r.write_png("./fishy.png")?;
```
