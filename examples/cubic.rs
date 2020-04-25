// drop.rs
use footile::{PathBuilder, Plotter};

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::new()
        .relative()
        .pen_width(2.0)
        .move_to(8.0, 16.0)
        .cubic_to(64.0, -16.0, 64.0, 48.0, 0.0, 32.0)
        .build();
    let mut p = Plotter::new(64, 64);
    png::write_matte(p.stroke(&path), "./cubic.png")
}
