// figure.rs        Example using a figure
extern crate footile;

use footile::{ FillRule, PlotterBuilder };

fn main() {
    let mut p = PlotterBuilder::new()
                               .width(64)
                               .height(64)
                               .build();
    p.move_to(4f32, 4f32)
     .line_to(28f32, 12f32)
     .line_to(28f32, -12f32)
     .line_to(-12f32, 28f32)
     .line_to(12f32, 28f32)
     .line_to(-28f32, -4f32)
     .line_to(-28f32, 4f32)
     .line_to(12f32, -28f32)
     .close()
     .fill(FillRule::NonZero);
    p.mask().write_png("./figure.png").unwrap();
}
