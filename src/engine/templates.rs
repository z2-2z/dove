use crate::posts::CacheEntry;
use std::collections::HashSet;
use askama::Template;

#[derive(Template)]
#[template(path = "post/header.html")]
pub struct PostHeader<'a> {
    pub title: &'a str,
    pub uses_code: bool,
    pub languages: &'a HashSet<String>,
    pub keywords: String,
    pub url: &'a str,
}

#[derive(Template)]
#[template(path = "post/footer.html")]
pub struct PostFooter;

#[derive(Template)]
#[template(path = "post/headline.html")]
pub struct Headline<'a> {
    pub headline: &'a str,
}

#[derive(Template)]
#[template(path = "post/paragraph.html")]
pub struct Paragraph<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/text.html")]
pub struct Text<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/subheading.html")]
pub struct Subheading<'a> {
    pub id: &'a str,
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/quote.html")]
pub struct Quote<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/codeblock.html")]
pub struct Codeblock<'a> {
    pub language: &'a str,
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/tag.html")]
pub struct Tag<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/ordered_list.html")]
pub struct OrderedList<'a> {
    pub items: &'a str,
}

#[derive(Template)]
#[template(path = "post/unordered_list.html")]
pub struct UnorderedList<'a> {
    pub items: &'a str,
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
pub struct TableHead<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/table_row.html")]
pub struct TableRow<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/table_cell.html")]
pub struct TableCell<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/emphasis.html")]
pub struct Emphasis<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/bold.html")]
pub struct Bold<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/strikethrough.html")]
pub struct Strikethrough<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/link.html")]
pub struct Link<'a> {
    pub url: &'a str,
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "post/figure.html")]
pub struct Figure<'a> {
    pub number: usize,
    pub url: &'a str,
    pub description: &'a str,
    pub inside_p: bool,
}

#[derive(Template)]
#[template(path = "post/linebreak.html")]
pub struct Linebreak;

#[derive(Template)]
#[template(path = "post/blank_line.html")]
pub struct BlankLine;

#[derive(Template)]
#[template(path = "post/bibliography.html")]
pub struct Bibliography<'a> {
    pub references: &'a [(usize, String)],
}

#[derive(Template)]
#[template(path = "post/cite.html")]
pub struct Citation<'a> {
    pub ids: &'a [usize],
}

#[derive(Template)]
#[template(path = "post/categories.html")]
pub struct Categories<'a> {
    pub categories: &'a [String],
    pub day: u8,
    pub month: &'a str,
    pub year: u16,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    pub entries: &'a [&'a CacheEntry],
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct Status404;

#[derive(Template)]
#[template(path = "archive.html")]
pub struct Archive<'a> {
    pub entries: &'a [&'a CacheEntry],
}

#[derive(Template)]
#[template(path = "feed/atom_entry.xml")]
pub struct AtomEntry<'a> {
    pub title: &'a str,
    pub url: String,
    pub published: String,
    pub categories: &'a [String],
}

#[derive(Template)]
#[template(path = "feed/atom.xml")]
pub struct AtomFeed<'a> {
    pub updated: &'a str,
    pub entries: &'a [AtomEntry<'a>],
}
