use std::path::{PathBuf, Path};
use anyhow::Result;
use clap::Parser;

mod engine;
mod posts;
mod fs;
mod transformer;
mod parser;
mod net;

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
        
        #[arg(short, long)]
        offline: bool,
        
        #[arg(short, long)]
        live: bool,
        
        #[arg(long, default_value_t = String::from("./static"))]
        static_folder: String,
    },
    
    New {
        output: String,
    },
}

#[allow(clippy::too_many_arguments)]
fn render(input_dir: &str, output_dir: &str, cache_file: &str, force: bool, live: bool, offline: bool, static_folder: &str, watcher: &mut fs::FileWatcher) -> Result<()> {
    /* Copy static files */
    fs::copy_dir_recursive(
        force,
        static_folder,
        output_dir,
    )?;
    
    let out_404 = PathBuf::from(format!("{output_dir}/404.html"));
    if force || !out_404.exists() {
        let mut output = engine::render_404()?.into_bytes();
        transformer::transform_buffer(&mut output, out_404)?;
    }
    
    /* Read posts */
    let mut cache = posts::PostCache::new(cache_file)?;
    let mut cache_changed = false;
    
    if force {
        cache.clear();
    }
    
    loop {
        for input_file in posts::PostIterator::new(input_dir)? {
            let rerender = if let Some(entry) = cache.get(&input_file) {
                entry.dependencies().iter().any(|d| fs::is_newer(d.input(), d.output()))
            } else {
                true
            };
            
            if rerender {
                if live {
                    println!("[{}] Rendering {}...", chrono::Local::now().format("%H:%M:%S"), input_file.display());
                } else {
                    println!("Rendering {}...", input_file.display());
                }
                
                let mut input_basedir = input_file.clone();
                input_basedir.pop();
                let mut output_basedir;
                let post = posts::Post::new(&input_file, offline)?;
                let mut renderer = engine::Renderer::new(&input_basedir, offline);
                let html_path;
                
                if let Some(filename) = post.filename() {
                    let mut body = renderer.render_body(post.content())?.into_bytes();
                    
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
                    
                    //TODO: optimize this
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
                
                cache_changed |= cache.insert(
                    &input_basedir,
                    &input_file,
                    &output_basedir,
                    Path::new(&html_path),
                    &post,
                    &renderer,
                );
            }
        }
        
        if cache_changed {
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
            
            cache.save(cache_file)?;
        }
        
        if live {
            watcher.wait()?;
        } else {
            break;
        }
    }
    
    Ok(())
}

fn new(output: String) -> Result<()> {
    let content = include_str!("new-template.md");
    std::fs::write(output, content)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Render { input, output, cache, force, live, offline, static_folder } => {
            let mut watcher = fs::FileWatcher::new(&input)?;
            
            loop {
                match render(&input, &output, &cache, force, live, offline, &static_folder, &mut watcher) {
                    Ok(_) => break,
                    Err(error) => println!("ERROR: {error}"),
                }
                
                if live {
                    watcher.wait()?;
                } else {
                    break;
                }
            }
            
            Ok(())
        },
        Commands::New { output } => new(output),
    }
}
