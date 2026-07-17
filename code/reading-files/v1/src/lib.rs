use std::io::BufRead;

// ANCHOR: code
#[derive(Debug, PartialEq)]
pub struct Post {
    pub title: String,
}

pub fn post_from_reader(reader: impl BufRead) -> std::io::Result<Post> {
    let mut lines = reader.lines();

    let title_line = match lines.next() {
        Some(line) => line?,
        None => return Err(invalid_post("the post is empty")),
    };

    let title = title_line
        .strip_prefix("Title: ")
        .ok_or_else(|| invalid_post("expected a line starting with \"Title: \""))?;

    Ok(Post {
        title: title.to_string(),
    })
}

fn invalid_post(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.to_string())
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn parses_the_title_from_a_post() {
        let post = post_from_reader("Title: Hello, TDD world!".as_bytes()).unwrap();

        assert_eq!(post.title, "Hello, TDD world!");
    }

    #[test]
    fn parses_a_different_title() {
        let post = post_from_reader("Title: Hello, Twitch!".as_bytes()).unwrap();

        assert_eq!(post.title, "Hello, Twitch!");
    }

    #[test]
    fn rejects_a_post_with_no_title_line() {
        let result = post_from_reader("not a title".as_bytes());

        assert!(result.is_err(), "expected an error but didn't get one");
    }
    // ANCHOR_END: test
}
