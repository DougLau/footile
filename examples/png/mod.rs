#![allow(unused)]

use pix::el::Pixel;
use pix::gray::SGray8;
use pix::matte::Matte8;
use pix::Raster;
use png_pong::Format;
use std::fs;
use std::io;

/// Write a `Raster` to a file.
pub fn write<P>(raster: &Raster<P>, filename: &str) -> io::Result<()>
where
    P: Pixel + Format
{
    let mut out_data = vec![];
    png_pong::FrameEncoder::new(&mut out_data).still(raster)?;
    fs::write(filename, out_data)
}

/// Write a `Raster<Matte8>` to a grayscale file.
pub fn write_matte(raster: &Raster<Matte8>, filename: &str) -> io::Result<()> {
    let pix = raster.as_u8_slice();
    let raster = Raster::<SGray8>::with_u8_buffer(
        raster.width(),
        raster.height(),
        pix,
    );
    write(&raster, filename)
}
