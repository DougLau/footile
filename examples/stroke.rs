// stroke.rs
use footile::{PathBuilder, Plotter};

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::default()
        .relative()
        .pen_width(5.0)
        .move_to(16.0, 48.0)
        .line_to(32.0, 0.0)
        .line_to(-16.0, -32.0)
        .close()
        .build();
    let mut p = Plotter::new(64, 64);
    png::write_matte(p.stroke(&path), "./stroke.png")
}
