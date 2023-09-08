use std::path::{Path, PathBuf};
use crate::msg::{
    error_header, error_footer,
};

pub struct ParsingError {
    line: usize,
    path: PathBuf,
    message: &'static str,
}

pub struct PostDate {
    day: u8,
    month: u8,
    year: u16,
}

impl PostDate {
    fn parse(data: &[u8]) -> Option<Self> {
        
        todo!()
    }
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
    pub fn parse(data: &'a [u8], file: &'a Path) -> Result<PostMetadata, ParsingError> {
        let mut parser = Self {
            file,
            cursor: 0,
            data,
            date: None,
        };
        
        /* 1.) lines with metadata fields */
        while let Some(linebreak) = parser.find_next_stop() {
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

    fn find_next_stop(&self) -> Option<usize> {
        let mut cursor = self.cursor;
        
        while let Some(byte) = self.data.get(cursor) {
            if *byte == b'\n' {
                return Some(cursor);
            }
            
            cursor += 1;
        }
        
        if cursor == self.cursor{
            None
        } else {
            Some(cursor)
        }
    }
    
    fn find_line_number(&self) -> usize {
        let mut lno = 1;
        
        for i in &self.data[0..self.cursor] {
            if *i == b'\n' {
                lno += 1;
            }
        }
        
        lno
    }
    
    fn parsing_error(&self, message: &'static str) -> ParsingError {
        ParsingError {
            line: self.find_line_number(),
            path: self.file.to_path_buf(),
            message,
        }
    }
    
    fn skip_whitespaces(&mut self) {
        let mut cursor = self.cursor;
        
        while let Some(byte) = self.data.get(cursor) {
            if !matches!(*byte, b' ') {
                break;
            }
            
            cursor += 1;
        }
        
        self.cursor = cursor;
    }
    
    fn parse_metadata(&mut self, end: usize) -> Result<(), ParsingError> {
        self.skip_whitespaces();
        
        let key_start = self.cursor;
        let mut colon = self.cursor;
        
        while colon < end && self.data[colon] != b':' {
            colon += 1;
        }
        
        if colon == end {
            return Err(self.parsing_error("Invalid metadata line: Missing colon"));
        }
        
        self.cursor = colon + 1;
        self.skip_whitespaces();
        
        match &self.data[key_start..colon] {
            b"date" => self.date = Some(self.parse_date(end)?),
            b"authors" => todo!(),
            b"categories" => todo!(),
            _ => return Err(self.parsing_error("Invalid metadata key")),
        }
        
        Ok(())
    }
    
    fn parse_date(&mut self, end: usize) -> Result<PostDate, ParsingError> {
        let line = &self.data[self.cursor..end];
        PostDate::parse(line).ok_or_else(|| self.parsing_error("Invalid date format"))
    }
    
    
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn missing_colon() {
        assert!(PostMetadataParser::parse(b"   missing colon\n", Path::new("<test>")).is_err());
    }
    
    #[test]
    fn invalid_key() {
        assert!(PostMetadataParser::parse(b"   x: y\n", Path::new("<test>")).is_err());
    }
}
