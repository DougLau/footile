// round.rs     Example plotting a rounded stroke
extern crate footile;

use footile::{PlotterBuilder, JoinStyle};

fn main() {
    let mut p = PlotterBuilder::new().build();
    p.pen_width(40f32)
     .join_style(JoinStyle::Round)
     .move_to(10f32, 60f32)
     .line_to(50f32, 0f32)
     .line_to(0f32, -50f32)
     .stroke();
    p.mask().write_png("./round.png").unwrap();
}
