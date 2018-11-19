#[macro_use]
extern crate criterion;
extern crate footile;

use footile::*;
use criterion::Criterion;

fn fill_256(c: &mut Criterion) {
    c.bench_function("fill_256", |b| b.iter(|| fill(256)));
}

fn fill_512(c: &mut Criterion) {
    c.bench_function("fill_512", |b| b.iter(|| fill(512)));
}

fn fill(i: u32) {
    make_plotter(i).fill(&make_fishy(), FillRule::NonZero);
}

fn stroke_256(c: &mut Criterion) {
    c.bench_function("stroke_256", |b| b.iter(|| gray_stroke(256)));
}

fn stroke_512(c: &mut Criterion) {
    c.bench_function("stroke_512", |b| b.iter(|| gray_stroke(512)));
}

fn gray_stroke(i: u32) {
    make_plotter(i).stroke(&make_fishy());
}

fn gray_over_256(c: &mut Criterion) {
    let mut p = make_plotter(256);
    p.fill(&make_fishy(), FillRule::NonZero);
    c.bench_function("gray_over_256", move |b| {
        let mut r = Raster::new(p.width(), p.height());
        b.iter(|| r.over(p.mask(), Gray8::new(100)))
    });
}

fn gray_over_512(c: &mut Criterion) {
    let mut p = make_plotter(512);
    p.fill(&make_fishy(), FillRule::NonZero);
    c.bench_function("gray_over_512", move |b| {
        let mut r = Raster::new(p.width(), p.height());
        b.iter(|| r.over(p.mask(), Gray8::new(100)))
    });
}

fn rgba_over_256(c: &mut Criterion) {
    let mut p = make_plotter(256);
    p.fill(&make_fishy(), FillRule::NonZero);
    c.bench_function("rgba_over_256", move |b| {
        let mut r = Raster::new(p.width(), p.height());
        b.iter(|| r.over(p.mask(), Rgba8::rgb(127, 96, 96)))
    });
}

fn rgba_over_512(c: &mut Criterion) {
    let mut p = make_plotter(512);
    p.fill(&make_fishy(), FillRule::NonZero);
    c.bench_function("rgba_over_512", move |b| {
        let mut r = Raster::new(p.width(), p.height());
        b.iter(|| r.over(p.mask(), Rgba8::rgb(127, 96, 96)))
    });
}

fn make_plotter(i: u32) -> Plotter {
    let mut p = Plotter::new(i, i);
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

criterion_group!(benches, fill_256, fill_512, stroke_256, stroke_512,
    gray_over_256, gray_over_512, rgba_over_256, rgba_over_512);
criterion_main!(benches);
