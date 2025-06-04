#[derive(Debug)]
pub struct ParsingError {
    line: usize,
    message: String,
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parsing Error in line {}: {}", self.line, self.message)
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostDate {
    year: u16,
    month: u8,
    day: u8,
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
    
    pub fn year(&self) -> u16 {
        self.year
    }
    
    pub fn month(&self) -> u8 {
        self.month
    }
    
    pub fn month_name(&self) -> &'static str {
        match self.month {
            1 => "jan.",
            2 => "feb.",
            3 => "mar.",
            4 => "apr.",
            5 => "may",
            6 => "jun.",
            7 => "jul.",
            8 => "aug.",
            9 => "sep.",
            10 => "oct.",
            11 => "nov.",
            12 => "dec.",
            _ => unreachable!(),
        }
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
    categories: Vec<String>,
    mirror: Option<String>,
    #[allow(dead_code)]
    startpage: bool,
    start_content: usize,
}

impl PostMetadata {
    pub fn title(&self) -> &str {
        &self.title
    }
    
    pub fn date(&self) -> &PostDate {
        &self.date
    }
    
    pub fn categories(&self) -> &[String] {
        &self.categories
    }
    
    pub fn start_content(&self) -> usize {
        self.start_content
    }
    
    pub fn mirror(&self) -> Option<&String> {
        self.mirror.as_ref()
    }
    
    pub fn startpage(&self) -> bool {
        self.startpage
    }
}

pub struct PostMetadataParser<'a> {
    cursor: usize,
    data: &'a [u8],
    date: Option<PostDate>,
    mirror: Option<String>,
    categories: Vec<String>,
    startpage: bool,
}

impl<'a> PostMetadataParser<'a> {
    pub fn parse(data: &'a [u8]) -> Result<PostMetadata, ParsingError> {
        let mut parser = Self {
            cursor: 0,
            data,
            date: None,
            mirror: None,
            categories: Vec::new(),
            startpage: false,
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
        parser.check_categories()?;
        parser.check_content()?;
        let date = parser.date()?;
        let categories = parser.categories;
        let mirror = parser.mirror;
        let startpage = parser.startpage;
        let start_content = parser.cursor;
        
        Ok(PostMetadata {
            title,
            date,
            categories,
            mirror,
            startpage,
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
            b"categories" => self.categories = self.parse_list(end)?,
            b"mirror" => self.mirror = self.parse_mirror(end)?,
            b"startpage" => self.startpage = self.parse_startpage(end)?,
            _ => return Err(self.parsing_error("Invalid metadata key")),
        }
        
        Ok(())
    }
    
    fn parse_startpage(&mut self, end: usize) -> Result<bool, ParsingError> {
        let value = &self.data[self.cursor..end];
        match value {
            b"false" => Ok(false),
            b"true" => Ok(true),
            _ => Err(self.parsing_error("Invalid boolean value"))
        }
    }
    
    fn parse_mirror(&mut self, end: usize) -> Result<Option<String>, ParsingError> {
        let url = &self.data[self.cursor..end];
        let url = std::str::from_utf8(url).map_err(|_| self.parsing_error("Invalid mirror URL"))?;
        Ok(Some(url.to_string()))
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
    
    fn check_categories(&self) -> Result<(), ParsingError> {
        if self.categories.is_empty() {
            Err(self.parsing_error("No categories were specified in post"))
        } else {
            Ok(())
        }
    }
    
    fn check_content(&self) -> Result<(), ParsingError> {
        if self.mirror.is_some() {
            Ok(())
        } else if self.cursor >= self.data.len() {
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
        assert!(PostMetadataParser::parse(b"   missing colon\n").is_err());
    }
    
    #[test]
    fn invalid_key() {
        assert!(PostMetadataParser::parse(b"   x: y\n").is_err());
    }
    
    #[test]
    fn list_empty() {
        assert!(PostMetadataParser::parse(b"categories: a,   ,b\n").is_err());
    }
    
    #[test]
    fn test_metadata() {
        let metadata = PostMetadataParser::parse(b"date: 01-02-1970\ncategories: A, B, C\n\n# Title \n\ncontent").unwrap();
        assert_eq!(metadata.date(), &PostDate {
            day: 1,
            month: 2,
            year: 1970
        });
        assert_eq!(metadata.title(), "Title ");
        assert_eq!(
            metadata.categories(),
            ["A", "B", "C"]
        );
    }
}
