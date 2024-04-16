use crate::posts::metadata::{PostMetadataParser, PostMetadata, ParsingError};

fn encode_filename(id: &str) -> String {
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
        .chain(".html".chars())
        .collect()
}

pub struct Post {
    metadata: PostMetadata,
    url: String,
    filename: String,
}

impl Post {
    pub fn new(content: &[u8]) -> Result<Self, ParsingError> {
        let metadata = PostMetadataParser::parse(content)?;
        let filename = format!(
            "{:04}/{}/{:02}/{}",
            metadata.date().year(),
            metadata.date().month_name(),
            metadata.date().day(),
            encode_filename(metadata.title())
        );
        let url = if let Some(mirror) = metadata.mirror() {
            mirror.clone()
        } else {
            format!("/{}", filename)
        }; 
        
        Ok(Self {
            metadata,
            url,
            filename,
        })
    }
    
    pub fn metadata(&self) -> &PostMetadata {
        &self.metadata
    }
    
    pub fn filename(&self) -> &str {
        &self.filename
    }
    
    pub fn url(&self) -> &str {
        &self.url
    }
    
    pub fn headless(&self) -> bool {
        self.metadata.mirror().is_some()
    }
}
