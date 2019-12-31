// figure.rs
use footile::{FillRule,PathBuilder,Plotter};

pub mod png;

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
    png::write_mask(p.fill(&path, FillRule::NonZero), "./figure.png")
}
