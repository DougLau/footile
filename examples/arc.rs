// arc.rs
use footile::{Path2D, Plotter};
use pix::matte::Matte8;
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let cs: Vec<(f32, f32, f32, f32, f32)> = vec![
        (24.0, 24.0, 6.0, 0.0, -280.0),
        (40.0, 24.0, -10.0, 0.0, 279.0),
        (40.0, 40.0, 0.0, -8.0, 278.0),
        (24.0, 40.0, 12.0, 0.0, 277.0),
    ];
    let mut path1 = Path2D::default().absolute().pen_width(2.0);
    let mut path2 = Path2D::default().absolute().pen_width(2.0);
    for (x, y, rx, ry, a) in cs.iter().map(|(a, b, c, d, e)| {
        (a * 10.0, b * 10.0, c * 10.0, d * 10.0, e)
    }) {
        path1 = path1
            .move_to(x + rx, y + ry)
            .arc_sweep(x, y, a.to_radians());
        path2 = path2.line_to(x, y);
    }

    let ops1 = path1.finish();
    let mut ops2 = path2.line_to(240.0, 240.0).close().finish();
    let r = Raster::with_clear(640, 640);
    let mut p = Plotter::new(r);
    ops2.extend(ops1.iter());

    png::write_matte(p.stroke(&ops2, Matte8::new(255)), "./arc.png")
}
