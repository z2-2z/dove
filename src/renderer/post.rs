use std::path::{PathBuf, Path};
use pulldown_cmark as md;
use std::collections::{HashSet, HashMap};
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
    TaskList,
    NonUtf8,
    NoRefId,
    InvalidRefId(String),
    NoBibliography,
    InvalidBibliography(&'static str),
    NoCitation(String),
}

impl std::fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkdownError::InvalidHeading => write!(f, "Invalid heading. Only heading level 2 allowed"),
            MarkdownError::Footnote => write!(f, "Footnotes are not supported"),
            MarkdownError::InvalidHtml(tag) => write!(f, "Invalid html tag: '{}'", tag),
            MarkdownError::TaskList => write!(f, "Tasklists are not supported"),
            MarkdownError::NonUtf8 => write!(f, "Post content is not utf-8"),
            MarkdownError::NoRefId => write!(f, "Reference tag in bibliography has no id attribute"),
            MarkdownError::InvalidRefId(id) => write!(f, "The bibliography has an entry with id '{}' but this id is never referenced", id),
            MarkdownError::NoBibliography => write!(f, "After a rule only reference elements are allowed to build the bibliography"),
            MarkdownError::InvalidBibliography(msg) => write!(f, "Bibliography is invalid: {}", msg),
            MarkdownError::NoCitation(id) => write!(f, "Bibliography entry for '{}' is missing", id),
        }
    }
}

pub struct PostRenderer {
    file: PathBuf,
    tables: usize,
    description: String,
    figures: usize,
    p_level: usize,
    urls: HashSet<String>,
    citations: HashMap<String, usize>,
    citations_cursor: usize,
    languages: HashSet<String>,
}

impl PostRenderer {
    pub fn new(output_dir: &str, post: &Post) -> Self {
        let mut file = PathBuf::from(output_dir);
        file.push(post.filename());
        
        Self {
            file,
            tables: 1,
            figures: 1,
            description: String::new(),
            p_level: 0,
            urls: HashSet::new(),
            citations: HashMap::new(),
            citations_cursor: 1,
            languages: HashSet::new(),
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
        
        /* Check for code blocks */
        for elem in md::Parser::new_ext(content, options) {
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
                        self.languages.insert(language);
                    },
                },
                _ => {},
            }
        }
        
        /* Generate html */
        minimizer.append_template(PostHeader {
            title: post.metadata().title(),
            uses_code,
            languages: &self.languages,
            keywords: post.metadata().categories().join(", "),
            url: post.filename(),
        });
        minimizer.append_template(Headline {
            headline: post.metadata().title(),
        });
        minimizer.append_template(Categories {
            categories: post.metadata().categories(),
            day: post.metadata().date().day(),
            month: post.metadata().date().month_name(),
            year: post.metadata().date().year(),
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
                match tag.as_ref().trim() {
                    "<table-title>" => {
                        let data = self.collect_html(parser, "</table-title>")?;
                        self.description = data.into_inner();
                    },
                    "<figure-title>" => {
                        let data = self.collect_html(parser, "</figure-title>")?;
                        self.description = data.into_inner();
                    },
                    /*"<br>" | "<br/>" => {
                        minimizer.append_template(Linebreak {});
                    },*/
                    "<cite>" => {
                        let data = self.collect_html(parser, "</cite>")?.into_inner();
                        let mut ids = Vec::new();
                        
                        for cite_id in data.split(',') {
                            for cite_id in cite_id.split(' ') {
                                if !cite_id.is_empty() {
                                    if let Some(id) = self.citations.get(cite_id) {
                                        ids.push(*id);
                                    } else {
                                        let id = self.citations_cursor;
                                        self.citations_cursor += 1;
                                        self.citations.insert(cite_id.to_string(), id);
                                        ids.push(id);
                                    }
                                }
                            }
                        }
                        
                        minimizer.append_template(Citation {
                            ids,
                        });
                    },
                    "<blank-line>" | "<blank-line/>" => {
                        minimizer.append_template(BlankLine {});
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
            md::Event::SoftBreak => {
                minimizer.append_string(" ");
            },
            md::Event::HardBreak => {
                minimizer.append_template(Linebreak {});
            },
            md::Event::Rule => {
                assert_eq!(self.p_level, 0);
                self.parse_bibliography(parser, minimizer)?;
            },
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
    
    fn parse_reference_tag(&self, parser: &mut md::Parser) -> Result<usize, MarkdownError> {
        let event = parser.next();
        let tag = match &event {
            Some(md::Event::Html(tag)) => tag.as_ref(),
            _ => return Err(MarkdownError::InvalidBibliography("Only reference tags allowed")),
        };
        
        if !tag.starts_with("<reference ") || !tag.ends_with('>') {
            return Err(MarkdownError::NoBibliography);
        }
        
        let tag = tag.as_bytes();
        let mut cursor = 11;
        
        /* skip spaces */
        while tag.get(cursor).copied() == Some(b' ') {
            cursor += 1;
        }
        
        /* expect id=" */
        if tag.get(cursor..cursor + 4) != Some(b"id=\"") {
            return Err(MarkdownError::NoRefId);
        }
        
        cursor += 4;
        
        /* Extract id */
        let start = cursor;
        
        while tag.get(cursor).copied() != Some(b'"') {
            cursor += 1;
        }
        
        if cursor >= tag.len() {
            return Err(MarkdownError::NoRefId);
        }
        
        let id = std::str::from_utf8(&tag[start..cursor]).map_err(|_| MarkdownError::NonUtf8)?;
        
        if let Some(id) = self.citations.get(id) {
            Ok(*id)
        } else {
            Err(MarkdownError::InvalidRefId(id.to_string()))
        }
    }
    
    fn parse_bibliography(&mut self, parser: &mut md::Parser, minimizer: &mut HtmlMinimizer) -> Result<(), MarkdownError> {
        let mut bib = Vec::new();
        
        assert!(matches!(parser.next(), Some(md::Event::Start(md::Tag::Paragraph))));
        
        loop {
            let id = self.parse_reference_tag(parser)?;
            let data = self.collect_html(parser, "</reference>")?.into_inner();
            
            bib.push((id, data));
            
            match parser.next() {
                Some(md::Event::End(md::Tag::Paragraph)) => {
                    break;
                },
                Some(md::Event::SoftBreak) => {},
                _ => return Err(MarkdownError::InvalidBibliography("Bibliography has unexpected content")),
            }
        }
        
        if parser.next().is_some() {
            return Err(MarkdownError::InvalidBibliography("Bibliography is not the last item in the post"));
        }
        
        bib.sort_by(|a, b| a.0.cmp(&b.0));
        
        /* Check that all citations have an entry in the bibliography */
        for (name, id) in &self.citations {
            if let Some(entry) = bib.get(*id - 1) {
                if entry.0 != *id {
                    return Err(MarkdownError::NoCitation(name.to_string()));
                }
            } else {
                return Err(MarkdownError::NoCitation(name.to_string()));
            }
        }
        
        /* Generate html */
        minimizer.append_template(Bibliography {
            references: bib,
        });
        
        Ok(())
    }
    
    pub fn urls(&self) -> &HashSet<String> {
        &self.urls
    }
    
    pub fn languages(&self) -> &HashSet<String> {
        &self.languages
    }
}
