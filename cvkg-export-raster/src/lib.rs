use image::{ImageFormat, RgbaImage};
use std::io::Cursor;

/// A single captured frame, RGBA8, tightly packed.
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Encode one frame as a PNG byte buffer.
pub fn encode_png(frame: &CapturedFrame) -> Result<Vec<u8>, String> {
    let img = RgbaImage::from_raw(frame.width, frame.height, frame.rgba.clone())
        .ok_or("frame buffer size does not match width*height*4")?;
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

/// Encode a sequence of frames as an animated GIF.
pub fn encode_gif(frames: &[CapturedFrame], fps: u16) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    {
        let (w, h) = frames
            .first()
            .map(|f| (f.width as u16, f.height as u16))
            .ok_or("no frames to encode")?;
        let mut encoder = gif::Encoder::new(&mut out, w, h, &[]).map_err(|e| e.to_string())?;
        let delay_cs = 100 / fps.max(1);
        for frame in frames {
            let mut gif_frame = gif::Frame::from_rgba_speed(
                frame.width as u16,
                frame.height as u16,
                &mut frame.rgba.clone(),
                10,
            );
            gif_frame.delay = delay_cs;
            encoder.write_frame(&gif_frame).map_err(|e| e.to_string())?;
        }
    }
    Ok(out)
}
