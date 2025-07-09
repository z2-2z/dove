#![feature(path_file_prefix)]

mod posts;
mod renderer;
mod mini;
mod logger;
mod feed;
mod img;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs::File;
use memmap2::Mmap;
use clap::Parser;
use posts::{
    iter::PostIterator,
    post::Post,
};
use feed::atom::generate_atom_feed;
use renderer::post::PostRenderer;
use renderer::index::render_index;
use renderer::status::render_404_page;
use renderer::archive::render_archive;
use logger::Logger;
use std::process::exit;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn map_file<P: AsRef<Path>>(path: P) -> Mmap {
    let file = File::open(path).unwrap();
    unsafe { Mmap::map(&file) }.unwrap()
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Render {
        #[arg(short, long, value_name = "DIR")]
        input: String,
        
        #[arg(short, long, value_name = "DIR")]
        output: String,
        
        #[arg(short, long)]
        force: bool,
        
        #[arg(long, default_value_t = String::from("static"))]
        static_folder: String,
    },
    
    New {
        output: String,
    },
}

#[inline]
fn needs_updating(src: &Path, dst: &Path) -> bool {
    !dst.exists() ||
    src.metadata().unwrap().modified().unwrap() > dst.metadata().unwrap().modified().unwrap()
}

#[inline]
fn needs_minification(path: &Path, ext: &str) -> bool {
    path.to_string_lossy().strip_suffix(ext).map(|x| x.ends_with(".min")) == Some(false)
}

fn copy_static_files(force: bool, dir: &Path, input: &Path, output: &str, logger: &Logger) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let src_path = entry.unwrap().path();
        
        let mut dst_path = PathBuf::from(output);
        let part = src_path.strip_prefix(input).unwrap();
        dst_path.push(part);
        
        if src_path.is_dir() {
            if !dst_path.exists() {
                logger.debug(format!("Creating directory {}", dst_path.display()));
                std::fs::create_dir(&dst_path).unwrap();
            }
            copy_static_files(force, &src_path, input, output, logger);
        } else if needs_minification(&src_path, ".css") {
            if force || needs_updating(&src_path, &dst_path) {
                logger.debug(format!("Minifying {} -> {}", src_path.display(), dst_path.display()));
                mini::css::minimize_css(&src_path, &dst_path);
            }
        } else if needs_minification(&src_path, ".js") {
            if force || needs_updating(&src_path, &dst_path) {
                logger.debug(format!("Minifying {} -> {}", src_path.display(), dst_path.display()));
                mini::js::minimize_js(&src_path, &dst_path);
            }
        } else if force || needs_updating(&src_path, &dst_path) {
            logger.debug(format!("Copying {} -> {}", src_path.display(), dst_path.display()));
            std::fs::copy(src_path, dst_path).unwrap();
        }
    }
}

fn render(input: String, output: String, force: bool, static_folder: String) {
    let mut posts = Vec::new();
    let mut erroneous_posts = false;
    let mut updated_posts = false;
    let mut logger = Logger::new();
    
    /* Generate posts */
    for mut path in PostIterator::read(&input) {
        let content = map_file(&path);
        let post = match Post::new(&content) {
            Ok(post) => post,
            Err(err) => {
                logger.error(format!("{}: {}", path.display(), err));
                erroneous_posts = true;
                continue;
            },
        };
        
        if post.headless() {
            logger.info(format!("Picked up {}", path.display()));
            updated_posts = true;
        } else {
            let mut renderer = PostRenderer::new(&output, &post);

            if force || needs_updating(&path, renderer.output_file()) {
                logger.info(format!("Rendering {}", path.display()));
        
                if let Err(err) = renderer.render(&content, &post) {
                    logger.error(format!("{}: {}", path.display(), err));
                    erroneous_posts = true;
                    continue;
                }
        
                /* Copy static file mentions */
                assert!(path.pop());
                let src_base = path;
                let mut dst_base = renderer.output_file().to_path_buf();
                assert!(dst_base.pop());
        
                for url in renderer.urls() {
                    if !url.contains("://") {
                        let src = src_base.join(url);
                        let dst = dst_base.join(url);
                
                        if src.exists() {
                            if img::is_image(&src) {
                                logger.info(format!("  -> converting asset to webp: {}", src.display()));
                                img::convert_to_webp(&src, &dst);
                            } else {
                                logger.info(format!("  -> asset: {}", src.display()));
                                std::fs::copy(src, dst).unwrap();
                            }
                        }
                    }
                }
        
                /* Check that code languages are correct */
                for language in renderer.languages() {
                    let path = format!("{}/js/hljs/{}.min.js", static_folder, language);
                    let path = Path::new(&path);
            
                    if !path.exists() {
                        logger.error(format!("{}: codeblock uses unknown language '{}'", path.display(), language));
                        erroneous_posts = true;
                        continue;
                    }
                }
        
                updated_posts = true;
            }
        }
        
        posts.push(post);
        drop(content);
    }
    
    if erroneous_posts {
        logger.abort();
        exit(1);
    }
    
    /* Copy static content */
    copy_static_files(
        force,
        Path::new(&static_folder),
        Path::new(&static_folder),
        &output,
        &logger,
    );
    
    if force || updated_posts {
        posts.sort_by(|a, b| b.metadata().date().cmp(a.metadata().date()));
        
        /* Index page */
        logger.debug("Rendering index");
        render_index(&output, &posts);
        
        /* Archive */
        logger.debug("Rendering archive");
        render_archive(&output, &posts);
    }
    
    generate_atom_feed(
        format!("{}/atom.xml", &output),
        &posts
    );
    logger.debug("Generated atom feed");
    
    render_404_page(&output);
    logger.debug("Generated 404 page");
}

fn new(output: String) {
    let content = include_str!("new-post-template.md");
    let mut file = File::create(output).expect("Could not create output file");
    write!(&mut file, "{}", content).expect("Could not write to output file");
}

fn main() {
    let args = Args::parse();
    
    match args.command {
        Commands::Render { input, output, force, static_folder } => render(input, output, force, static_folder),
        Commands::New { output } => new(output),
    }
}
