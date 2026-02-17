/// Slugify a display name into an ID.
///
/// Rules (from the architecture brief):
/// - Lowercase the text.
/// - Replace spaces with hyphens.
/// - Strip characters that are not alphanumeric or hyphens.
/// - Collapse consecutive hyphens.
/// - Trim leading and trailing hyphens.
pub fn slugify(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
        } else if ch == ' ' || ch == '-' {
            result.push('-');
        }
        // Strip all other characters
    }

    // Collapse consecutive hyphens
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_hyphen = false;
    for ch in result.chars() {
        if ch == '-' {
            if !prev_hyphen {
                collapsed.push('-');
            }
            prev_hyphen = true;
        } else {
            collapsed.push(ch);
            prev_hyphen = false;
        }
    }

    // Trim leading and trailing hyphens
    collapsed.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_slugification() {
        assert_eq!(slugify("The Rusty Anchor"), "the-rusty-anchor");
    }

    #[test]
    fn strips_special_characters() {
        assert_eq!(slugify("Ask about the harbor!"), "ask-about-the-harbor");
    }

    #[test]
    fn collapses_hyphens() {
        assert_eq!(slugify("foo - - bar"), "foo-bar");
    }

    #[test]
    fn trims_leading_trailing_hyphens() {
        assert_eq!(slugify("- hello -"), "hello");
    }

    #[test]
    fn already_slugified() {
        assert_eq!(slugify("cell"), "cell");
    }

    #[test]
    fn empty_input() {
        assert_eq!(slugify(""), "");
    }
}
