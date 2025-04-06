#![allow(unused)]

use pix::Raster;
use pix::el::Pixel;
use pix::gray::SGray8;
use pix::matte::Matte8;
use png_pong::Encoder;
use std::fs::File;
use std::io;

/// Write a `Raster` to a file.
pub fn write<P>(raster: &Raster<P>, filename: &str) -> io::Result<()>
where
    P: Pixel,
{
    let mut file = File::create(filename)?;
    Encoder::new(&mut file).into_step_enc().still(raster);
    Ok(())
}

/// Write a `Raster<Matte8>` to a grayscale file.
pub fn write_matte(raster: &Raster<Matte8>, filename: &str) -> io::Result<()> {
    let pix = raster.as_u8_slice();
    let raster =
        Raster::<SGray8>::with_u8_buffer(raster.width(), raster.height(), pix);
    write(&raster, filename)
}
