// letter.rs     Example plotting the letter C
use footile::{FillRule, PathBuilder, Plotter};
use pix::matte::Matte8;
use pix::Raster;

mod png;

fn main() -> Result<(), std::io::Error> {
    let pb = PathBuilder::default().absolute();
    let path = pb
        .move_to(88.61539, 64.895096)
        .quad_to(62.433567, 64.895096, 47.88811, 81.79021)
        .quad_to(33.342655, 98.57342, 33.342655, 127.88811)
        .quad_to(33.342655, 156.86713, 48.44755, 174.54544)
        .quad_to(63.664333, 192.11188, 89.51049, 192.11188)
        .quad_to(122.62937, 192.11188, 139.3007, 159.32866)
        .line_to(156.75525, 168.05594)
        .quad_to(147.02098, 188.41957, 129.34265, 199.04895)
        .quad_to(111.77622, 209.67831, 88.503494, 209.67831)
        .quad_to(64.671326, 209.67831, 47.21678, 199.83215)
        .quad_to(29.874126, 189.87411, 20.6993, 171.52448)
        .quad_to(11.636364, 153.06293, 11.636364, 127.88811)
        .quad_to(11.636364, 90.18181, 32.0, 68.81119)
        .quad_to(52.363636, 47.44055, 88.39161, 47.44055)
        .quad_to(113.56643, 47.44055, 130.46153, 57.286713)
        .quad_to(147.35664, 67.13286, 155.3007, 86.4895)
        .line_to(135.04895, 93.20279)
        .quad_to(129.56644, 79.44055, 117.37063, 72.16783)
        .quad_to(105.28671, 64.895096, 88.61539, 64.895096)
        .build();
    let r = Raster::with_clear(165, 256);
    let mut p = Plotter::new(r);
    p.fill(FillRule::NonZero, &path, Matte8::new(255));
    png::write_matte(&p.raster(), "./letter.png")
}
