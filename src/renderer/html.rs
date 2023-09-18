use std::path::{PathBuf, Path};
use pulldown_cmark as md;
use std::collections::HashSet;
use crate::posts::post::Post;
use crate::renderer::templates::*;
use crate::mini::html::HtmlMinimizer;

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

pub enum MarkdownError {
    InvalidHeading,
    Footnote,
    InvalidHtml(String),
    Rule,
    TaskList,
}

impl std::fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkdownError::InvalidHeading => write!(f, "Invalid heading. Only heading level 2 allowed"),
            MarkdownError::Footnote => write!(f, "Footnotes are not supported"),
            MarkdownError::InvalidHtml(tag) => write!(f, "Invalid html tag: {}", tag),
            MarkdownError::Rule => write!(f, "Rules are not supported"),
            MarkdownError::TaskList => write!(f, "Tasklists are not supported"),
        }
    }
}

pub struct HtmlRenderer {
    file: PathBuf,
    tables: usize,
    description: String,
    figures: usize,
    p_level: usize,
    urls: HashSet<String>,
}

impl HtmlRenderer {
    pub fn new(output_dir: &str, post: &Post) -> Self {
        let mut file = PathBuf::from(output_dir);
        file.push(post.url());
        
        Self {
            file,
            tables: 1,
            figures: 1,
            description: String::new(),
            p_level: 0,
            urls: HashSet::new(),
        }
    }
    
    pub fn output_file(&self) -> &Path {
        &self.file
    }
    
    pub fn render(&mut self, content: &[u8], post: &Post) -> Result<(), MarkdownError>  {
        std::fs::create_dir_all(self.file.parent().unwrap()).unwrap();
        let content = std::str::from_utf8(&content[post.metadata().start_content()..]).unwrap();
        let mut minimizer = HtmlMinimizer::new();
        let mut options = md::Options::empty();
        options.insert(md::Options::ENABLE_TABLES);
        options.insert(md::Options::ENABLE_STRIKETHROUGH);
        let mut uses_code = false;
        let mut languages: HashSet<String> = HashSet::new();
        
        /* Check for code blocks */
        for elem in md::Parser::new_ext(content, options) {
            println!("{:?}", elem);
            
            match elem {
                md::Event::Code(_) => uses_code = true,
                md::Event::Start(md::Tag::CodeBlock(kind)) => match kind {
                    md::CodeBlockKind::Indented => todo!("what does this mean?"),
                    md::CodeBlockKind::Fenced(language) => {
                        uses_code = true;
                        let language = match language.as_ref() {
                            "" => "plaintext".to_string(),
                            language => language.to_ascii_lowercase(),
                        };
                        languages.insert(language);
                    },
                },
                _ => {},
            }
        }
        
        /* Generate html */
        minimizer.append_template(PostHeader {
            title: post.metadata().title(),
            uses_code,
            languages,
        });
        minimizer.append_template(Headline {
            headline: post.metadata().title(),
        });
        
        #[cfg(not(feature = "test-content"))]
        {
            let mut parser = md::Parser::new_ext(content, options);
            
            while let Some(event) = parser.next() {
                self.dispatch(event, &mut parser, &mut minimizer)?;
            }
        }
        
        #[cfg(feature = "test-content")]
        minimizer.append_template(TestContent {});
        
        minimizer.append_template(PostFooter {});
        minimizer.minimize(&self.file);
        
        Ok(())
    }
    
