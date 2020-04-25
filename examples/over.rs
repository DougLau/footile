// over.rs
use footile::{PathBuilder, Plotter};

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new()
        .relative()
        .pen_width(8.0)
        .move_to(32.0, 16.0)
        .line_to(16.0, 16.0)
        .line_to(-16.0, 16.0)
        .line_to(-16.0, -16.0)
        .line_to(16.0, -16.0)
        .line_to(0.0, 32.0)
        .build();
    let mut p = Plotter::new(64, 64);
    png::write_matte(p.stroke(&path), "./over.png")
}
