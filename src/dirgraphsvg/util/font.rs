use anyhow::{Error, Result};
use font_kit::{
    family_name::FamilyName,
    properties::{Properties, Stretch, Style, Weight},
    source::SystemSource,
};
use glyph_brush_layout::{
    ab_glyph::{FontVec, PxScale},
    FontId, GlyphPositioner, Layout, SectionGeometry, SectionText,
};

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub static DEFAULT_FONT_FAMILY_NAME: &str = "Arial";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub static DEFAULT_FONT_FAMILY_NAME: &str = "DejaVuSans";

pub struct FontInfo {
    font: FontVec,
    font_bold: FontVec,
    font_italic: FontVec,
    pub name: String,
    pub size: f32,
}

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

pub fn get_default_font(bold: bool, italic: bool) -> Result<FontVec> {
    get_font(DEFAULT_FONT_FAMILY_NAME, bold, italic)
}

pub fn get_font(font_name: &str, bold: bool, italic: bool) -> Result<FontVec> {
    let mut props = Properties::new();
    props = *props.stretch(Stretch::NORMAL);
    if bold {
        props = *props.weight(Weight::BOLD);
    }
    if italic {
        props = *props.style(Style::Italic);
    }
    let f = SystemSource::new()
        .select_best_match(&[FamilyName::Title(font_name.to_owned())], &props)?
        .load()?;
    let fd = f.copy_font_data().unwrap();
    FontVec::try_from_vec(fd.to_vec()).map_err(Error::from)
}

pub fn text_bounding_box(font_info: &FontInfo, text: &str, bold: bool) -> (i32, i32) {
    // let scale = font_info.font.pt_to_px_scale(font_info.size).unwrap();
    let kern = if bold {
        text.chars().count() as i32 * 4
    } else {
        (text.chars().count() as f32 * 1.5) as i32
    };
    let line_gap = 5;
    let font_id = if bold { 1 } else { 0 };
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
                g.glyph.position.x as i32 + kern,
                g.glyph.position.y as i32 + line_gap,
            )
        })
        .unwrap_or((0, 0))
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn default_font_exists() {
        assert!(get_default_font(false, false).is_ok());
    }

    #[test]
    fn non_existing_font() {
        assert!(get_font(
            "ahopefullycrazyenoughfontnamethatdoesnotexistanywhere",
            false,
            false
        )
        .is_err());
    }

    #[test]
    fn bounding_box() {
        let font_info = FontInfo::default();
        let (w, h) = dbg!(text_bounding_box(&font_info, "text", false));
        assert!(w.abs_diff(20) <= 5);
        assert!(h.abs_diff(15) <= 5);
    }
}
