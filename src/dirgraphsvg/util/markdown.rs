use crate::file_utils::get_url_identifiers;

#[derive(Debug, PartialEq, Eq)]
pub struct Link<'a> {
    pub href: &'a str,
    pub text: Vec<TextType<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TextType<'a> {
    Normal(&'a str),
    Italic(&'a str),
    Bold(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Text<'a> {
    String(TextType<'a>),
    Link(Link<'a>),
}

///
/// Parse text and search for markdown syntax links
///
pub fn parse_markdown_links(input: &str) -> Vec<Text> {
    let mut output: Vec<Text> = Vec::new();
    let mut indices = get_url_identifiers()
        .iter()
        .flat_map(|url_id| input.match_indices(url_id))
        .collect::<Vec<_>>();
    indices.sort();
    let mut last_index = 0;

    for (index, _) in indices {
        let mut next_index = index;
        let start_link = index;
        let mut link_text: Option<&str> = None;
        let mut end_link = input[index..]
            .find(char::is_whitespace)
            .unwrap_or(input.len() - index)
            + index;
        let mut skip_bracket = 0;
        if index > 0 && &input[index - 1..index] == "(" {
            if let Some(end_link_p) = input[index..].find(')') {
                end_link = end_link_p + index;
                if index > 1 && &input[index - 2..index - 1] == "]" {
                    if let Some(start_text) = input[..index - 2].rfind('[') {
                        // Link with separate text
                        let end_text = index - 2;
                        link_text = Some(&input[start_text + 1..end_text]);
                        next_index = start_text;
                        skip_bracket = 1;
                    }
                }
            }
        }
        // Add text before the current link (and after the last one) as text
        if next_index > last_index {
            output.append(
                &mut parse_markdown_text(&input[last_index..next_index])
                    .into_iter()
                    .map(Text::String)
                    .collect(),
            );
        }
        // Add the link itself
        output.push(Text::Link(Link {
            href: &input[start_link..end_link],
            text: if let Some(link_text) = link_text {
                parse_markdown_text(link_text)
            } else {
                vec![]
            },
        }));
        last_index = end_link + skip_bracket;
    }
    if last_index < input.len() {
        output.append(
            &mut parse_markdown_text(&input[last_index..])
                .into_iter()
                .map(Text::String)
                .collect(),
        );
    }

    output
}

///
/// Parse text for formatting markers
/// " _" <- Start of italic text
/// "_ " <- End of italic text
/// " *" <- Start of bold text
/// "* " <- End of bold text
///
pub fn parse_markdown_text(input: &str) -> Vec<TextType> {
    let mut output: Vec<TextType> = Vec::new();
    let mut indices = ["*", "_"]
        .iter()
        .flat_map(|url_id| input.match_indices(url_id))
        .collect::<Vec<_>>();
    indices.sort();
    let mut indices_iter = indices.into_iter();
    let mut last_index = 0;
    let mut in_emph_char = None;
    let mut start_emph = 0;

    while let Some((cur_index, emph_char)) = indices_iter.next() {
        if let Some(open_char) = in_emph_char {
            if emph_char == open_char
                && (cur_index == input.len() - 1
                    || (cur_index < input.len() - 1
                        && input[cur_index + 1..cur_index + 2]
                            .find(is_separator)
                            .is_some()))
            {
                let end_emph = cur_index;

                // Add the non-emphasized part before the current match
                if start_emph - 1 > last_index {
                    output.push(TextType::Normal(&input[last_index..start_emph - 1]));
                }

                // Add the emphasized part itself
                output.push(match emph_char {
                    "*" => TextType::Bold(&input[start_emph..end_emph]),
                    "_" => TextType::Italic(&input[start_emph..end_emph]),
                    _ => unreachable!(),
                });
                in_emph_char = None;
                last_index = cur_index + 1;
            }
            // else skip, if non-matching character is found
        } else {
            // Looking for an opening emphasis character
            if cur_index == 0
                || (cur_index > 0 && input[cur_index - 1..cur_index].find(is_separator).is_some())
            {
                start_emph = cur_index + 1;
                in_emph_char = Some(emph_char);
            }
            // Else ignoring emphasis character
        }
    }
    // Add remaining text as normal text
    if last_index < input.len() {
        output.push(TextType::Normal(&input[last_index..]));
    }
    output
}

///
/// Helper function to decide if an emphasis is done.
///
fn is_separator(c: char) -> bool {
    c.is_whitespace() || (c.is_ascii_punctuation() && c != '*' && c != '_')
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let res = parse_markdown_links("");
        assert_eq!(res, vec![]);
    }

    #[test]
    fn only_emphasis_char() {
        let res = parse_markdown_text("*");
        assert_eq!(res, vec![TextType::Normal("*")]);
    }

    #[test]
    fn no_link() {
        let res = parse_markdown_links("no link, file");
        assert_eq!(res, vec![Text::String(TextType::Normal("no link, file"))]);
    }

    #[test]
    fn simple_link() {
        let res = parse_markdown_links("https://www.google.com");
        assert_eq!(
            res,
            vec![Text::Link(Link {
                href: "https://www.google.com",
                text: vec![]
            })]
        );
    }

    #[test]
    fn link_with_emphasis() {
        let res = parse_markdown_links("[*Bold Title* normal](https://www.google.com)");
        assert_eq!(
            res,
            vec![Text::Link(Link {
                href: "https://www.google.com",
                text: vec![TextType::Bold("Bold Title"), TextType::Normal(" normal")]
            })]
        );
    }

    #[test]
    fn two_simple_links() {
        let res = parse_markdown_links("Goto (https://www.google.com) or https://www.yahoo.com");
        assert_eq!(
            res,
            vec![
                Text::String(TextType::Normal("Goto (")),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: vec![]
                }),
                Text::String(TextType::Normal(") or ")),
                Text::Link(Link {
                    href: "https://www.yahoo.com",
                    text: vec![]
                })
            ]
        );
    }

    #[test]
    fn simple_link_with_text() {
        let res = parse_markdown_links("[Google](https://www.google.com)");
        assert_eq!(
            res,
            vec![Text::Link(Link {
                href: "https://www.google.com",
                text: vec![TextType::Normal("Google")]
            })]
        );
    }

    #[test]
    fn nested_link() {
        let res = parse_markdown_links("[[Yahoo](https://www.yahoo.com)](https://www.google.com)");
        assert_eq!(
            res,
            vec![
                Text::String(TextType::Normal("[")),
                Text::Link(Link {
                    href: "https://www.yahoo.com",
                    text: vec![TextType::Normal("Yahoo")]
                }),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: vec![TextType::Normal("Yahoo](https://www.yahoo.com)")]
                })
            ]
        );
    }

    #[test]
    fn simple_link_with_quoted_text() {
        let res = parse_markdown_links("- \"[Google](https://www.google.com)\"");
        assert_eq!(
            res,
            vec![
                Text::String(TextType::Normal("- \"")),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: vec![TextType::Normal("Google")]
                }),
                Text::String(TextType::Normal("\""))
            ]
        );
    }

    #[test]
    fn simple_link_with_incomplete_text() {
        let res = parse_markdown_links("Google](https://www.google.com)");
        assert_eq!(
            res,
            vec![
                Text::String(TextType::Normal("Google](")),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: vec![]
                }),
                Text::String(TextType::Normal(")")),
            ]
        );
    }

    #[test]
    fn simple_link_with_just_brackets() {
        let res = parse_markdown_links("(https://www.google.com)");
        assert_eq!(
            res,
            vec![
                Text::String(TextType::Normal("(")),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: vec![]
                }),
                Text::String(TextType::Normal(")")),
            ]
        );
    }

    #[test]
    fn simple_link_with_just_opening_bracket() {
        let res = parse_markdown_links("(https://www.google.com and some other string");
        assert_eq!(
            res,
            vec![
                Text::String(TextType::Normal("(")),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: vec![]
                }),
                Text::String(TextType::Normal(" and some other string")),
            ]
        );
    }

    #[test]
    fn simple_italic() {
        let res = parse_markdown_text("This is an _italic_ text.");
        assert_eq!(
            res,
            vec![
                TextType::Normal("This is an "),
                TextType::Italic("italic"),
                TextType::Normal(" text.")
            ]
        );
    }

    #[test]
    fn simple_bold() {
        let res = parse_markdown_text("This is an *bold* text.");
        assert_eq!(
            res,
            vec![
                TextType::Normal("This is an "),
                TextType::Bold("bold"),
                TextType::Normal(" text.")
            ]
        );
    }

    #[test]
    fn double_italic() {
        let res = parse_markdown_text("__what is this__");
        assert_eq!(res, vec![TextType::Italic("_what is this_"),]);
    }

    #[test]
    fn crazy_emphasis1() {
        let res = parse_markdown_text("_*_or this* _ * ");
        assert_eq!(
            res,
            vec![TextType::Italic("*_or this* "), TextType::Normal(" * ")]
        );
    }

    #[test]
    fn crazy_emphasis2() {
        let res = parse_markdown_text("This is* another _scary_crazy_string_.");
        assert_eq!(
            res,
            vec![
                TextType::Normal("This is* another "),
                TextType::Italic("scary_crazy_string"),
                TextType::Normal("."),
            ]
        );
    }

    #[test]
    fn single_emphasis_chars() {
        let res = parse_markdown_text("This should not * match, and this neither _.");
        assert_eq!(
            res,
            vec![TextType::Normal(
                "This should not * match, and this neither _."
            ),]
        );
    }
}
