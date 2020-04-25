use footile::{Plotter, PathOp::*};

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = vec![
        Move(8.0, 4.0),
        Line(8.0, 3.0),
        Cubic(8.0, 3.0, 8.0, 3.0, 9.0, 3.75),
        Line(8.0, 3.75),
        Line(8.5, 3.75),
        Line(8.5, 3.5),
        Close(),
    ];
    let mut p = Plotter::new(64, 64);

    png::write_matte(p.fill(&path, footile::FillRule::NonZero), "./overlapping.png")
}
