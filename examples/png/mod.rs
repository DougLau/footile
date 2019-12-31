use pix::{Gray8, Mask8, Raster, Rgba8};
use png::HasParameters;
use std::fs::File;
use std::io;

pub fn write_mask(raster: &Raster<Mask8>, filename: &str) -> io::Result<()> {
    let fl = File::create(filename)?;
    let ref mut bw = io::BufWriter::new(fl);
    let mut enc = png::Encoder::new(bw, raster.width(), raster.height());
    enc.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
    let mut writer = enc.write_header()?;
    let pix = raster.as_u8_slice();
    writer.write_image_data(pix)?;
    Ok(())
}

pub fn write_gray(raster: &Raster<Gray8>, filename: &str) -> io::Result<()> {
    let fl = File::create(filename)?;
    let ref mut bw = io::BufWriter::new(fl);
    let mut enc = png::Encoder::new(bw, raster.width(), raster.height());
    enc.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
    let mut writer = enc.write_header()?;
    let pix = raster.as_u8_slice();
    writer.write_image_data(pix)?;
    Ok(())
}

pub fn write_rgba(raster: &Raster<Rgba8>, filename: &str) -> io::Result<()> {
    let fl = File::create(filename)?;
    let ref mut bw = io::BufWriter::new(fl);
    let mut enc = png::Encoder::new(bw, raster.width(), raster.height());
    enc.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = enc.write_header()?;
    let pix = raster.as_u8_slice();
    writer.write_image_data(pix)?;
    Ok(())
}
