use crate::file_utils::get_url_identifiers;

#[derive(Debug, PartialEq, Eq)]
pub struct Link<'a> {
    pub href: &'a str,
    pub text: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Text<'a> {
    String(&'a str),
    Link(Link<'a>),
}

///
/// Parse text and search for markdown syntax links
///
pub fn parse_markdown_links(input: &str) -> Vec<Text> {
    let mut output = Vec::new();
    let mut indices = get_url_identifiers()
        .iter()
        .flat_map(|url_id| input.match_indices(url_id))
        .collect::<Vec<_>>();
    indices.sort();
    let mut running_index = 0;

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
        if next_index > running_index {
            output.push(Text::String(&input[running_index..next_index]));
        }
        // Add the link itself
        output.push(Text::Link(Link {
            href: &input[start_link..end_link],
            text: link_text,
        }));
        running_index = end_link + skip_bracket;
    }
    if running_index < input.len() {
        output.push(Text::String(&input[running_index..]));
    }

    output
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
    fn no_link() {
        let res = parse_markdown_links("no link, file");
        assert_eq!(res, vec![Text::String("no link, file")]);
    }

    #[test]
    fn simple_link() {
        let res = parse_markdown_links("https://www.google.com");
        assert_eq!(
            res,
            vec![Text::Link(Link {
                href: "https://www.google.com",
                text: None
            })]
        );
    }

    #[test]
    fn two_simple_links() {
        let res = parse_markdown_links("Goto (https://www.google.com) or https://www.yahoo.com");
        assert_eq!(
            res,
            vec![
                Text::String("Goto ("),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: None
                }),
                Text::String(") or "),
                Text::Link(Link {
                    href: "https://www.yahoo.com",
                    text: None
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
                text: Some("Google")
            })]
        );
    }

    #[test]
    fn simple_link_with_quoted_text() {
        let res = parse_markdown_links("- \"[Google](https://www.google.com)\"");
        assert_eq!(
            res,
            vec![
                Text::String("- \""),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: Some("Google")
                }),
                Text::String("\"")
            ]
        );
    }

    #[test]
    fn simple_link_with_incomplete_text() {
        let res = parse_markdown_links("Google](https://www.google.com)");
        assert_eq!(
            res,
            vec![
                Text::String("Google]("),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: None
                }),
                Text::String(")"),
            ]
        );
    }

    #[test]
    fn simple_link_with_just_brackets() {
        let res = parse_markdown_links("(https://www.google.com)");
        assert_eq!(
            res,
            vec![
                Text::String("("),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: None
                }),
                Text::String(")"),
            ]
        );
    }

    #[test]
    fn simple_link_with_just_opening_bracket() {
        let res = parse_markdown_links("(https://www.google.com and some other string");
        assert_eq!(
            res,
            vec![
                Text::String("("),
                Text::Link(Link {
                    href: "https://www.google.com",
                    text: None
                }),
                Text::String(" and some other string"),
            ]
        );
    }
}
