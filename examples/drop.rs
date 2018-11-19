// drop.rs
extern crate footile;

use footile::{FillRule,Gray8,PathBuilder,Plotter,Raster};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative().pen_width(3.0)
                           .move_to(50.0, 34.0)
                           .cubic_to(4.0, 16.0, 16.0, 28.0, 0.0, 32.0)
                           .cubic_to(-16.0, -4.0, -4.0, -16.0, 0.0, -32.0)
                           .close()
                           .build();
    let mut p = Plotter::new(100, 100);
    let mut r = Raster::new(p.width(), p.height());
    r.over(p.fill(&path, FillRule::NonZero), Gray8::new(128));
    r.over(p.stroke(&path), Gray8::new(255));
    r.write_png("./drop.png")
}
