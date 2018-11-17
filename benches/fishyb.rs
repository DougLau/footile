#[macro_use]
extern crate criterion;
extern crate footile;

use footile::*;
use criterion::Criterion;

fn fill_fishy_256(c: &mut Criterion) {
    c.bench_function("fill_fishy_256", |b| b.iter(|| fill_fishy(256)));
}

fn fill_fishy_512(c: &mut Criterion) {
    c.bench_function("fill_fishy_512", |b| b.iter(|| fill_fishy(512)));
}

fn fill_fishy(i: u32) {
    make_plotter(i).fill(&make_fishy(), FillRule::NonZero);
}

fn compose_fishy_256(c: &mut Criterion) {
    let mut p = make_plotter(256);
    p.fill(&make_fishy(), FillRule::NonZero);
    c.bench_function("compose_fishy_256", move |b| {
        let mut r = Raster::new(p.width(), p.height());
        let m = p.mask();
        b.iter(|| r.over(m, Rgba8::rgb(127, 96, 96)))
    });
}

fn compose_fishy_512(c: &mut Criterion) {
    let mut p = make_plotter(512);
    p.fill(&make_fishy(), FillRule::NonZero);
    c.bench_function("compose_fishy_512", move |b| {
        let mut r = Raster::new(p.width(), p.height());
        let m = p.mask();
        b.iter(|| r.over(m, Rgba8::rgb(127, 96, 96)))
    });
}

fn stroke_fishy(c: &mut Criterion) {
    c.bench_function("stroke_fishy", |b| b.iter(|| stroke_fishy2()));
}

fn stroke_fishy2() {
    make_plotter(256).stroke(&make_fishy());
}

fn make_plotter(i: u32) -> Plotter<Rgba8> {
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

criterion_group!(benches, fill_fishy_256, fill_fishy_512, compose_fishy_256,
    compose_fishy_512, stroke_fishy);
criterion_main!(benches);
