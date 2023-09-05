use std::path::Path;
use std::fs::File;
use std::borrow::Borrow;
use memmap2::Mmap;
use crate::posts::metadata::PostMetadataParser;

pub const POST_EXTENSION: &str = "md";

fn map_file<P: AsRef<Path>>(path: P) -> Mmap {
    let file = File::open(path).unwrap();
    unsafe { Mmap::map(&file) }.unwrap()
}

pub struct Post {
    id: String,
    start_content: usize,
}

impl Post {
    pub fn from_file(path: &Path) -> Self {
        let id: &str = path.file_prefix().unwrap().to_string_lossy().borrow();
        let content = map_file(path);
        let parser = PostMetadataParser::parse(&content, path);
        
        todo!()
    }
}

