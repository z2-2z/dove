use std::path::{PathBuf, Path};
use anyhow::Result;
use clap::Parser;

mod engine;
mod posts;
mod fs;
mod transformer;
mod parser;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Render {
        #[arg(short, long, required = true, value_name = "DIR")]
        input: String,
        
        #[arg(short, long, required = true, value_name = "DIR")]
        output: String,
        
        #[arg(short, long, required = true, value_name = "FILE")]
        cache: String,
        
        #[arg(short, long)]
        force: bool,
        
        #[arg(long, default_value_t = String::from("./static"))]
        static_folder: String,
    },
    
    New {
        output: String,
    },
}

fn render(input_dir: String, output_dir: String, cache_file: String, force: bool, static_folder: String) -> Result<()> {
    /* Copy static files */
    fs::copy_dir_recursive(
        force,
        &static_folder,
        &output_dir,
    )?;
    
    let out_404 = PathBuf::from(format!("{output_dir}/404.html"));
    if force || !out_404.exists() {
        let mut output = engine::render_404()?.into_bytes();
        transformer::transform_buffer(&mut output, out_404)?;
    }
    
    /* Read posts */
    //TODO: change to postcard
    let mut cache = posts::PostCache::new(&cache_file)?;
    let mut updated_posts = false;
    
    if force {
        cache.clear();
    }
    
    for input_file in posts::PostIterator::new(&input_dir)? {
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
                let mut body = renderer.render_body(post.content(), &input_basedir)?.into_bytes();
                
                /* Check languages  */
                for lang in renderer.languages_used() {
                    let path = format!("{static_folder}/js/hljs/{lang}.min.js");
                    let path = Path::new(&path);
                    
                    if !path.exists() {
                        anyhow::bail!("Language {lang} does not exist");
                    }
                }
                
                let mut header = renderer.render_header(&post)?.into_bytes();
                let mut footer = renderer.render_footer()?.into_bytes();
                
                /* Render page */
                let output_file = format!("{output_dir}/{filename}");
                output_basedir = PathBuf::from(&output_file);
                output_basedir.pop();
                if !output_basedir.exists() {
                    std::fs::create_dir_all(&output_basedir)?;
                }
                
                header.reserve(body.len() + footer.len());
                header.append(&mut body);
                header.append(&mut footer);
                
                transformer::transform_buffer(&mut header, &output_file)?;
                
                /* Copy file mentions */
                for path in renderer.file_mentions() {
                    transformer::transform_file(
                        input_basedir.join(path),
                        output_basedir.join(path)
                    )?;
                }
                
                html_path = output_file;
            } else {
                html_path = format!("{output_dir}/archive.html");
                output_basedir = PathBuf::from(&output_dir);
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
        let mut entries: Vec<&posts::CacheEntry> = cache.resources().collect();
        entries.sort_by(|a, b| b.metadata().date().cmp(a.metadata().date()));
        
        /* Render index */
        let mut output = engine::render_index(&entries)?.into_bytes();
        transformer::transform_buffer(&mut output, format!("{output_dir}/index.html"))?;
        
        /* Render archive */
        let mut output = engine::render_archive(&entries)?.into_bytes();
        transformer::transform_buffer(&mut output, format!("{output_dir}/archive.html"))?;
        
        /* Render feed */
        let mut output = engine::render_feed(&entries)?.into_bytes();
        transformer::transform_buffer(&mut output, format!("{output_dir}/atom.xml"))?;
        
        cache.save(&cache_file)?;
    }
    
    Ok(())
}

fn main() {
    let args = Args::parse();
    
    match args.command {
        Commands::Render { input, output, cache, force, static_folder } => render(input, output, cache, force, static_folder).unwrap(),
        Commands::New { output } => todo!(),
    }
}
