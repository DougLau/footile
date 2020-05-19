#[macro_use]
extern crate criterion;

use criterion::Criterion;
use footile::*;
use pix::matte::Matte8;
use pix::Raster;

fn fill_16(c: &mut Criterion) {
    c.bench_function("fill_16", |b| b.iter(|| fill(16)));
}

fn fill_256(c: &mut Criterion) {
    c.bench_function("fill_256", |b| b.iter(|| fill(256)));
}

fn fill(i: u32) {
    make_plotter(i).fill(FillRule::NonZero, &make_fishy(), Matte8::new(255));
}

fn stroke_16(c: &mut Criterion) {
    c.bench_function("stroke_16", |b| b.iter(|| gray_stroke(16)));
}

fn stroke_256(c: &mut Criterion) {
    c.bench_function("stroke_256", |b| b.iter(|| gray_stroke(256)));
}

fn gray_stroke(i: u32) {
    make_plotter(i).stroke(&make_fishy(), Matte8::new(255));
}

fn make_plotter(i: u32) -> Plotter<Matte8> {
    let r = Raster::with_clear(i, i);
    let mut p = Plotter::new(r);
    p.set_transform(Transform::with_scale(2.0, 2.0));
    p
}

fn make_fishy() -> Vec<PathOp> {
    Path2D::default()
        .relative()
        .move_to(112.0, 16.0)
        .line_to(-48.0, 32.0)
        .cubic_to(-64.0, -48.0, -64.0, 80.0, 0.0, 32.0)
        .line_to(48.0, 32.0)
        .line_to(-32.0, -48.0)
        .close()
        .finish()
}

criterion_group!(benches, fill_16, fill_256, stroke_16, stroke_256);
criterion_main!(benches);
