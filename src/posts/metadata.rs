use std::path::{Path, PathBuf};
use crate::msg::{
    error_header, error_footer,
};

pub struct ParsingError {
    line: usize,
    path: PathBuf,
    message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostDate {
    day: u8,
    month: u8,
    year: u16,
}

impl PostDate {
    fn parse(data: &[u8]) -> Option<Self> {
        /* First number */
        let mut cursor = 0;
        
        while let Some(byte) = data.get(cursor) {
            if !byte.is_ascii_digit() {
                break;
            }
            
            cursor += 1;
        }
        
        let day: u8 = std::str::from_utf8(&data[0..cursor]).ok()?.parse().ok()?;
        
        if day == 0 || day > 31 {
            return None;
        }
        
        /* Separator */
        if data.get(cursor).copied() != Some(b'-') {
            return None;
        }
        cursor += 1;
        
        /* Second number */
        let start_number = cursor;
        
        while let Some(byte) = data.get(cursor) {
            if !byte.is_ascii_digit() {
                break;
            }
            
            cursor += 1;
        }
        
        let month: u8 = std::str::from_utf8(&data[start_number..cursor]).ok()?.parse().ok()?;
        
        if month == 0 || month > 12 {
            return None;
        }
        
        /* Separator */
        if data.get(cursor).copied() != Some(b'-') {
            return None;
        }
        cursor += 1;
        
        /* Third number */
        let start_number = cursor;
        
        while let Some(byte) = data.get(cursor) {
            if !byte.is_ascii_digit() {
                break;
            }
            
            cursor += 1;
        }
        
        let year: u16 = std::str::from_utf8(&data[start_number..cursor]).ok()?.parse().ok()?;
        
        if !(1970..=9999).contains(&year) {
            return None;
        }
        
        /* End of line */
        if cursor < data.len() {
            return None;
        }
        
        Some(Self {
            day,
            month,
            year,
        })
    }
}

#[cfg(test)]
mod postdate_tests {
    use super::*;
    
    #[test]
    fn working() {
        let date = PostDate::parse(b"01-02-1970").unwrap();
        assert_eq!(
            date,
            PostDate {
                day: 1,
                month: 2,
                year: 1970,
            }
        );
    }
    
    #[test]
    fn missing_fields() {
        assert!(PostDate::parse(b"-02-1970").is_none());
        assert!(PostDate::parse(b"01--1970").is_none());
        assert!(PostDate::parse(b"01-02-").is_none());
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
        
        /* Lines with metadata fields */
        while let Some(linebreak) = parser.find_next_stop() {
            if linebreak == parser.cursor {
                break;
            }
            
            parser.parse_metadata(linebreak)?;
            
            parser.cursor = linebreak + 1;
        }
        
        /* Empty line */
        parser.cursor += 1;
        
        /* Title */
        let title = parser.parse_title()?;
        
        /* Collect parsed metadata */
        let date = parser.date()?;
        
        Ok(PostMetadata {
            title,
            date,
            authors: todo!(),
            categories: todo!(),
        })
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
    
    fn parsing_error<S: Into<String>>(&self, message: S) -> ParsingError {
        ParsingError {
            line: self.find_line_number(),
            path: self.file.to_path_buf(),
            message: message.into(),
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
    
    fn parse_title(&mut self) -> Result<String, ParsingError> {
        if self.data.get(self.cursor).copied() != Some(b'#') {
            return Err(self.parsing_error("Expected title (h1) after metadata lines"));
        }
        
        self.cursor += 1;
        
        if self.data.get(self.cursor).copied() != Some(b' ') {
            return Err(self.parsing_error("Expected whitespace after hash"));
        }
        
        self.skip_whitespaces();
        
        let end = self.find_next_stop().ok_or_else(|| self.parsing_error("No title content given"))?;
        
        let title = std::str::from_utf8(&self.data[self.cursor..end]).map_err(|_| self.parsing_error("Title contains invalid characters"))?;
        
        self.cursor = end + 1;
        
        Ok(title.to_string())
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
    
    fn date(&self) -> Result<PostDate, ParsingError> {
        self.date.as_ref().cloned().ok_or_else(|| self.parsing_error("Post date was not set"))
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
