use std::io::Write;

use askama::Template;

pub struct Post {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
}

mod filters {
    #[askama::filter_fn]
    pub fn slug<T: std::fmt::Display>(input: T, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(input.to_string().to_lowercase().replace(' ', "-"))
    }
}

// ANCHOR: code
#[derive(Template)]
#[template(path = "blog.html")]
struct BlogTemplate<'a> {
    post: &'a Post,
    html_body: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    posts: &'a [Post],
}

pub fn render(writer: &mut impl Write, post: &Post) -> askama::Result<()> {
    let html_body = markdown_to_html(&post.body);
    BlogTemplate { post, html_body }.write_into(writer)?;
    Ok(())
}

pub fn render_index(writer: &mut impl Write, posts: &[Post]) -> askama::Result<()> {
    IndexTemplate { posts }.write_into(writer)?;
    Ok(())
}

fn markdown_to_html(markdown: &str) -> String {
    let parser = pulldown_cmark::Parser::new(markdown);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(post: &Post) -> String {
        let mut buffer = Vec::new();
        render(&mut buffer, post).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    // ANCHOR: test
    #[test]
    fn renders_the_markdown_body_as_html() {
        let post = Post {
            title: "hello world".to_string(),
            description: "desc".to_string(),
            tags: Vec::new(),
            body: "This is *emphasised* and this is a [link](https://example.com).".to_string(),
        };

        let got = render_to_string(&post);

        assert!(got.contains("<em>emphasised</em>"), "{got}");
        assert!(
            got.contains(r#"<a href="https://example.com">link</a>"#),
            "{got}"
        );
    }
    // ANCHOR_END: test
}
