pub mod font;
pub mod point2d;
pub mod wordwrap;

///
///
/// Escape string to prevent misrendering if special characters are used.
///
///
pub fn escape_text(input: &str) -> String {
    input
        .replace('.', "_")
        .replace('-', "_")
        .replace(' ', "_")
        .replace('/', "_")
        .replace('\\', "_")
        .replace(':', "_")
        .replace('\'', "")
        .replace('"', "")
}
