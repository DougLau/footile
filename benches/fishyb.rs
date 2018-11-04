#[macro_use]
extern crate criterion;
extern crate footile;

use footile::*;
use criterion::Criterion;

fn fill_fishy(c: &mut Criterion) {
    c.bench_function("fill_fishy", |b| b.iter(|| fill_fishy2()));
}

fn fill_fishy2() {
    make_plotter().fill(&make_fishy(), FillRule::NonZero);
}

fn stroke_fishy(c: &mut Criterion) {
    c.bench_function("stroke_fishy", |b| b.iter(|| stroke_fishy2()));
}

fn stroke_fishy2() {
    make_plotter().stroke(&make_fishy());
}

fn make_plotter() -> Plotter {
    let mut p = Plotter::new(256, 256);
    p.set_transform(Transform::new_scale(2f32, 2f32));
    p
}

fn make_fishy() -> Path2D {
    PathBuilder::new().relative()
                .move_to(112f32, 16f32)
                .line_to(-48f32, 32f32)
                .cubic_to(-64f32, -48f32, -64f32, 80f32, 0f32, 32f32)
                .line_to(48f32, 32f32)
                .line_to(-32f32, -48f32)
                .close()
                .build()
}

criterion_group!(benches, fill_fishy, stroke_fishy);
criterion_main!(benches);
