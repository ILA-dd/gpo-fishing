use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, ImageFormat, ImageOrder};

use super::x11::X11Connection;
use crate::core::platform::{Capture, PlatformError, Result};
use crate::core::types::{Frame, PxRect};

/// Captures arbitrary desktop regions from the X11 root window.
///
/// Unlike a window-only capturer, this keeps the coordinates used by the rest
/// of the application identical to Windows: all regions are absolute desktop
/// pixels. This is important for regions saved relative to the Roblox client.
pub struct X11Capture {
    x11: Option<X11Connection>,
}

#[derive(Debug, Clone, Copy)]
struct PixelFormat {
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    bits_per_pixel: u8,
    scanline_pad: u8,
    image_order: ImageOrder,
}

impl X11Capture {
    pub fn new() -> Self {
        match X11Connection::open() {
            Ok(x11) => Self { x11: Some(x11) },
            Err(err) => {
                tracing::warn!("X11 capture is unavailable: {err}");
                Self { x11: None }
            }
        }
    }
}

impl Default for X11Capture {
    fn default() -> Self {
        Self::new()
    }
}

impl Capture for X11Capture {
    fn grab(&self, rect: PxRect) -> Result<Frame> {
        if rect.is_empty() {
            return Err(PlatformError::Capture("empty rect".into()));
        }
        let x11 = self
            .x11
            .as_ref()
            .ok_or_else(|| PlatformError::Capture("could not connect to the X11 display".into()))?;
        if rect.x < 0
            || rect.y < 0
            || rect.x > i16::MAX as i32
            || rect.y > i16::MAX as i32
            || rect.w > u16::MAX as i32
            || rect.h > u16::MAX as i32
        {
            return Err(PlatformError::Capture(
                "region is outside the X11 root window".into(),
            ));
        }

        let connection = x11.connection.lock();
        let geometry = connection
            .get_geometry(x11.root)
            .map_err(x11_error)?
            .reply()
            .map_err(x11_error)?;
        if rect.right() > i32::from(geometry.width) || rect.bottom() > i32::from(geometry.height) {
            return Err(PlatformError::Capture(
                "region is outside the X11 root window".into(),
            ));
        }
        let format = pixel_format(&*connection, x11.root, geometry.depth)?;
        let image = connection
            .get_image(
                ImageFormat::Z_PIXMAP,
                x11.root,
                rect.x as i16,
                rect.y as i16,
                rect.w as u16,
                rect.h as u16,
                u32::MAX,
            )
            .map_err(x11_error)?
            .reply()
            .map_err(x11_error)?;

        decode_rgba(&image.data, rect.w as usize, rect.h as usize, format)
            .map(|rgba| Frame::new(rect.w as usize, rect.h as usize, rgba))
            .map_err(PlatformError::Capture)
    }
}

fn x11_error(err: impl std::fmt::Display) -> PlatformError {
    PlatformError::Capture(err.to_string())
}

fn pixel_format(connection: &impl Connection, root: u32, depth: u8) -> Result<PixelFormat> {
    let setup = connection.setup();
    let screen = setup
        .roots
        .iter()
        .find(|screen| screen.root == root)
        .ok_or_else(|| PlatformError::Capture("X11 root screen is unavailable".into()))?;
    let root_visual = screen.root_visual;
    let visual = screen
        .allowed_depths
        .iter()
        .flat_map(|allowed| allowed.visuals.iter())
        .find(|visual| visual.visual_id == root_visual)
        .ok_or_else(|| PlatformError::Capture("X11 visual format is unsupported".into()))?;
    let pixmap = setup
        .pixmap_formats
        .iter()
        .find(|format| format.depth == depth)
        .ok_or_else(|| {
            PlatformError::Capture(format!("X11 has no pixmap format for depth {depth}"))
        })?;
    Ok(PixelFormat {
        red_mask: visual.red_mask,
        green_mask: visual.green_mask,
        blue_mask: visual.blue_mask,
        bits_per_pixel: pixmap.bits_per_pixel,
        scanline_pad: pixmap.scanline_pad,
        image_order: setup.image_byte_order,
    })
}

fn decode_rgba(
    data: &[u8],
    width: usize,
    height: usize,
    format: PixelFormat,
) -> std::result::Result<Vec<u8>, String> {
    let bytes_per_pixel = usize::from(format.bits_per_pixel).div_ceil(8);
    if !(1..=4).contains(&bytes_per_pixel) || format.scanline_pad == 0 {
        return Err(format!(
            "unsupported X11 pixel layout: {} bits per pixel",
            format.bits_per_pixel
        ));
    }
    let bits_per_line = width
        .checked_mul(usize::from(format.bits_per_pixel))
        .ok_or_else(|| "capture dimensions overflow".to_string())?;
    let padded_bits =
        bits_per_line.div_ceil(usize::from(format.scanline_pad)) * usize::from(format.scanline_pad);
    let bytes_per_line = padded_bits.div_ceil(8);
    let required = bytes_per_line
        .checked_mul(height)
        .ok_or_else(|| "capture dimensions overflow".to_string())?;
    if data.len() < required {
        return Err("X11 returned a truncated image".into());
    }

    let mut rgba = Vec::with_capacity(width * height * 4);
    for row in data[..required].chunks_exact(bytes_per_line) {
        for pixel in row.chunks_exact(bytes_per_pixel).take(width) {
            let value = match format.image_order {
                ImageOrder::LSB_FIRST => {
                    pixel.iter().enumerate().fold(0u32, |value, (shift, byte)| {
                        value | (u32::from(*byte) << (shift * 8))
                    })
                }
                ImageOrder::MSB_FIRST => pixel
                    .iter()
                    .fold(0u32, |value, byte| (value << 8) | u32::from(*byte)),
                _ => return Err("unsupported X11 image byte order".into()),
            };
            rgba.push(scale_channel(value, format.red_mask));
            rgba.push(scale_channel(value, format.green_mask));
            rgba.push(scale_channel(value, format.blue_mask));
            rgba.push(255);
        }
    }
    Ok(rgba)
}

fn scale_channel(pixel: u32, mask: u32) -> u8 {
    if mask == 0 {
        return 0;
    }
    let shift = mask.trailing_zeros();
    let max = mask >> shift;
    (((pixel & mask) >> shift) * 255 / max.max(1)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_little_endian_bgrx() {
        let format = PixelFormat {
            red_mask: 0x00ff_0000,
            green_mask: 0x0000_ff00,
            blue_mask: 0x0000_00ff,
            bits_per_pixel: 32,
            scanline_pad: 32,
            image_order: ImageOrder::LSB_FIRST,
        };
        assert_eq!(
            decode_rgba(&[0x33, 0x22, 0x11, 0x00], 1, 1, format).unwrap(),
            [0x11, 0x22, 0x33, 255]
        );
    }
}
