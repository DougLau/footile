# footile
A 2D vector graphics library written in Rust

## Example
```rust
use footile::{Color,FillRule,PathBuilder,Plotter};

let fish = PathBuilder::new().relative().pen_width(3f32)
                       .move_to(112f32, 24f32)
                       .line_to(-32f32, 24f32)
                       .cubic_to(-96f32, -48f32, -96f32, 80f32, 0f32, 32f32)
                       .line_to(32f32, 24f32)
                       .line_to(-16f32, -40f32)
                       .close().build();
let mut p = Plotter::new(128, 128);
p.fill(&fish, FillRule::NonZero).composite(Color::rgb(127, 96, 96));
p.stroke(&fish).composite(Color::rgb(255, 208, 208));
p.raster().unwrap().write_png("./fishy.png")?;
```
