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

#[derive(Template)]
#[template(path = "post/quote.html")]
pub struct Quote {
    pub content: String,
}

#[derive(Template)]
#[template(path = "post/codeblock.html")]
pub struct Codeblock {
    pub language: String,
    pub content: String,
}

#[derive(Template)]
#[template(path = "post/tag.html")]
pub struct Tag<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/ordered_list.html")]
pub struct OrderedList {
    pub items: String,
}

#[derive(Template)]
#[template(path = "post/unordered_list.html")]
pub struct UnorderedList {
    pub items: String,
}

#[derive(Template)]
#[template(path = "post/list_item.html")]
pub struct ListItem {
    pub content: String,
}

#[derive(Template)]
#[template(path = "post/table.html")]
pub struct Table<'a> {
    pub number: usize,
    pub content: String,
    pub description: &'a str,
}

#[derive(Template)]
#[template(path = "post/table_head.html")]
pub struct TableHead {
    pub content: String,
}

#[derive(Template)]
#[template(path = "post/table_row.html")]
pub struct TableRow {
    pub content: String,
}

#[derive(Template)]
#[template(path = "post/table_cell.html")]
pub struct TableCell {
    pub content: String,
}
