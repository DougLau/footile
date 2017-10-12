// heptagram.rs     Example heptagram plot.
extern crate footile;

use footile::{ FillRule, PlotterBuilder, Transform };

fn main() {
    const PI: f32 = std::f32::consts::PI;
    let mut p = PlotterBuilder::new().absolute().build();
    let h = (p.width() / 2u32) as f32;
    p.set_transform(Transform::new_scale(h, h).translate(h, h));
    p.move_to(0f32.cos(), 0f32.sin());
    for n in 1..7 {
        let th = PI * 4f32 * (n as f32) / 7f32;
        p.line_to(th.cos(), th.sin());
    }
    p.close().fill(FillRule::EvenOdd);
    p.mask().write_png("./heptagram.png").unwrap();
}
