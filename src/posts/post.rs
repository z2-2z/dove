use crate::posts::metadata::{PostMetadataParser, PostMetadata, ParsingError};

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
    url: String,
}

impl Post {
    pub fn new(content: &[u8]) -> Result<Self, ParsingError> {
        let metadata = PostMetadataParser::parse(content)?;
        let url = format!(
            "{:04}/{}/{:02}/{}",
            metadata.date().year(),
            month_name(metadata.date().month()),
            metadata.date().day(),
            encode_filename(metadata.title())
        );
        
        Ok(Self {
            metadata,
            url,
        })
    }
    
    pub fn metadata(&self) -> &PostMetadata {
        &self.metadata
    }
    
    pub fn url(&self) -> &str {
        &self.url
    }
}
