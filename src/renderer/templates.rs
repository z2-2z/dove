use std::collections::HashSet;
use askama::Template;

#[derive(Template)]
#[template(path = "post/header.html")]
pub struct PostHeader<'a> {
    pub title: &'a str,
    pub uses_code: bool,
    pub languages: HashSet<String>,
}

#[derive(Template)]
#[template(path = "post/footer.html")]
pub struct PostFooter;

#[cfg(feature = "test-content")]
#[derive(Template)]
#[template(path = "test/content.html")]
pub struct TestContent;

#[derive(Template)]
#[template(path = "post/headline.html")]
pub struct Headline<'a> {
    pub headline: &'a str,
}

#[derive(Template)]
#[template(path = "post/paragraph.html")]
pub struct Paragraph {
    pub content: String,
}

#[derive(Template)]
#[template(path = "post/text.html")]
pub struct Text<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/subheading.html")]
pub struct Subheading {
    pub id: String,
    pub content: String,
}
