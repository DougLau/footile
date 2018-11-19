// figure.rs
extern crate footile;

use footile::{FillRule,PathBuilder,Plotter};

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new().relative()
                           .move_to(4.0, 4.0)
                           .line_to(28.0, 12.0)
                           .line_to(28.0, -12.0)
                           .line_to(-12.0, 28.0)
                           .line_to(12.0, 28.0)
                           .line_to(-28.0, -4.0)
                           .line_to(-28.0, 4.0)
                           .line_to(12.0, -28.0)
                           .close().build();
    let mut p = Plotter::new(64, 64);
    p.fill(&path, FillRule::NonZero).write_png("./figure.png")
}
