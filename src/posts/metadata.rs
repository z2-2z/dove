pub struct PostDate {
    day: u8,
    month: u8,
    year: u16,
}

pub struct PostMetadata {
    title: String,
    date: PostDate,
    authors: Vec<String>,
    categories: Vec<String>,
}

pub struct PostMetadataParser<'a> {
    cursor: usize,
    data: &'a [u8],
}

impl<'a> PostMetadataParser<'a> {
    pub fn parse(data: &'a [u8]) -> PostMetadata {
        let mut parser = Self {
            cursor: 0,
            data,
        };
        
        /* 1.) lines with metadata fields */
        while let Some(linebreak) = parser.find_linebreak() {
            if linebreak == parser.cursor {
                break;
            }
            
            parser.cursor = linebreak + 1;
        }
        
        /* 2.) empty line */
        parser.cursor += 1;
        
        /* 3.) title */
        
        
        todo!()
    }

    fn find_linebreak(&self) -> Option<usize> {
        let mut cursor = self.cursor;
        
        while let Some(byte) = self.data.get(cursor) {
            if *byte == b'\n' {
                return Some(cursor);
            }
            
            cursor += 1;
        }
        
        None
    }
}
