use footile::{PathBuilder, Plotter};

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = PathBuilder::default()
        .relative()
        .pen_width(0.0)
        .move_to(64.0, 48.0)
        .pen_width(18.0)
        .cubic_to(-64.0, -48.0, -64.0, 80.0, 0.0, 32.0)
        .build();
    let mut p = Plotter::new(128, 128);
    png::write_matte(p.stroke(&path), "./curve.png")
}
