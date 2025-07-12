use anyhow::Result;

mod engine;
mod posts;
mod fs;
mod transformer;
mod parser;

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
            let mut input_basedir = input_file.clone();
            input_basedir.pop();
            let post = posts::Post::new(input_file)?;
            let mut renderer = engine::Renderer::new();
            let html_path;
            
            if let Some(filename) = post.filename() {
                // The order is important!
                let body = renderer.render_body(post.content(), &input_basedir)?;
                let header = renderer.render_header(&post)?;
                let footer = renderer.render_footer()?;
                
                let output_file = format!("OUTPUT/{filename}");
                
                // minify
                
                html_path = output_file;
            } else {
                html_path = "OUTPUT/archive.html".to_string();
            }
            
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
