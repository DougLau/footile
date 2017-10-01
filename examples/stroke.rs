// stroke.rs        Example stroking a figure
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;
use footile::geom::Vec3;
use footile::mask::Mask;
use footile::fig::Fig;
use footile::fig::FillRule;

fn main() {
    let mut m = Mask::new(64, 64);
    let mut b = Mask::new(64, 1);
    let mut f = Fig::new();
    let mut s = Fig::new();
    f.add_point(Vec3::new(16f32, 48f32, 5f32));
    f.add_point(Vec3::new(48f32, 48f32, 5f32));
    f.add_point(Vec3::new(32f32, 16f32, 5f32));
    f.close(true);
    f.stroke(&mut s);
    s.fill(&mut m, &mut b, FillRule::NonZero);
    m.write_pgm("./stroke.pgm").unwrap();
}
