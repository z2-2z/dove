use anyhow::Result;

mod posts;
mod fs;

fn main() -> Result<()> {
    let mut cache = posts::PostCache::new("CACHE")?;
    let mut updated_posts = false;
    
    for input_file in posts::PostIterator::new("INPUT")? {
        let rerender = if let Some(entries) = cache.get(&input_file) {
            // if any input is newer than output in entries, rerender
            todo!()
        } else {
            true
        };
        
        if rerender {
            let post = posts::Post::new(input_file)?;
            
            
            
            // update cache entry
        }
        
        updated_posts |= rerender;
    }
    
    if updated_posts {
        // regenerate index, archive, feed completely from cache
        
        // save cache
    }
    
    // conditional recursive copy of static folder: only copy static source files that are newer
    
    Ok(())
}
