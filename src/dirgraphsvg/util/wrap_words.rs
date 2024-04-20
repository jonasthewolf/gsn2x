///
/// Wraps `s` at each `width`-th character adding `wrapstr` as a kind of line ending.
///
pub fn wrap_words(s: &str, width: u32, wrapstr: &str) -> String {
    let mut out = Vec::<String>::new();
    let mut cur_line = String::new();
    for word in s.split_ascii_whitespace() {
        if cur_line.chars().count() + word.chars().count() > width as usize {
            if !cur_line.is_empty() {
                // Relevant if cur_line.len = 0 and word.len > width
                out.push(cur_line);
            }
            cur_line = String::new();
        } else if !cur_line.is_empty() {
            cur_line.push(' ');
        }
        cur_line.push_str(word);
    }
    if !cur_line.is_empty() {
        out.push(cur_line);
    }
    out.join(wrapstr)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn simple() {
        let input = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.";
        let expected = concat!(
            "Lorem ipsum dolor sit amet, consetetur sadipscing\n",
            "elitr, sed diam nonumy eirmod tempor invidunt ut\n",
            "labore et dolore magna aliquyam erat, sed diam\n",
            "voluptua. At vero eos et accusam et justo duo\n",
            "dolores et ea rebum. Stet clita kasd gubergren, no\n",
            "sea takimata sanctus est Lorem ipsum dolor sit\n",
            "amet. Lorem ipsum dolor sit amet, consetetur\n",
            "sadipscing elitr, sed diam nonumy eirmod tempor\n",
            "invidunt ut labore et dolore magna aliquyam erat,\n",
            "sed diam voluptua. At vero eos et accusam et justo\n",
            "duo dolores et ea rebum. Stet clita kasd gubergren,\n",
            "no sea takimata sanctus est Lorem ipsum dolor sit\n",
            "amet."
        );
        let out = wrap_words(input, 50, "\n");
        assert_eq!(out, expected);
    }

    #[test]
    fn shorter() {
        let input = "Lorem ipsum dolor sit amet, consetetur";
        let expected = "Lorem ipsum dolor sit amet, consetetur".to_owned();
        let out = wrap_words(input, 50, "\n");
        assert_eq!(out, expected);
    }

    #[test]
    fn empty_line() {
        let input = " ";
        let expected = "".to_owned();
        let out = wrap_words(input, 50, "\n");
        assert_eq!(out, expected);
    }

    #[test]
    fn wrap_string() {
        let input = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt";
        let expected = concat!(
            "Lorem ipsum dolor sit amet, consetetur sadipscing<br align=\"left\"/>",
            "elitr, sed diam nonumy eirmod tempor invidunt",
        );
        let out = wrap_words(input, 50, "<br align=\"left\"/>");
        assert_eq!(out, expected);
    }

    #[test]
    fn with_newlines() {
        let input = "Lorem ipsum dolor sit amet,\nconsetetur sadipscing\nelitr, sed diam nonumy eirmod tempor invidunt";
        let expected = concat!(
            "Lorem ipsum dolor sit amet, consetetur\n",
            "sadipscing elitr, sed diam nonumy eirmod\n",
            "tempor invidunt",
        );
        let out = wrap_words(input, 45, "\n");
        assert_eq!(out, expected);
    }

    #[test]
    fn non_breaking_space() {
        let input = "aaaa bbbb\u{00a0}cccc";
        let expected = "aaaa\nbbbb\u{00a0}cccc";
        assert_eq!(wrap_words(input, 2, "\n"), expected);
    }

    #[test]
    fn even_shorter() {
        let input = "Devide";
        let out = wrap_words(input, 5, "\n");
        assert_eq!(input, out);
    }

    #[test]
    fn zero_no_space() {
        let input = "Devide";
        let out = wrap_words(input, 0, "\n");
        assert_eq!(input, out);
    }

    #[test]
    fn zero_some_space() {
        let input = "Devide and conquer";
        let expected = "Devide\nand\nconquer";
        let out = wrap_words(input, 0, "\n");
        assert_eq!(expected, out);
    }
}
