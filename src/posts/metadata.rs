use std::path::{Path, PathBuf};
use crate::msg::{
    error_header, error_footer,
};

#[derive(Debug)]
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
    
    pub fn day(&self) -> u8 {
        self.day
    }
    
    pub fn month(&self) -> u8 {
        self.month
    }
    
    pub fn year(&self) -> u16 {
        self.year
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
    start_content: usize,
}

impl PostMetadata {
    pub fn title(&self) -> &str {
        &self.title
    }
    
    pub fn date(&self) -> &PostDate {
        &self.date
    }
    
    pub fn authors(&self) -> &[String] {
        &self.authors
    }
    
    pub fn categories(&self) -> &[String] {
        &self.categories
    }
    
    pub fn start_content(&self) -> usize {
        self.start_content
    }
}

pub struct PostMetadataParser<'a> {
    file: &'a Path,
    cursor: usize,
    data: &'a [u8],
    date: Option<PostDate>,
    authors: Vec<String>,
    categories: Vec<String>,
}

impl<'a> PostMetadataParser<'a> {
    pub fn parse(data: &'a [u8], file: &'a Path) -> Result<PostMetadata, ParsingError> {
        let mut parser = Self {
            file,
            cursor: 0,
            data,
            date: None,
            authors: Vec::new(),
            categories: Vec::new(),
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
        parser.skip_linebreaks();
        
        /* Collect parsed metadata */
        parser.check_authors()?;
        parser.check_categories()?;
        parser.check_content()?;
        let date = parser.date()?;
        let authors = parser.authors;
        let categories = parser.categories;
        let start_content = parser.cursor;
        
        Ok(PostMetadata {
            title,
            date,
            authors,
            categories,
            start_content,
        })
    }
    
    fn skip_linebreaks(&mut self) {
        let mut cursor = self.cursor;
        
        while let Some(byte) = self.data.get(cursor) {
            if *byte != b'\n' {
                break;
            }
            
            cursor += 1;
        }
        
        self.cursor = cursor;
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
        let cursor = std::cmp::min(self.cursor, self.data.len());
        
        for i in &self.data[0..cursor] {
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
            b"authors" => self.authors = self.parse_list(end)?,
            b"categories" => self.categories = self.parse_list(end)?,
            _ => return Err(self.parsing_error("Invalid metadata key")),
        }
        
        Ok(())
    }
    
    fn parse_date(&mut self, end: usize) -> Result<PostDate, ParsingError> {
        let line = &self.data[self.cursor..end];
        PostDate::parse(line).ok_or_else(|| self.parsing_error("Invalid date format"))
    }
    
    fn parse_list(&mut self, end: usize) -> Result<Vec<String>, ParsingError> {
        let mut ret = Vec::new();
        let mut cursor = self.cursor;
        
        while cursor < end {
            /* Skip whitespace */
            while cursor < end && self.data[cursor] == b' ' {
                cursor += 1;
            }
            
            let start = cursor;
            
            /* Find next separator */
            while cursor < end && self.data[cursor] != b',' {
                cursor += 1;
            }
            
            /* Extract current item */
            let mut end = cursor;
            
            while start < end && self.data[end - 1] == b' ' {
                end -= 1;
            }
            
            if start == end {
                return Err(self.parsing_error("Empty author name"));
            }
            
            let item = std::str::from_utf8(&self.data[start..end]).map_err(|_| self.parsing_error("List item contains invalid characters"))?;
            ret.push(item.to_string());
            
            /* Skip separator */
            cursor += 1;
        }
        
        self.cursor = cursor;
        Ok(ret)
    }
    
    fn date(&self) -> Result<PostDate, ParsingError> {
        self.date.as_ref().cloned().ok_or_else(|| self.parsing_error("Post date was not set"))
    }
    
    fn check_authors(&self) -> Result<(), ParsingError> {
        if self.authors.is_empty() {
            Err(self.parsing_error("No authors were specified in post"))
        } else {
            Ok(())
        }
    }
    
    fn check_categories(&self) -> Result<(), ParsingError> {
        if self.authors.is_empty() {
            Err(self.parsing_error("No categories were specified in post"))
        } else {
            Ok(())
        }
    }
    
    fn check_content(&self) -> Result<(), ParsingError> {
        if self.cursor >= self.data.len() {
            Err(self.parsing_error("No content in post"))
        } else {
            Ok(())
        }
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
    
    #[test]
    fn list_empty() {
        assert!(PostMetadataParser::parse(b"authors: a,   ,b\n", Path::new("<test>")).is_err());
    }
    
    #[test]
    fn test_metadata() {
        let metadata = PostMetadataParser::parse(b"date: 01-02-1970\nauthors: Me , you , John Doe \ncategories: A, B, C\n\n# Title \n\ncontent", Path::new("<test>")).unwrap();
        assert_eq!(metadata.date(), &PostDate {
            day: 1,
            month: 2,
            year: 1970
        });
        assert_eq!(metadata.title(), "Title ");
        assert_eq!(
            metadata.authors(),
            ["Me", "you", "John Doe"]
        );
        assert_eq!(
            metadata.categories(),
            ["A", "B", "C"]
        );
    }
}
