use std::io::Write;

use askama::Template;

pub struct Post {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
}

// ANCHOR: filters
mod filters {
    #[askama::filter_fn]
    pub fn slug<T: std::fmt::Display>(input: T, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(input.to_string().to_lowercase().replace(' ', "-"))
    }
}
// ANCHOR_END: filters

// ANCHOR: code
#[derive(Template)]
#[template(path = "blog.html")]
struct BlogTemplate<'a> {
    post: &'a Post,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    posts: &'a [Post],
}

pub fn render(writer: &mut impl Write, post: &Post) -> askama::Result<()> {
    BlogTemplate { post }.write_into(writer)?;
    Ok(())
}

pub fn render_index(writer: &mut impl Write, posts: &[Post]) -> askama::Result<()> {
    IndexTemplate { posts }.write_into(writer)?;
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

    fn render_to_string(post: &Post) -> String {
        let mut buffer = Vec::new();
        render(&mut buffer, post).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    // ANCHOR: test
    #[test]
    fn renders_the_post_metadata() {
        let got = render_to_string(&a_post());

        assert!(got.contains("<h1>hello world</h1>"), "{got}");
        assert!(got.contains("<p>This is a description</p>"), "{got}");
        assert!(got.contains("<li>rust</li><li>tdd</li>"), "{got}");
    }

    #[test]
    fn renders_an_index_of_posts_with_slugged_links() {
        let posts = [
            Post {
                title: "Hello World".to_string(),
                description: String::new(),
                tags: Vec::new(),
                body: String::new(),
            },
            Post {
                title: "Hello World 2".to_string(),
                description: String::new(),
                tags: Vec::new(),
                body: String::new(),
            },
        ];
        let mut buffer = Vec::new();

        render_index(&mut buffer, &posts).unwrap();

        let got = String::from_utf8(buffer).unwrap();
        let want = concat!(
            "<ol>",
            "<li><a href=\"/post/hello-world\">Hello World</a></li>",
            "<li><a href=\"/post/hello-world-2\">Hello World 2</a></li>",
            "</ol>"
        );
        assert_eq!(got, want);
    }
    // ANCHOR_END: test

    // ANCHOR: escape_test
    #[test]
    fn escapes_html_in_post_data() {
        let dodgy = Post {
            title: "<script>alert('xss')</script>".to_string(),
            description: "safe & sound".to_string(),
            tags: Vec::new(),
            body: String::new(),
        };

        let got = render_to_string(&dodgy);

        assert!(got.contains("&#60;script&#62;"), "{got}");
        assert!(got.contains("safe &#38; sound"), "{got}");
    }
    // ANCHOR_END: escape_test
}
