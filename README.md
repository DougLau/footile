# footile
A 2D vector graphics library written in Rust

## Example
```rust
use footile::PlotterBuilder;

let mut p = PlotterBuilder::new().build();
p.pen_width(3f32, false);
p.move_to(50f32, 34f32);
p.cubic_to(4f32, 16f32, 16f32, 28f32, 0f32, 32f32);
p.cubic_to(-16f32, -4f32, -4f32, -16f32, 0f32, -32f32);
p.close();
p.stroke();
p.write_png("./drop.png").unwrap();
```
