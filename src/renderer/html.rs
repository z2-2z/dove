use std::path::{PathBuf, Path};
use pulldown_cmark as md;
use crate::posts::post::Post;
use crate::renderer::templates::*;
use crate::renderer::minimize::MinimizerRenderer;

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
    
    pub fn needs_updating(&self, source_file: &Path) -> bool {
        !self.file.exists() ||
        source_file.metadata().unwrap().modified().unwrap() > self.file.metadata().unwrap().modified().unwrap()
    }
    
    pub fn render(&self, content: &[u8], post: &Post) {
        std::fs::create_dir_all(self.file.parent().unwrap()).unwrap();
        let content = std::str::from_utf8(&content[post.metadata().start_content()..]).unwrap();
        let mut minimizer = MinimizerRenderer::new();
        let mut options = md::Options::empty();
        options.insert(md::Options::ENABLE_TABLES);
        options.insert(md::Options::ENABLE_STRIKETHROUGH);
        
        minimizer.append_template(PostHeader {});
        
        for elem in md::Parser::new_ext(content, options) {
            println!("{:?}", elem);
        }
        
        minimizer.append_template(PostFooter {});
        minimizer.minimize(&self.file);
    }
}
