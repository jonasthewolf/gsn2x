use super::markdown::Text;

///
/// Default font family names and parameters on different operating systems
///
#[cfg(any(target_os = "windows", target_os = "macos"))]
mod font_constants {
    pub const FONT_FAMILY_NAME: &str = "Arial";
    pub const FONT_AVERAGE_ADVANCE_NORMAL_UPPER: f64 = 1459.655172413793 / 2048.0;
    pub const FONT_AVERAGE_ADVANCE_NORMAL_LOWER: f64 = 1043.8398791540785 / 2048.0;
    pub const FONT_AVERAGE_ADVANCE_BOLD_UPPER: f64 = 1490.2364532019703 / 2048.0;
    pub const FONT_AVERAGE_ADVANCE_BOLD_LOWER: f64 = 1133.2326283987916 / 2048.0;
    pub const FONT_HEIGHT: f64 = 1.14990234;
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod font_constants {
    pub const FONT_FAMILY_NAME: &str = "DejaVuSans";
    pub const FONT_AVERAGE_ADVANCE_NORMAL_UPPER: f64 = 1499.2061611374409 / 2048.0;
    pub const FONT_AVERAGE_ADVANCE_NORMAL_LOWER: f64 = 1213.687651331719 / 2048.0;
    pub const FONT_AVERAGE_ADVANCE_BOLD_UPPER: f64 = 1660.7261219792865 / 2048.0;
    pub const FONT_AVERAGE_ADVANCE_BOLD_LOWER: f64 = 1352.9566929133857 / 2048.0;
    pub const FONT_HEIGHT: f64 = 1.1640625;
}

///
/// All we need to know about a font
///
pub struct FontInfo {
    pub name: String,
    pub size: f64,
}

///
/// Default font with size 12
///
impl Default for FontInfo {
    fn default() -> Self {
        FontInfo {
            name: font_constants::FONT_FAMILY_NAME.to_owned(),
            size: 12.0,
        }
    }
}

///
/// Get a bounding box for a line of text with the given font info and bold setting.
/// Returns (width, height) of the bounding box.
///
pub fn text_line_bounding_box(font_info: &FontInfo, text: &[Text], bold: bool) -> (i32, i32) {
    let text = text
        .iter()
        .map(Into::into)
        .collect::<Vec<String>>()
        .join(" ");
    str_line_bounding_box(font_info, &text, bold)
}

///
/// Get the bounding box of `text` for the font described by `font_info` and `bold`.
/// `text` is a single line of text.
/// If the line is empty, font_info.size is returned as height
///
pub fn str_line_bounding_box(font_info: &FontInfo, text: &str, bold: bool) -> (i32, i32) {
    let uppercase_chars = text.chars().filter(|c| c.is_uppercase()).count() as i32;
    let other_chars = std::cmp::max(text.chars().count() as i32 - uppercase_chars, 0);
    let lower_advance = font_info.size
        * if bold {
            font_constants::FONT_AVERAGE_ADVANCE_BOLD_LOWER
        } else {
            font_constants::FONT_AVERAGE_ADVANCE_NORMAL_LOWER
        };
    let upper_advance = font_info.size
        * if bold {
            font_constants::FONT_AVERAGE_ADVANCE_BOLD_UPPER
        } else {
            font_constants::FONT_AVERAGE_ADVANCE_NORMAL_UPPER
        };
    (
        if text.chars().all(|c| c.is_whitespace()) {
            0
        } else {
            (lower_advance * other_chars as f64) as i32
                + (upper_advance * uppercase_chars as f64) as i32
        },
        (font_info.size * font_constants::FONT_HEIGHT) as i32,
    )
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn bounding_box() {
        let font_info = FontInfo::default();
        let (w, h) = str_line_bounding_box(&font_info, "text", false);
        println!("Width: {w} Height: {h}");
        assert!(w.abs_diff(20) <= 5);
        assert!(h.abs_diff(15) <= 5);
    }
}
