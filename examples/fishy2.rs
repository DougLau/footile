// fishy2.rs
extern crate footile;

use footile::{FillRule,PathBuilder,Plotter,Raster,Rgba8};

fn main() -> Result<(), std::io::Error> {
    let fish = PathBuilder::new().relative().pen_width(3.0)
                           .move_to(112.0, 24.0)
                           .line_to(-32.0, 24.0)
                           .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
                           .line_to(32.0, 24.0)
                           .line_to(-16.0, -40.0)
                           .close().build();
    let eye = PathBuilder::new().relative().pen_width(2.0)
                          .move_to(24.0, 48.0)
                          .line_to(8.0, 8.0)
                          .move_to(0.0, -8.0)
                          .line_to(-8.0, 8.0)
                          .build();
    let v = vec![Rgba8::new(0,0,0,0); 128*128];
    let mut p = Plotter::new(128, 128);
    let mut r = Raster::with_pixels(p.width(), p.height(), v);
    r.over(p.fill(&fish, FillRule::NonZero), Rgba8::rgb(127, 96, 96));
    r.over(p.stroke(&fish), Rgba8::rgb(255, 208, 208));
    r.over(p.stroke(&eye), Rgba8::rgb(0, 0, 0));
    r.write_png("./fishy2.png")
}
