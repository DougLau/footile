// quad.rs     Example plotting a quadratic bézier spline.
extern crate footile;

use footile::PlotterBuilder;

fn main() {
    let mut p = PlotterBuilder::new()
                               .width(64)
                               .height(64)
                               .build();
    p.pen_width(2f32)
     .move_to(0f32, 16f32)
     .quad_to(100f32, 16f32, 0f32, 32f32)
     .stroke();
    p.mask().write_png("./quad.png").unwrap();
}
