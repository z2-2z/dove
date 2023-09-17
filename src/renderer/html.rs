use std::path::{PathBuf, Path};
use std::borrow::Borrow;
use pulldown_cmark as md;
use std::collections::HashSet;
use crate::posts::post::Post;
use crate::renderer::templates::*;
use crate::mini::html::HtmlMinimizer;

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
    
    pub fn render(&self, buffer: &mut String, content: &[u8], post: &Post) {
        std::fs::create_dir_all(self.file.parent().unwrap()).unwrap();
        let content = std::str::from_utf8(&content[post.metadata().start_content()..]).unwrap();
        let mut minimizer = HtmlMinimizer::new(buffer);
        let mut options = md::Options::empty();
        options.insert(md::Options::ENABLE_TABLES);
        options.insert(md::Options::ENABLE_STRIKETHROUGH);
        let mut uses_code = false;
        let mut languages: HashSet<String> = HashSet::new();
        
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
            for elem in md::Parser::new_ext(content, options) {
                println!("{:?}", elem);
            }
        }
        
        #[cfg(feature = "test-content")]
        minimizer.append_template(TestContent {});
        
        minimizer.append_template(PostFooter {});
        minimizer.minimize(&self.file);
    }
}