    fn dispatch(&mut self, event: md::Event, parser: &mut md::Parser, minimizer: &mut HtmlMinimizer) -> Result<(), MarkdownError> {
        match event {
            md::Event::Start(tag) => match tag {
                md::Tag::Paragraph => {
                    self.p_level += 1;
                    let data = self.collect(parser)?;
                    minimizer.append_template(Paragraph {
                        content: data.into_inner(),
                    });
                    self.p_level -= 1;
                },
                md::Tag::Heading(level, _, _) => {
                    if !matches!(level, md::HeadingLevel::H2) {
                        return Err(MarkdownError::InvalidHeading);
                    }
                    let data = self.collect(parser)?.into_inner();
                    let id = make_id(&data);
                    minimizer.append_template(Subheading {
                        content: data,
                        id,
                    });
                },
                md::Tag::BlockQuote => {
                    let data = self.collect(parser)?;
                    minimizer.append_template(Quote {
                        content: data.into_inner(),
                    });
                },
                md::Tag::CodeBlock(kind) => {
                    let language = match kind {
                        md::CodeBlockKind::Indented => "plaintext".to_string(),
                        md::CodeBlockKind::Fenced(language) => language.as_ref().to_ascii_lowercase(),
                    };
                    let data = self.collect(parser)?;
                    minimizer.append_template(Codeblock {
                        language,
                        content: data.into_inner(),
                    });
                },
                md::Tag::List(start_number) => {
                    if start_number.is_some() {
                        let data = self.collect(parser)?;
                        minimizer.append_template(OrderedList {
                            items: data.into_inner(),
                        });
                    } else {
                        let data = self.collect(parser)?;
                        minimizer.append_template(UnorderedList {
                            items: data.into_inner(),
                        });
                    }
                },
                md::Tag::Item => {
                    let data = self.collect(parser)?;
                    minimizer.append_template(ListItem {
                        content: data.into_inner(),
                    });
                },
                md::Tag::FootnoteDefinition(_) => {
                    return Err(MarkdownError::Footnote);
                },
                md::Tag::Table(_) => {
                    let data = self.collect(parser)?;
                    minimizer.append_template(Table {
                        number: self.tables,
                        content: data.into_inner(),
                        description: &self.description,
                    });
                    self.tables += 1;
                    self.description.clear();
                },
                md::Tag::TableHead => {
                    let data = self.collect(parser)?;
                    minimizer.append_template(TableHead {
                        content: data.into_inner(),
                    });
                },
                md::Tag::TableRow => {
                    let data = self.collect(parser)?;
                    minimizer.append_template(TableRow {
                        content: data.into_inner(),
                    });
                },
                md::Tag::TableCell => {
                    let data = self.collect(parser)?;
                    minimizer.append_template(TableCell {
                        content: data.into_inner(),
                    });
                },
                md::Tag::Emphasis => {
                    let data = self.collect(parser)?;
                    minimizer.append_template(Emphasis {
                        content: data.into_inner(),
                    });
                },
                md::Tag::Strong => {
                    let data = self.collect(parser)?;
                    minimizer.append_template(Bold {
                        content: data.into_inner(),
                    });
                },
                md::Tag::Strikethrough => {
                    let data = self.collect(parser)?;
                    minimizer.append_template(Strikethrough {
                        content: data.into_inner(),
                    });
                },
                md::Tag::Link(_, url, title) => {
                    assert!(title.is_empty());
                    let data = self.collect(parser)?;
                    minimizer.append_template(Link {
                        url: url.as_ref(),
                        content: data.into_inner(),
                    });
                    self.urls.insert(url.into_string());
                },
                md::Tag::Image(_, url, title) => {
                    assert!(title.is_empty());
                    self.collect(parser)?;
                    minimizer.append_template(Figure {
                        number: self.figures,
                        url: url.as_ref(),
                        description: &self.description,
                        inside_p: self.p_level > 0,
                    });
                    self.figures += 1;
                    self.description.clear();
                    self.urls.insert(url.into_string());
                },
            },
            md::Event::Html(tag) => {
                match tag.as_ref() {
                    "<table-title>" => {
                        let data = self.collect_html(parser, "</table-title>")?;
                        self.description = data.into_inner();
                    },
                    "<figure-title>" => {
                        let data = self.collect_html(parser, "</figure-title>")?;
                        self.description = data.into_inner();
                    },
                    "<br>" | "<br/>" => {
                        minimizer.append_template(Linebreak {});
                    },
                    tag => return Err(MarkdownError::InvalidHtml(tag.to_string())),
                }
            },
            md::Event::Code(content) => {
                minimizer.append_template(Tag {
                    content: content.as_ref(),
                });
            },
            md::Event::Text(text) => {
                minimizer.append_template(Text {
                    content: text.as_ref(),
                });
            },
            md::Event::FootnoteReference(_) => return Err(MarkdownError::Footnote),
            md::Event::SoftBreak => {},
            md::Event::HardBreak => {
                minimizer.append_template(Linebreak {});
            },
            md::Event::Rule => return Err(MarkdownError::Rule),
            md::Event::End(_) => unreachable!(),
            md::Event::TaskListMarker(_) => return Err(MarkdownError::TaskList),
        }
        
        Ok(())
    }
    
    fn collect(&mut self, parser: &mut md::Parser) -> Result<HtmlMinimizer, MarkdownError> {
        let mut temp = HtmlMinimizer::new();
        
        while let Some(event) = parser.next() {
            match event {
                md::Event::End(_) => {
                    break;
                },
                event => self.dispatch(event, parser, &mut temp)?,
            }
        }
        
        Ok(temp)
    }
    
    fn collect_html(&mut self, parser: &mut md::Parser, end_tag: &str) -> Result<HtmlMinimizer, MarkdownError> {
        let mut temp = HtmlMinimizer::new();
        
        while let Some(event) = parser.next() {
            if let md::Event::Html(tag) = &event {
                if tag.as_ref() == end_tag {
                    break;
                }
            }
            
            self.dispatch(event, parser, &mut temp)?;
        }
        
        Ok(temp)
    }
    
    pub fn urls(&self) -> &HashSet<String> {
        &self.urls
    }
}
