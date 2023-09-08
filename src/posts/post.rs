use std::path::Path;
use std::fs::File;
use std::borrow::Borrow;
use memmap2::Mmap;
use crate::posts::metadata::{PostMetadataParser, PostMetadata, ParsingError};

pub const POST_EXTENSION: &str = "md";

fn map_file<P: AsRef<Path>>(path: P) -> Mmap {
    let file = File::open(path).unwrap();
    unsafe { Mmap::map(&file) }.unwrap()
}

fn encode_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_ascii())
        .map(|c| if c.is_ascii_alphanumeric() {
            c
        } else {
            '_'
        })
        .collect()
}

pub struct Post {
    id: String,
    metadata: PostMetadata,
}

impl Post {
    pub fn from_file(path: &Path) -> Result<Self, ParsingError> {
        let filename = path.file_prefix().unwrap().to_string_lossy();
        let content = map_file(path);
        let metadata = PostMetadataParser::parse(&content)?;
        
        Ok(Self {
            id: encode_id(filename.borrow()),
            metadata,
        })
    }
}

