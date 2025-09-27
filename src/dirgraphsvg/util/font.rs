use super::markdown::Text;

mod metrics_bold;
mod metrics_normal;

use metrics_bold::BoldFont;
use metrics_normal::NormalFont;

pub trait Font {
    fn get_advance(c: char) -> f64;

    fn get_line_height() -> f64;
}

pub const FONT_SIZE: f64 = 12.0;
pub const FONT_FAMILY: &str = "Liberation Sans, Arial, sans-serif";

///
/// Get a bounding box for a line of text with the given font info and bold setting.
/// Returns (width, height) of the bounding box.
///
pub fn text_line_bounding_box(text: &[Text], bold: bool) -> (i32, i32) {
    let text = text
        .iter()
        .map(Into::into)
        .collect::<Vec<String>>()
        .join(" ");
    str_line_bounding_box(&text, bold)
}

///
/// Get the bounding box of `text` for the font described by `font_info` and `bold`.
/// `text` is a single line of text.
/// If the line is empty, font_info.size is returned as height
///
pub fn str_line_bounding_box(text: &str, bold: bool) -> (i32, i32) {
    (
        text.chars()
            .map(|c| {
                FONT_SIZE
                    * if bold {
                        BoldFont::get_advance(c)
                    } else {
                        NormalFont::get_advance(c)
                    }
            })
            .sum::<f64>() as i32,
        (FONT_SIZE
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
        let (w, h) = str_line_bounding_box("text", false);
        println!("Width: {w} Height: {h}");
        assert!(w.abs_diff(20) <= 5);
        assert!(h.abs_diff(15) <= 5);
    }
}
