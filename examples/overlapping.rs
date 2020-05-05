use footile::{FillRule, PathOp::*, Plotter, Pt};
use pix::matte::Matte8;
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let path = vec![
        Move(Pt(8.0, 4.0)),
        Line(Pt(8.0, 3.0)),
        Cubic(Pt(8.0, 3.0), Pt(8.0, 3.0), Pt(9.0, 3.75)),
        Line(Pt(8.0, 3.75)),
        Line(Pt(8.5, 3.75)),
        Line(Pt(8.5, 3.5)),
        Close(),
    ];
    let r = Raster::with_clear(64, 64);
    let mut p = Plotter::new(r);
    p.fill(FillRule::NonZero, &path, Matte8::new(255));
    png::write_matte(&p.raster(), "./overlapping.png")
}
