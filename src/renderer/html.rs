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
}

impl std::fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkdownError::InvalidHeading => write!(f, "Invalid heading. Only heading level 2 allowed"),
        }
    }
}

pub struct HtmlRenderer {
    file: PathBuf,
}

impl HtmlRenderer {
    pub fn new(output_dir: &str, post: &Post) -> Self {
        let mut file = PathBuf::from(output_dir);
        file.push(post.url());
        
        Self {
            file,
        }
    }
    
    pub fn output_file(&self) -> &Path {
        &self.file
    }
    
    pub fn render(&self, content: &[u8], post: &Post) -> Result<(), MarkdownError>  {
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
    
    fn dispatch(&self, event: md::Event, parser: &mut md::Parser, minimizer: &mut HtmlMinimizer) -> Result<(), MarkdownError> {
        match event {
            md::Event::Start(tag) => match tag {
                md::Tag::Paragraph => {
                    let data = self.collect(parser)?;
                    minimizer.append_template(Paragraph {
                        content: data.into_inner(),
                    });
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
                _ => {},
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
            md::Event::End(_) => unreachable!(),
            _ => {},
        }
        
        Ok(())
    }
    
    fn collect(&self, parser: &mut md::Parser) -> Result<HtmlMinimizer, MarkdownError> {
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
}
