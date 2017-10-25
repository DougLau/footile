#![feature(test)]
extern crate test;
extern crate footile;

use footile::*;
use test::Bencher;

#[bench]
fn fill_fishy(b: &mut Bencher) {
    b.iter(|| fill_fishy2())
}

fn fill_fishy2() {
    make_fishy().fill(FillRule::NonZero);
}

#[bench]
fn stroke_fishy(b: &mut Bencher) {
    b.iter(|| stroke_fishy2())
}

fn stroke_fishy2() {
    make_fishy().stroke();
}

fn make_fishy() -> Plotter {
    let fish = PathBuilder::new().relative()
                           .move_to(112f32, 16f32)
                           .line_to(-48f32, 32f32)
                           .cubic_to(-64f32, -48f32, -64f32, 80f32, 0f32, 32f32)
                           .line_to(48f32, 32f32)
                           .line_to(-32f32, -48f32)
                           .close().build();
    let mut p = Plotter::new(256, 256);
    p.set_transform(Transform::new_scale(2f32, 2f32));
    p.add_path(fish);
    p
}
