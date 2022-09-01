use anyhow::{Error, Result};
use font_kit::source::SystemSource;
use glyph_brush_layout::{
    ab_glyph::{Font, FontVec},
    FontId, GlyphPositioner, Layout, SectionGeometry, SectionText,
};

#[cfg(target_os = "windows")]
pub static DEFAULT_FONT_FAMILY_NAME: &str = "ArialMT";
#[cfg(target_os = "macos")]
pub static DEFAULT_FONT_FAMILY_NAME: &str = "AppleSystemUIFont";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub static DEFAULT_FONT_FAMILY_NAME: &str = "DejaVuSans";

pub fn get_default_font() -> Result<FontVec> {
    get_font(DEFAULT_FONT_FAMILY_NAME)
}

pub fn get_font(font_name: &str) -> Result<FontVec> {
    let f = SystemSource::new()
        .select_by_postscript_name(font_name)?
        .load()?;
    let fd = f.copy_font_data().unwrap();
    FontVec::try_from_vec(fd.to_vec()).map_err(Error::from)
}

pub fn text_bounding_box(font: &FontVec, text: &str, size: f32) -> (i32, i32) {
    let scale = font.pt_to_px_scale(size).unwrap();
    Layout::default_single_line()
        .calculate_glyphs(
            &[&font],
            &SectionGeometry::default(),
            &[SectionText {
                text,
                scale,
                font_id: FontId(0),
            }],
        )
        .last()
        .map(|g| (g.glyph.position.x as i32, g.glyph.position.y as i32))
        // .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .unwrap_or((0, 0))

    // let v_metrics = font.v_metrics(scale);
    // let height = (v_metrics.ascent - v_metrics.descent).ceil() + v_metrics.line_gap;

    // ((width as f32 * 1.1) as i32, (height as f32 * 1.1) as i32) // Do magic: rusttype seems to be roughly 11 percent too small
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn default_font_exists() {
        assert!(get_default_font().is_ok());
    }

    #[test]
    fn non_existing_font() {
        assert!(get_font("ahopefullycrazyenoughfontnamethatdoesnotexistanywhere").is_err());
    }

    #[test]
    fn bounding_box() {
        let font = get_default_font().unwrap();
        let (w, h) = text_bounding_box(&font, "text", 12.0);
        assert!(w.abs_diff(19) < 5);
        assert!(h.abs_diff(14) < 5);
    }
}
