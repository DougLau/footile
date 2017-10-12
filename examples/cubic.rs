// cubic.rs     Example plotting a cubic bézier spline.
extern crate footile;

use footile::PlotterBuilder;

fn main() {
    let mut p = PlotterBuilder::new()
                               .width(64)
                               .height(64)
                               .build();
    p.pen_width(2f32)
     .move_to(8f32, 16f32)
     .cubic_to(64f32, -16f32, 64f32, 48f32, 0f32, 32f32)
     .stroke();
    p.mask().write_png("./cubic.png").unwrap();
}
