# footile
A 2D vector graphics library written in Rust

## Example
```rust
use footile::{FillRule, PathBuilder, Plotter, Raster};

let fish = PathBuilder::new().relative().pen_width(3f32)
                       .move_to(112f32, 24f32)
                       .line_to(-32f32, 24f32)
                       .cubic_to(-96f32, -48f32, -96f32, 80f32, 0f32, 32f32)
                       .line_to(32f32, 24f32)
                       .line_to(-16f32, -40f32)
                       .close().build();
let mut p = Plotter::new(128, 128);
let mut r = Raster::new(p.width(), p.height());
p.add_ops(&fish);
p.fill(FillRule::NonZero);
r.composite(p.mask(), [127u8, 96u8, 96u8]);
p.clear().stroke();
r.composite(p.mask(), [255u8, 208u8, 208u8]);
r.write_png("./fishy.png")?;
```
