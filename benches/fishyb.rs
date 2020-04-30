#[macro_use]
extern crate criterion;

use criterion::Criterion;
use footile::*;

fn fill_16(c: &mut Criterion) {
    c.bench_function("fill_16", |b| b.iter(|| fill(16)));
}

fn fill_256(c: &mut Criterion) {
    c.bench_function("fill_256", |b| b.iter(|| fill(256)));
}

fn fill(i: u32) {
    make_plotter(i).fill(&make_fishy(), FillRule::NonZero);
}

fn stroke_16(c: &mut Criterion) {
    c.bench_function("stroke_16", |b| b.iter(|| gray_stroke(16)));
}

fn stroke_256(c: &mut Criterion) {
    c.bench_function("stroke_256", |b| b.iter(|| gray_stroke(256)));
}

fn gray_stroke(i: u32) {
    make_plotter(i).stroke(&make_fishy());
}

fn make_plotter(i: u32) -> Plotter {
    let mut p = Plotter::new(i, i);
    p.set_transform(Transform::new_scale(2.0, 2.0));
    p
}

fn make_fishy() -> Path2D {
    PathBuilder::default()
        .relative()
        .move_to(112.0, 16.0)
        .line_to(-48.0, 32.0)
        .cubic_to(-64.0, -48.0, -64.0, 80.0, 0.0, 32.0)
        .line_to(48.0, 32.0)
        .line_to(-32.0, -48.0)
        .close()
        .build()
}

criterion_group!(benches, fill_16, fill_256, stroke_16, stroke_256);
criterion_main!(benches);
