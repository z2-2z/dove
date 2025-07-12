use std::path::{PathBuf, Path};
use anyhow::Result;

mod engine;
mod posts;
mod fs;
mod transformer;
mod parser;

fn main() -> Result<()> {
    //TODO: change to postcard
    let mut cache = posts::PostCache::new("CACHE")?;
    let mut updated_posts = false;
    
    for input_file in posts::PostIterator::new("INPUT")? {
        let rerender = if let Some(entry) = cache.get(&input_file) {
            entry.dependencies().iter().any(|d| fs::is_newer(d.input(), d.output()))
        } else {
            true
        };
        
        if rerender {
            let mut input_basedir = input_file.clone();
            input_basedir.pop();
            let mut output_basedir;
            let post = posts::Post::new(&input_file)?;
            let mut renderer = engine::Renderer::new();
            let html_path;
            
            if let Some(filename) = post.filename() {
                // The order is important!
                let mut body = renderer.render_body(post.content(), &input_basedir)?.into_bytes();
                let mut header = renderer.render_header(&post)?.into_bytes();
                let mut footer = renderer.render_footer()?.into_bytes();
                
                /* Check languages  */
                for lang in renderer.languages_used() {
                    let path = format!("STATIC_FOLDER/js/hljs/{lang}.min.js");
                    let path = Path::new(&path);
                    
                    if !path.exists() {
                        anyhow::bail!("Language {lang} does not exist");
                    }
                }
                
                /* Render page */
                let output_file = format!("OUTPUT/{filename}");
                output_basedir = PathBuf::from(&output_file);
                output_basedir.pop();
                
                transformer::transform_buffer(&mut header, &output_file, true)?;
                transformer::transform_buffer(&mut body, &output_file, false)?;
                transformer::transform_buffer(&mut footer, &output_file, false)?;
                
                /* Copy file mentions */
                for path in renderer.file_mentions() {
                    transformer::transform_file(
                        input_basedir.join(path),
                        output_basedir.join(path)
                    )?;
                }
                
                html_path = output_file;
            } else {
                html_path = "OUTPUT/archive.html".to_string();
                output_basedir = PathBuf::from("OUTPUT");
            }
            
            cache.insert(
                &input_basedir,
                &input_file,
                &output_basedir,
                Path::new(&html_path),
                &post,
                &renderer,
            );
        }
        
        updated_posts |= rerender;
    }
    
    if updated_posts {
        // regenerate index, archive, feed completely from cache
        
        cache.save("CACHE")?;
    }
    
    // conditional recursive copy of static folder: only copy static source files that are newer
    
    Ok(())
}
