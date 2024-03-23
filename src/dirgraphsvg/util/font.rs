use anyhow::{anyhow, Error, Result};
use font_loader::system_fonts;
use glyph_brush_layout::{
    ab_glyph::{FontVec, PxScale},
    FontId, GlyphPositioner, Layout, SectionGeometry, SectionText,
};

///
/// Default font family names on different operating systems
///
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub static DEFAULT_FONT_FAMILY_NAME: &str = "Arial";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub static DEFAULT_FONT_FAMILY_NAME: &str = "DejaVuSans";

///
/// All we need to know about a font
///
pub struct FontInfo {
    font: FontVec,
    font_bold: FontVec,
    font_italic: FontVec,
    pub name: String,
    pub size: f32,
}

///
/// Default font with size 12
///
impl Default for FontInfo {
    fn default() -> Self {
        FontInfo {
            font: get_default_font(false, false).unwrap(),
            font_bold: get_default_font(true, false).unwrap(),
            font_italic: get_default_font(false, true).unwrap(),
            name: DEFAULT_FONT_FAMILY_NAME.to_owned(),
            size: 12.0,
        }
    }
}

///
/// Get the default font as a byte vector
///
pub fn get_default_font(bold: bool, italic: bool) -> Result<FontVec> {
    get_font(DEFAULT_FONT_FAMILY_NAME, bold, italic)
}

///
/// Get a font as a byte vector
///
fn get_font(font_name: &str, bold: bool, italic: bool) -> Result<FontVec> {
    let mut props = system_fonts::FontPropertyBuilder::new();
    props = props.family(font_name);
    if bold {
        props = props.bold();
    }
    if italic {
        props = props.italic();
    }
    let prop = props.build();
    let (fd, _) =
        system_fonts::get(&prop).ok_or_else(|| anyhow!("Font {font_name} is not found."))?;
    FontVec::try_from_vec(fd.to_vec()).map_err(Error::from)
}

///
/// Get the bounding box of `text` for the font described by `font_info` and `bold`
/// If the line is empty, font_info.size is returned as height
///
pub fn text_bounding_box(font_info: &FontInfo, text: &str, bold: bool) -> (i32, i32) {
    if text.chars().filter(|c| !c.is_whitespace()).count() == 0 {
        (0, font_info.size as i32)
    } else {
        let kern = if bold {
            text.chars().count() as f64 * 1.2
        } else {
            text.chars().count() as f64 * 1.0
        };
        let line_gap = 5.0;
        let font_id = usize::from(bold);
        Layout::default_single_line()
            .calculate_glyphs(
                &[
                    &font_info.font,
                    &font_info.font_bold,
                    &font_info.font_italic,
                ],
                &SectionGeometry::default(),
                &[SectionText {
                    text,
                    scale: PxScale::from(font_info.size),
                    font_id: FontId(font_id),
                }],
            )
            .last()
            .map(|g| {
                (
                    g.glyph.position.x as f64 + kern,
                    g.glyph.position.y as f64 + line_gap,
                )
            })
            .map(|(x, y)| (x as i32, y as i32))
            .unwrap_or((0, font_info.size as i32))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn default_font_exists() {
        assert!(get_default_font(false, false).is_ok());
    }

    // We cannot test for non-existing fonts, since Linux will use a default font anyway.

    #[test]
    fn bounding_box() {
        let font_info = FontInfo::default();
        let (w, h) = text_bounding_box(&font_info, "text", false);
        println!("Width: {w} Height: {h}");
        assert!(w.abs_diff(20) <= 5);
        assert!(h.abs_diff(15) <= 5);
    }
}
