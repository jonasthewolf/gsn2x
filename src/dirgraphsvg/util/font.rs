use super::markdown::Text;

mod arial;
mod arial_bold;

pub trait Font {
    fn get_advance(c: char) -> f64;

    fn get_line_height() -> f64;

    // fn is_bold() -> bool;

    fn get_name() -> &'static str;
}

///
/// Default font family names and parameters on different operating systems
///
#[cfg(any(target_os = "windows", target_os = "macos"))]
type NormalFont = arial::Arial;
#[cfg(any(target_os = "windows", target_os = "macos"))]
type BoldFont = arial_bold::Arial;
// #[cfg(not(any(target_os = "windows", target_os = "macos")))]
// type NormalFont = dejavusans::DejavuSans;
// #[cfg(any(target_os = "windows", target_os = "macos"))]
// type BoldFont = dejavusans_bold::DejavuSans;

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
            name: NormalFont::get_name().to_owned(),
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
    (
        text.chars()
            .map(|c| {
                font_info.size
                    * if bold {
                        BoldFont::get_advance(c)
                    } else {
                        NormalFont::get_advance(c)
                    }
            })
            .sum::<f64>() as i32,
        (font_info.size
            * if bold {
                BoldFont::get_line_height()
            } else {
                NormalFont::get_line_height()
            }) as i32,
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
