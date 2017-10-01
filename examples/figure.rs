// figure.rs        Example using a figure
//
// Copyright (c) 2017  Douglas P Lau
//
extern crate footile;
use std::fs::File;
use std::io::BufWriter;
use footile::geom::Vec3;
use footile::mask::Mask;
use footile::fig::Fig;
use footile::fig::FillRule;

fn main() {
    let mut m = Mask::new(64, 64);
    let mut b = Mask::new(64, 1);
    let mut f = Fig::new();
    f.add_point(Vec3::new(4f32, 4f32, 1f32));
    f.add_point(Vec3::new(32f32, 16f32, 1f32));
    f.add_point(Vec3::new(60f32, 4f32, 1f32));
    f.add_point(Vec3::new(48f32, 32f32, 1f32));
    f.add_point(Vec3::new(60f32, 60f32, 1f32));
    f.add_point(Vec3::new(32f32, 56f32, 1f32));
    f.add_point(Vec3::new(4f32, 60f32, 1f32));
    f.add_point(Vec3::new(16f32, 32f32, 1f32));
    f.close(true);
    f.fill(&mut m, &mut b, FillRule::NonZero);
    let fl = File::create("./fig.pgm").unwrap();
    let mut bw = BufWriter::new(fl);
    let _ = m.write_pgm(bw.get_mut());
}
