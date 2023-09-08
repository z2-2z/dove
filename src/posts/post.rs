use std::path::PathBuf;
use std::io::BufWriter;
use std::fs::File;
use crate::posts::metadata::{PostMetadataParser, PostMetadata, ParsingError};

pub const POST_EXTENSION: &str = "md";

fn month_name(month: u8) -> &'static str {
    match month {
        1 => "jan",
        2 => "feb",
        3 => "mar",
        4 => "apr",
        5 => "may",
        6 => "jun",
        7 => "jul",
        8 => "aug",
        9 => "sep",
        10 => "oct",
        11 => "nov",
        12 => "dec",
        _ => unreachable!(),
    }
}

fn encode_filename(id: &str) -> String {
    id.chars()
        .map(|c| if !c.is_ascii() || c.is_ascii_alphanumeric() {
            c
        } else {
            '_'
        })
        .map(|c| if c.is_ascii() {
            c
        } else {
            '@'
        })
        .chain(".html".chars())
        .collect()
}

pub struct Post {
    metadata: PostMetadata,
    directory: String,
    filename: String,
}

impl Post {
    pub fn new(content: &[u8]) -> Result<Self, ParsingError> {
        let metadata = PostMetadataParser::parse(content)?;
        let directory = format!(
            "{:04}/{}/{:02}",
            metadata.date().year(),
            month_name(metadata.date().month()),
            metadata.date().day(),
        );
        let filename = encode_filename(metadata.title());
        
        Ok(Self {
            metadata,
            directory,
            filename,
        })
    }
    
    pub fn generate_html(&self, output: &str, content: &[u8]) {
        let mut path = PathBuf::from(output);
        path.push(&self.directory);
        std::fs::create_dir_all(&path).unwrap();
        
        path.push(&self.filename);
        let output_file = File::create(&path).unwrap();
        let output_file = BufWriter::new(output_file);
        
        let content = &content[self.metadata.start_content()..];
    }
}

