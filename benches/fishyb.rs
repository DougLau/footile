#![feature(test)]
extern crate test;
extern crate footile;

use footile::{FillRule, Plotter, PlotterBuilder};
use test::Bencher;

#[bench]
fn fill_fishy(b: &mut Bencher) {
    b.iter(|| fill_fishy2())
}

fn fill_fishy2() {
    make_fishy().fill(FillRule::EvenOdd);
}

#[bench]
fn stroke_fishy(b: &mut Bencher) {
    b.iter(|| stroke_fishy2())
}

fn stroke_fishy2() {
    make_fishy().stroke();
}

fn make_fishy() -> Plotter {
    let mut p = PlotterBuilder::new().width(256).height(256).user_width(128).user_height(128).build();
    p.move_to(112f32, 16f32)
     .line_to(-48f32, 32f32)
     .cubic_to(-64f32, -48f32, -64f32, 80f32, 0f32, 32f32)
     .line_to(48f32, 32f32)
     .line_to(-32f32, -48f32)
     .close();
    p
}
