use std::io::BufRead;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Post {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
}

const TITLE_PREFIX: &str = "Title: ";
const DESCRIPTION_PREFIX: &str = "Description: ";
const TAGS_PREFIX: &str = "Tags: ";
const SEPARATOR: &str = "---";

// ANCHOR: from_dir
pub fn posts_from_dir(dir: impl AsRef<Path>) -> std::io::Result<Vec<Post>> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<std::io::Result<_>>()?;
    entries.sort();

    entries
        .iter()
        .map(|path| {
            let file = std::fs::File::open(path)?;
            post_from_reader(std::io::BufReader::new(file))
        })
        .collect()
}
// ANCHOR_END: from_dir

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

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn reads_every_post_in_a_directory() {
        let posts = posts_from_dir("testdata").unwrap();

        assert_eq!(posts.len(), 2);
    }

    #[test]
    fn parses_the_first_post_on_disk() {
        let posts = posts_from_dir("testdata").unwrap();

        assert_eq!(posts[0].title, "Hello, Twitch!");
        assert_eq!(posts[0].tags, vec!["streaming", "rust"]);
    }
    // ANCHOR_END: test
}
