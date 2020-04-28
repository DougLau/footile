// round.rs
use footile::{JoinStyle, PathBuilder, Plotter};

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::default()
        .relative()
        .pen_width(40.0)
        .move_to(10.0, 60.0)
        .line_to(50.0, 0.0)
        .line_to(0.0, -50.0)
        .build();
    let mut p = Plotter::new(100, 100);
    p.set_join(JoinStyle::Round);
    png::write_matte(p.stroke(&path), "./round.png")
}
