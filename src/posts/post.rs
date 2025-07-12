use memmap2::Mmap;
use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::parser::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PostDate {
    year: u16,
    month: u8,
    day: u8,
}

impl PostDate {
    pub fn day(&self) -> u8 {
        self.day
    }
    
    pub fn month(&self) -> u8 {
        self.month
    }
    
    pub fn month_name(&self) -> &'static str {
        match self.month {
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
    
    pub fn year(&self) -> u16 {
        self.year
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PostMetadata {
    date: PostDate,
    categories: Vec<String>,
    startpage: bool,
    title: String,
}

impl PostMetadata {
    pub fn date(&self) -> &PostDate {
        &self.date
    }
    
    pub fn categories(&self) -> &[String] {
        &self.categories
    }
    
    pub fn startpage(&self) -> bool {
        self.startpage
    }
    
    pub fn title(&self) -> &str {
        &self.title
    }
}

#[derive(Default)]
struct Parser {
    metadata: PostMetadata,
    content_offset: usize,
    mirror: Option<String>,
}

impl Parser {
    fn parse(&mut self, file: &[u8]) -> Result<()> {
        let mut line = 1;
        let mut cursor = 0;
        let mut cont = true;
        
        while cont {
            let mut end = cursor;
            
            loop {
                if let Some(c) = file.get(end) {
                    if *c == b'\n' {
                        break;
                    } else {
                        end += 1;
                    }
                } else {
                    anyhow::bail!("Parsing error in line {}: Premature end of file", line);
                }
            }
            
            match self.parse_line(&file[cursor..end]) {
                Ok(c) => cont = c,
                Err(msg) => anyhow::bail!("Parsing error in line {}: {}", line, msg),
            }
            
            cursor = end + 1;
            line += 1;
        }
        
        self.content_offset = cursor;
        
        self.check(file)
    }
    
    fn check(&self, file: &[u8]) -> Result<()> {
        /* Required fields */
        if self.metadata.date.day == 0 &&
                self.metadata.date.month == 0 &&
                self.metadata.date.year == 0 {
            anyhow::bail!("Metadata missing: date");    
        }
        if self.metadata.categories.is_empty() {
            anyhow::bail!("Metadata missing: categories");
        }
        
        /* Title constraints */
        if self.metadata.title.len() > 128 {
            anyhow::bail!("Title too long");
        }
        
        /* Mirror constraints */
        if self.mirror.is_some() && self.content_offset < file.len() {
            anyhow::bail!("Post with mirror attribute set has trailing data");
        }
        
        //TODO: warn if mirror URL gives 404 ?
        
        Ok(())
    }
    
    fn parse_line(&mut self, line: &[u8]) -> Result<bool, String> {
        if let Some(c) = line.first() {
            if *c == b'#' {
                /* Parse title */
                let title = trim_whitespaces(&line[1..]);
                
                if title.is_empty() {
                    return Err("Title cannot be empty".to_string());
                }
                
                let title = std::str::from_utf8(title).map_err(|_| "Title is not UTF-8")?;
                
                self.metadata.title = title.to_owned();
                return Ok(false);
            }
        } else {
            return Ok(true);
        }
        
        /* Parse key-value pair */
        let (key, value) = split_once(line, b':');
        let value = trim_whitespaces(value);
        
        if key.is_empty() || value.is_empty() {
            return Err("Invalid key-value pair".to_string());
        }
        
        if trim_whitespaces(key).len() != key.len() {
            return Err("Post metadata keys must not be surrounded by whitespaces".to_string());
        }
        
        match key {
            b"date" => self.parse_date(value)?,
            b"categories" => self.parse_categories(value)?,
            b"startpage" => self.parse_startpage(value)?,
            b"mirror" => self.parse_mirror(value)?,
            _ => return Err("Invalid metadata. Attribute does not exist".to_string()),
        }
        
        Ok(true)
    }
    
    fn parse_date(&mut self, value: &[u8]) -> Result<(), String> {
        const SEPARATOR: u8 = b'.';
        
        let (day, rest) = split_once(value, SEPARATOR);
        let (month, year) = split_once(rest, SEPARATOR);
        
        if day.len() != 2 || month.len() != 2 || year.len() != 4 {
            return Err("Invalid date format. Must be DD.MM.YYYY".to_string());
        }
        
        if !is_numerical(day) {
            return Err("Invalid day".to_string());
        }
        if !is_numerical(month) {
            return Err("Invalid month".to_string());
        }
        if !is_numerical(year) {
            return Err("Invalid year".to_string());
        }
        
        self.metadata.date.day = convert_number(day) as u8;
        self.metadata.date.month = convert_number(month) as u8;
        self.metadata.date.year = convert_number(year) as u16;
        
        if !(1..=31).contains(&self.metadata.date.day) {
            return Err("Invalid day".to_string());
        }
        if !(1..=12).contains(&self.metadata.date.month) {
            return Err("Invalid month".to_string());
        }
        if !(0..=9999).contains(&self.metadata.date.year) {
            return Err("Invalid year".to_string());
        }
        
        Ok(())
    }
    
    fn parse_categories(&mut self, value: &[u8]) -> Result<(), String> {
        const SEPARATOR: u8 = b',';
        let charsets = [b'a'..=b'z', b'0'..=b'9', b'-'..=b'-'];
        
        let (mut cat, mut rest) = split_once(value, SEPARATOR);
        
        while !cat.is_empty() {
            cat = trim_whitespaces(cat);
            
            if cat.is_empty() {
                return Err("Invalid syntax".to_string());
            }
            
            for c in cat {
                if charsets.iter().all(|r| !r.contains(c)) {
                    return Err("Category name contains invalid characters".to_string());
                }
            }
            
            self.metadata.categories.push(std::str::from_utf8(cat).unwrap().to_owned());
            
            (cat, rest) = split_once(rest, SEPARATOR);
        }
        
        if self.metadata.categories.is_empty() {
            return Err("No categories given".to_string());
        }
        
        Ok(())
    }
    
    fn parse_startpage(&mut self, value: &[u8]) -> Result<(), String> {
        match value {
            b"false" => self.metadata.startpage = false,
            b"true" => self.metadata.startpage = true,
            _ => return Err("Invalid boolean value".to_string()),
        }
        
        Ok(())
    }
    
    fn parse_mirror(&mut self, value: &[u8]) -> Result<(), String> {
        let value = std::str::from_utf8(value).map_err(|_| "Invalid characters in URL".to_string())?;
        self.mirror = Some(value.to_owned());
        Ok(())
    }
}

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

#[derive(Debug)]
pub struct Post {
    metadata: PostMetadata,
    url: String,
    filename: Option<String>,
    content_offset: usize,
    file: Mmap,
}

impl Post {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = crate::fs::mmap_file(path)?;
        let mut parser = Parser::default();
        parser.parse(&file)?;
        
        let filename;
        let url;
        
        if let Some(mirror) = parser.mirror {
            url = mirror;
            filename = None;
        } else {
            let encoded = format!(
                "{:04}/{}/{:02}/{}",
                parser.metadata.date.year,
                parser.metadata.date.month_name(),
                parser.metadata.date.day,
                encode_filename(&parser.metadata.title)
            );
            url = format!("/{encoded}");
            filename = Some(encoded);
        }
        
        Ok(Self {
            metadata: parser.metadata,
            url,
            filename,
            content_offset: parser.content_offset,
            file,
        })
    }
    
    pub fn metadata(&self) -> &PostMetadata {
        &self.metadata
    }
    
    pub fn url(&self) -> &str {
        &self.url
    }
    
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }
    
    pub fn content(&self) -> &[u8] {
        &self.file[self.content_offset..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn print_mirror() {
        let post = Post::new("test-data/postmeta/mirror.md").unwrap();
        println!("{post:#?}");
    }
    
    #[test]
    fn print_title() {
        let post = Post::new("test-data/postmeta/title.md").unwrap();
        println!("{post:#?}");
        println!("{:?}", post.content());
    }
}
