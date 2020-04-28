use footile::{PathBuilder, Plotter};

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::default()
        .relative()
        .move_to(0.0, 8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .move_to(-64.0, 32.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .line_to(8.0, 8.0)
        .line_to(8.0, -8.0)
        .build();
    let mut p = Plotter::new(64, 64);
    png::write_matte(p.stroke(&path), "./teeth.png")
}
