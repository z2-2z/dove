use std::path::Path;
use crate::msg::{
    error_header, error_footer,
};

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
    file: &'a Path,
    cursor: usize,
    data: &'a [u8],
    date: Option<PostDate>,
}

impl<'a> PostMetadataParser<'a> {
    pub fn parse(data: &'a [u8], file: &'a Path) -> Option<PostMetadata> {
        let mut parser = Self {
            file,
            cursor: 0,
            data,
            date: None,
        };
        
        /* 1.) lines with metadata fields */
        while let Some(linebreak) = parser.find_linebreak() {
            if linebreak == parser.cursor {
                break;
            }
            
            parser.parse_metadata(linebreak)?;
            
            parser.cursor = linebreak + 1;
        }
        
        /* 2.) empty line */
        parser.cursor += 1;
        
        /* 3.) title */
        if parser.data.get(parser.cursor).copied() != Some(b'#') {
            
        }
        
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
    
    fn find_line_number(&self, pos: usize) -> usize {
        let mut lno = 1;
        
        for i in &self.data[0..pos] {
            if *i == b'\n' {
                lno += 1;
            }
        }
        
        lno
    }
    
    fn throw_error<S: AsRef<str>>(&self, pos: usize, len: usize, msg: S) -> Option<()> {
        let lno = format!("{}", self.find_line_number(pos));
        
        error_header("Parsing Error");
        
        eprintln!("In file {}: {}", self.file.display(), msg.as_ref());
        let subpart = std::str::from_utf8(&self.data[pos..pos + len]).unwrap();
        eprintln!("Line {}: {}", lno, subpart);
        eprintln!("     {3: <2$}  {1:^<0$}", len, "", lno.len(), "");
        
        error_footer("Parsing Error");
        
        None
        
    }
    
    fn parse_metadata(&self, end: usize) -> Option<()> {
        /* Get to the first colon */
        let mut colon = self.cursor;
        
        while colon < end && self.data[colon] != b':' {
            colon += 1;
        }
        
        if colon == end {
            return self.throw_error(self.cursor, end - self.cursor, "Invalid metadata: Missing colon");
        }
        
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn missing_colon() {
        assert!(PostMetadataParser::parse(b"missing colon\n", Path::new("<test>")).is_none());
    }
}
