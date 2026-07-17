// ANCHOR: shout
pub fn shout(text: &str) -> String {
    text.to_uppercase()
}
// ANCHOR_END: shout

// ANCHOR: preview
pub struct Preview<'a> {
    pub title: String,
    pub excerpt: &'a str,
}

impl<'a> Preview<'a> {
    pub fn new(title: &str, body: &'a str) -> Preview<'a> {
        Preview {
            title: title.to_uppercase(),
            excerpt: first_sentence(body),
        }
    }
}
// ANCHOR_END: preview

pub fn first_sentence(text: &str) -> &str {
    match text.find('.') {
        Some(position) => &text[..position],
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shout_returns_an_upper_case_copy() {
        let got = shout("may contain traces of lifetimes");

        assert_eq!(got, "MAY CONTAIN TRACES OF LIFETIMES");
    }

    // ANCHOR: preview_test
    #[test]
    fn preview_shows_the_title_and_the_first_sentence() {
        let body = "Lifetimes are regions, not clocks. They are resolved entirely at \
                    compile time.";

        let preview = Preview::new("understanding lifetimes", body);

        assert_eq!(preview.title, "UNDERSTANDING LIFETIMES");
        assert_eq!(preview.excerpt, "Lifetimes are regions, not clocks");
    }
    // ANCHOR_END: preview_test
}
