use std::io::BufRead;

// ANCHOR: post
#[derive(Debug, PartialEq)]
pub struct Post {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
}
// ANCHOR_END: post

// ANCHOR: code
const TITLE_PREFIX: &str = "Title: ";
const DESCRIPTION_PREFIX: &str = "Description: ";
const TAGS_PREFIX: &str = "Tags: ";
const SEPARATOR: &str = "---";

pub fn post_from_reader(reader: impl BufRead) -> std::io::Result<Post> {
    let mut lines = reader.lines();

    let title = read_meta_line(&mut lines, TITLE_PREFIX)?;
    let description = read_meta_line(&mut lines, DESCRIPTION_PREFIX)?;
    let tags = read_meta_line(&mut lines, TAGS_PREFIX)?
        .split(", ")
        .map(String::from)
        .collect();

    let separator = match lines.next() {
        Some(line) => line?,
        None => return Err(invalid_post("expected a \"---\" separator line")),
    };
    if separator != SEPARATOR {
        return Err(invalid_post("expected a \"---\" separator line"));
    }

    let body_lines: Vec<String> = lines.collect::<std::io::Result<_>>()?;
    let body = body_lines.join("\n");

    Ok(Post {
        title,
        description,
        tags,
        body,
    })
}

fn read_meta_line(
    lines: &mut impl Iterator<Item = std::io::Result<String>>,
    prefix: &str,
) -> std::io::Result<String> {
    let line = match lines.next() {
        Some(line) => line?,
        None => return Err(invalid_post("the post ended before its metadata did")),
    };

    match line.strip_prefix(prefix) {
        Some(value) => Ok(value.to_string()),
        None => Err(invalid_post(&format!(
            "expected a line starting with {prefix:?}"
        ))),
    }
}

fn invalid_post(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.to_string())
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    const FIRST_POST: &str = "\
Title: Hello, TDD world!
Description: First post on our wonderful blog
Tags: tdd, rust
---
Hello world!

The body of posts starts after the `---`";

    #[test]
    fn parses_a_whole_post() {
        let got = post_from_reader(FIRST_POST.as_bytes()).unwrap();

        let want = Post {
            title: "Hello, TDD world!".to_string(),
            description: "First post on our wonderful blog".to_string(),
            tags: vec!["tdd".to_string(), "rust".to_string()],
            body: "Hello world!\n\nThe body of posts starts after the `---`".to_string(),
        };

        assert_eq!(got, want);
    }
    // ANCHOR_END: test

    #[test]
    fn rejects_a_post_with_missing_metadata() {
        let result = post_from_reader("Title: only a title".as_bytes());

        assert!(result.is_err(), "expected an error but didn't get one");
    }

    #[test]
    fn rejects_a_post_without_a_separator() {
        let post = "Title: a\nDescription: b\nTags: c\nno separator here";

        let result = post_from_reader(post.as_bytes());

        assert!(result.is_err(), "expected an error but didn't get one");
    }
}
