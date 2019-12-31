#![allow(unused)]

use pix::{Raster, RasterBuilder};
use std::fs;
use std::io;
use png_pong::Format;

/// Write a `Raster` to a file.
pub fn write<F: Format>(raster: &Raster<F>, filename: &str) -> io::Result<()> {
    let mut out_data = vec![];
    png_pong::EncoderBuilder::new()
        .encode_rasters(&mut out_data)
        .add_frame(raster, 0)
        .expect("Failed to add frame");
    fs::write(filename, out_data)
}

/// Write a `Raster<Mask8>` to a grayscale file.
pub fn write_mask(raster: &Raster<pix::Mask8>, filename: &str) -> io::Result<()> {
    let pix = raster.as_u8_slice();
    let raster = RasterBuilder::<pix::Gray8>::new()
        .with_u8_buffer(raster.width(), raster.height(), pix);
    write(&raster, filename)
}
