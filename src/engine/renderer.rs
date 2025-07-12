use std::path::{Path, PathBuf};
use std::collections::{HashSet, HashMap};
use anyhow::Result;
use pulldown_cmark as md;
use askama::Template;

use crate::{transformer, engine::templates::*, parser};

#[inline]
fn append_template<T: Template>(output: &mut String, template: T) -> Result<()> {
    template.render_into(output)?;
    Ok(())
}

fn make_id(id: &str) -> String {
    let mut prev_dash = false;
    id.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .map(|c| if c.is_ascii_alphanumeric() {
            c.to_ascii_lowercase()
        } else {
            '-'
        })
        .filter(|c| {
            let ret = *c != '-' || !prev_dash;
            prev_dash = *c == '-';
            ret
        })
        .collect()
}

pub struct Renderer {
    table_cursor: usize,
    figure_cursor: usize,
    reference_cursor: usize,
    p_level: usize,
    description: String,
    file_mentions: HashSet<PathBuf>,
    references: HashMap<String, usize>,
    languages: HashSet<String>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            table_cursor: 1,
            figure_cursor: 1,
            reference_cursor: 1,
            p_level: 0,
            description: String::new(),
            file_mentions: HashSet::new(),
            references: HashMap::new(),
            languages: HashSet::new(),
        }
    }
    
    pub fn render(&mut self, content: &[u8], basedir: &Path) -> Result<String> {
        let content = std::str::from_utf8(content)?;
        let mut output = String::with_capacity(128 * 1024);
        let mut options = md::Options::empty();
        options.insert(md::Options::ENABLE_TABLES);
        options.insert(md::Options::ENABLE_STRIKETHROUGH);
        
        let mut parser = md::Parser::new_ext(content, options);
        
        while let Some(event) = parser.next() {
            self.dispatch(event, &mut parser, &mut output, basedir)?;
        }
        
        Ok(output)
    }
    
    fn dispatch(&mut self, event: md::Event, parser: &mut md::Parser, output: &mut String, basedir: &Path) -> Result<()> {
        match event {
            md::Event::Start(tag) => match tag {
                md::Tag::Paragraph => {
                    self.p_level += 1;
                    let data = self.collect(parser, basedir)?;
                    if !data.is_empty() {
                        append_template(output, Paragraph {
                            content: data,
                        })?;
                    }
                    self.p_level -= 1;
                },
                md::Tag::Heading { level, .. } => {
                    if !matches!(level, md::HeadingLevel::H2) {
                        anyhow::bail!("Invalid heading. Only ## headings are allowed");
                    }
                    let data = self.collect(parser, basedir)?;
                    let id = make_id(&data);
                    append_template(output, Subheading {
                        content: data,
                        id,
                    })?;
                },
                md::Tag::BlockQuote(_) => {
                    let data = self.collect(parser, basedir)?;
                    append_template(output, Quote {
                        content: data,
                    })?;
                },
                md::Tag::CodeBlock(kind) => {
                    let language = match kind {
                        md::CodeBlockKind::Indented => todo!("?"),
                        md::CodeBlockKind::Fenced(language) => match language.as_ref() {
                            "" => "plaintext".to_string(),
                            _ => language.as_ref().to_ascii_lowercase(),
                        }
                    };
                    let data = self.collect(parser, basedir)?;
                    append_template(output, Codeblock {
                        language: &language,
                        content: data,
                    })?;
                    self.languages.insert(language);
                },
                md::Tag::List(start_number) => {
                    if start_number.is_some() {
                        let data = self.collect(parser, basedir)?;
                        append_template(output, OrderedList {
                            items: data,
                        })?;
                    } else {
                        let data = self.collect(parser, basedir)?;
                        append_template(output, UnorderedList {
                            items: data,
                        })?;
                    }
                },
                md::Tag::Item => {
                    let data = self.collect(parser, basedir)?;
                    append_template(output, ListItem {
                        content: data,
                    })?;
                },
                md::Tag::Table(_) => {
                    let data = self.collect(parser, basedir)?;
                    append_template(output, Table {
                        number: self.table_cursor,
                        content: data,
                        description: &self.description,
                    })?;
                    self.table_cursor += 1;
                    self.description.clear();
                },
                md::Tag::TableHead => {
                    let data = self.collect(parser, basedir)?;
                    append_template(output, TableHead {
                        content: data,
                    })?;
                },
                md::Tag::TableRow => {
                    let data = self.collect(parser, basedir)?;
                    append_template(output, TableRow {
                        content: data,
                    })?;
                },
                md::Tag::TableCell => {
                    let data = self.collect(parser, basedir)?;
                    append_template(output, TableCell {
                        content: data,
                    })?;
                },
                md::Tag::Emphasis => {
                    let data = self.collect(parser, basedir)?;
                    append_template(output, Emphasis {
                        content: data,
                    })?;
                },
                md::Tag::Strong => {
                    let data = self.collect(parser, basedir)?;
                    append_template(output, Bold {
                        content: data,
                    })?;
                },
                md::Tag::Strikethrough => {
                    let data = self.collect(parser, basedir)?;
                    append_template(output, Strikethrough {
                        content: data,
                    })?;
                },
                md::Tag::Link { dest_url, .. } => {
                    let data = self.collect(parser, basedir)?;
                    append_template(output, Link {
                        url: dest_url.as_ref(),
                        content: data,
                    })?;
                    let path = basedir.join(dest_url.as_ref());
                    if path.exists() {
                        self.file_mentions.insert(path);
                    }
                },
                md::Tag::Image { dest_url, .. } => {
                    self.collect(parser, basedir)?;
                    
                    let mut path = basedir.join(dest_url.as_ref());
                    
                    if path.exists() {
                        self.file_mentions.insert(path.clone());
                    }
                    
                    let url = if path.exists() && transformer::is_image(&path) {
                        path = transformer::transform_image_filename(dest_url.as_ref());
                        path.to_str().unwrap()
                    } else {
                        dest_url.as_ref()
                    };
                    
                    append_template(output, Figure {
                        number: self.figure_cursor,
                        url,
                        description: &self.description,
                        inside_p: self.p_level > 0,
                    })?;
                    self.figure_cursor += 1;
                    self.description.clear();
                },
                md::Tag::HtmlBlock => {
                    let data = self.collect(parser, basedir)?;
                    output.push_str(&data);
                },
                _ => unreachable!("{:?}", tag),
            },
            md::Event::Html(tag) => {
                match tag.as_ref().trim() {
                    "<table-title>" => {
                        let data = self.collect_html(parser, "</table-title>", basedir)?;
                        self.description = data;
                    },
                    "<figure-title>" => {
                        let data = self.collect_html(parser, "</figure-title>", basedir)?;
                        self.description = data;
                    },
                    "<cite>" => {
                        let data = self.collect_html(parser, "</cite>", basedir)?;
                        let mut ids = Vec::new();
                        
                        for cite_id in data.split(',') {
                            for cite_id in cite_id.split(' ') {
                                if !cite_id.is_empty() {
                                    if let Some(id) = self.references.get(cite_id) {
                                        ids.push(*id);
                                    } else {
                                        let id = self.reference_cursor;
                                        self.reference_cursor += 1;
                                        self.references.insert(cite_id.to_string(), id);
                                        ids.push(id);
                                    }
                                }
                            }
                        }
                        
                        append_template(output, Citation {
                            ids,
                        })?;
                    },
                    "<blank-line>" | "<blank-line/>" => {
                        append_template(output, BlankLine {})?;
                    },
                    tag => anyhow::bail!("Invalid html tag: {}", tag),
                }
            },
            md::Event::InlineHtml(tag) => {
                match tag.as_ref().trim() {
                    "<blank-line>" | "<blank-line/>" => {
                        append_template(output, BlankLine {})?;
                    },
                    "<table-title>" => {
                        let data = self.collect_html(parser, "</table-title>", basedir)?;
                        self.description = data;
                    },
                    "<figure-title>" => {
                        let data = self.collect_html(parser, "</figure-title>", basedir)?;
                        self.description = data;
                    },
                    "<cite>" => {
                        let data = self.collect_html(parser, "</cite>", basedir)?;
                        let mut ids = Vec::new();
                        
                        for cite_id in data.split(',') {
                            for cite_id in cite_id.split(' ') {
                                if !cite_id.is_empty() {
                                    if let Some(id) = self.references.get(cite_id) {
                                        ids.push(*id);
                                    } else {
                                        let id = self.reference_cursor;
                                        self.reference_cursor += 1;
                                        self.references.insert(cite_id.to_string(), id);
                                        ids.push(id);
                                    }
                                }
                            }
                        }
                        
                        append_template(output, Citation {
                            ids,
                        })?;
                    },
                    tag => anyhow::bail!("Invalid inline html: {}", tag),
                }
            },
            md::Event::Code(content) => {
                append_template(output, Tag {
                    content: content.as_ref(),
                })?;
            },
            md::Event::Text(text) => {
                append_template(output, Text {
                    content: text.as_ref(),
                })?;
            },
            md::Event::SoftBreak => {
                output.push(' ');
            },
            md::Event::HardBreak => {
                append_template(output, Linebreak {})?;
            },
            md::Event::Rule => {
                assert_eq!(self.p_level, 0);
                self.parse_bibliography(parser, output, basedir)?;
            },
            _ => unreachable!("{:?}", event),
        }
        
        Ok(())
    }
    
    fn collect(&mut self, parser: &mut md::Parser, basedir: &Path) -> Result<String> {
        let mut temp = String::with_capacity(4 * 1024);
        
        while let Some(event) = parser.next() {
            match event {
                md::Event::End(_) => {
                    break;
                },
                event => self.dispatch(event, parser, &mut temp, basedir)?,
            }
        }
        
        Ok(temp)
    }
    
    fn collect_html(&mut self, parser: &mut md::Parser, end_tag: &str, basedir: &Path) -> Result<String> {
        let mut temp = String::with_capacity(4 * 1024);
        
        while let Some(event) = parser.next() {
            match &event {
                md::Event::InlineHtml(tag) |
                md::Event::Html(tag) => if tag.as_ref() == end_tag {
                    break;
                }
                _ => {},
            }
            
            self.dispatch(event, parser, &mut temp, basedir)?;
        }
        
        Ok(temp)
    }
    
    fn parse_reference_tag(&self, parser: &mut md::Parser) -> Result<usize> {
        let event = parser.next();
        let tag = match &event {
            Some(md::Event::Html(tag)) |
            Some(md::Event::InlineHtml(tag)) => tag.as_ref(),
            _ => anyhow::bail!("Only <ref> tags are allowed in the bibliography"),
        };
        
        if !tag.starts_with("<ref ") || !tag.ends_with('>') {
            anyhow::bail!("Bibliography only accepts <ref> tags");
        }
        
        let tag = &tag.as_bytes()[4..];
        let tag = parser::trim_whitespaces(tag);
        
        /* expect id=" */
        let (key, value) = parser::split_once(tag, b'"');
        
        if key != b"id=" {
            anyhow::bail!("Invalid <ref> tag in bibliography");
        }
        
        /* Extract id */
        let (id, _) = parser::split_once(value, b'"');
        let id = parser::trim_whitespaces(id);
        let id = std::str::from_utf8(id)?;
        
        if let Some(id) = self.references.get(id) {
            Ok(*id)
        } else {
            anyhow::bail!("Bibliography entry with id '{id}' is never referenced")
        }
    }
    
    fn parse_bibliography(&mut self, parser: &mut md::Parser, output: &mut String, basedir: &Path) -> Result<()> {
        /* Collect all the markdown */
        let mut bib = Vec::new();
        
        assert!(matches!(parser.next(), Some(md::Event::Start(md::Tag::Paragraph))));
        
        loop {
            let id = self.parse_reference_tag(parser)?;
            let data = self.collect_html(parser, "</ref>", basedir)?;
            
            bib.push((id, data));
            
            match parser.next() {
                Some(md::Event::End(md::TagEnd::Paragraph)) => {
                    break;
                },
                Some(md::Event::SoftBreak) => {},
                _ => anyhow::bail!("Bibliography has unexpected content"),
            }
        }
        
        if parser.next().is_some() {
            anyhow::bail!("Bibliography is not last element in post");
        }
        
        bib.sort_by(|a, b| a.0.cmp(&b.0));
        
        /* Check that all citations have an entry in the bibliography */
        for (name, id) in &self.references {
            if let Some(entry) = bib.get(*id - 1) {
                if entry.0 != *id {
                    anyhow::bail!("Bibliography entry for '{name}' is missing");
                }
            } else {
                anyhow::bail!("Bibliography entry for '{name}' is missing");
            }
        }
        
        /* Generate html */
        append_template(output, Bibliography {
            references: bib,
        })?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn render_example() {
        let post = crate::posts::Post::new("test-data/renderer/example.md").unwrap();
        let mut renderer = Renderer::new();
        let output = renderer.render(post.content(), Path::new("test-data/renderer/")).unwrap();
        println!("{output}");
    }
}
