use anyhow::{Error, Result};
use font_kit::source::SystemSource;
use rusttype::Font;

#[cfg(target_os = "windows")]
pub static DEFAULT_FONT_FAMILY_NAME: &str = "ArialMT";
#[cfg(target_os = "macos")]
pub static DEFAULT_FONT_FAMILY_NAME: &str = "AppleSystemUIFont";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub static DEFAULT_FONT_FAMILY_NAME: &str = "DejaVuSans";

pub fn get_default_font() -> Result<rusttype::Font<'static>> {
    get_font(DEFAULT_FONT_FAMILY_NAME)
}

pub fn get_font(font_name: &str) -> Result<rusttype::Font<'static>> {
    let f = SystemSource::new()
        .select_by_postscript_name(font_name)?
        .load()?;
    let fd = f.copy_font_data().unwrap();
    let font: Option<Font<'static>> = Font::try_from_vec(fd.to_vec());
    font.ok_or_else(|| Error::msg("Font not found"))
}

pub fn text_bounding_box(font: &Font, text: &str, size: f32) -> (u32, u32) {
    let scale = rusttype::Scale::uniform(size);
    let width = font
        .layout(text, scale, rusttype::point(0.0, 0.0))
        .last()
        .map(|g| g.pixel_bounding_box().unwrap().max.x)
        // .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .unwrap_or(0);

    let v_metrics = font.v_metrics(scale);
    let height = (v_metrics.ascent - v_metrics.descent).ceil() + v_metrics.line_gap;

    ((width as f32 * 1.1) as u32, (height as f32 * 1.1) as u32) // Do magic: rusttype seems to be roughly 11 percent too small
}
