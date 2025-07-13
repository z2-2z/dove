use anyhow::Result;
use curl::easy::Easy;

pub fn http_url_exists(url: &str) -> Result<bool> {
    if url.starts_with("http://") || url.starts_with("https://") {
        let mut easy = Easy::new();
        easy.url(url)?;
        easy.follow_location(true)?;
        easy.nobody(true)?;
        
        match easy.perform() {
            Ok(_) => {},
            Err(_) => return Ok(false),
        }
        
        let code = easy.response_code()?;
        Ok(code < 400)
    } else {
        Ok(true)
    }
}
