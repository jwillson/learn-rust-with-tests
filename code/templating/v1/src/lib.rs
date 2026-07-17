use std::io::Write;

use askama::Template;

// ANCHOR: post
pub struct Post {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
}
// ANCHOR_END: post

// ANCHOR: code
#[derive(Template)]
#[template(path = "blog.html")]
struct BlogTemplate<'a> {
    title: &'a str,
}

pub fn render(writer: &mut impl Write, post: &Post) -> askama::Result<()> {
    BlogTemplate { title: &post.title }.write_into(writer)?;
    Ok(())
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    fn a_post() -> Post {
        Post {
            title: "hello world".to_string(),
            description: "This is a description".to_string(),
            tags: vec!["rust".to_string(), "tdd".to_string()],
            body: "This is a post".to_string(),
        }
    }

    // ANCHOR: test
    #[test]
    fn renders_a_post_title_as_a_heading() {
        let mut buffer = Vec::new();

        render(&mut buffer, &a_post()).unwrap();

        let got = String::from_utf8(buffer).unwrap();
        assert_eq!(got, "<h1>hello world</h1>");
    }
    // ANCHOR_END: test
}
