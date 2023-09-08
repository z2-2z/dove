use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::BufWriter;
use pulldown_cmark as md;
use crate::posts::post::Post;

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
        let output_file = File::create(&self.file).unwrap();
        let output_file = BufWriter::new(output_file);
        
        let content = std::str::from_utf8(&content[post.metadata().start_content()..]).unwrap();
        
        let mut options = md::Options::empty();
        options.insert(md::Options::ENABLE_TABLES);
        options.insert(md::Options::ENABLE_STRIKETHROUGH);
        
        for elem in md::Parser::new_ext(content, options) {
            println!("{:?}", elem);
        }
    }
}
