pub mod font;
pub mod point2d;
pub mod wrap_words;

///
/// Escape string to prevent misrendering if special characters are used.
///
pub fn escape_text(input: &str) -> String {
    input.replace(['.', '-', ' ', '/', '\\', ':', '\'', '"', '~'], "_")
}

///
/// Create node identifiers for the rendered SVG.
///
pub fn escape_node_id(id: &str) -> String {
    format!("node_{}", escape_text(id))
}

///
/// Escape characters that are invalid in XML (SVG)
///
/// To prevent double escaping, undo escaping that is already in the string.
///
pub fn escape_url(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace('&', "&amp;")
        .replace("&lt;", "<")
        .replace('<', "&lt;")
        .replace("&gt;", "&")
        .replace('>', "&gt;")
        .replace("&apos;", "\'")
        .replace('\'', "&apos;")
        .replace("&quot;", "\"")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod test {
    use super::escape_text;

    #[test]
    fn escape_test() {
        assert_eq!(escape_text(".- /\\:\'\"~"), "_________");
    }
}
