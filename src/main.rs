#![feature(path_file_prefix)]

mod posts;
mod renderer;
mod mini;

use std::path::{Path, PathBuf};
use std::fs::File;
use memmap2::Mmap;
use clap::Parser;
use posts::{
    iter::PostIterator,
    post::Post,
};
use renderer::html::HtmlRenderer;
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
    #[arg(short, long, value_name = "DIR")]
    input: String,
    
    #[arg(short, long, value_name = "DIR")]
    output: String,
    
    #[arg(short, long)]
    force: bool,
    
    #[arg(long, default_value_t = String::from("static"))]
    static_folder: String,
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

fn copy_static_files(force: bool, src_dir: &Path, output: &str) {
    for entry in std::fs::read_dir(src_dir).unwrap() {
        let src_path = entry.unwrap().path();
        
        let mut dst_path = PathBuf::from(output);
        let part: PathBuf = src_path.iter().skip(1).collect();
        dst_path.push(part);
        
        if src_path.is_dir() {
            if !dst_path.exists() {
                std::fs::create_dir(&dst_path).unwrap();
            }
            copy_static_files(force, &src_path, output);
        } else if needs_minification(&src_path, ".css") {
            if force || needs_updating(&src_path, &dst_path) {
                mini::css::minimize_css(&src_path, &dst_path);
            }
        } else if needs_minification(&src_path, ".js") {
            if force || needs_updating(&src_path, &dst_path) {
                mini::js::minimize_js(&src_path, &dst_path);
            }
        } else if force || needs_updating(&src_path, &dst_path) {
            std::fs::copy(src_path, dst_path).unwrap();
        }
    }
}

fn main() {
    let args = Args::parse();
    let mut posts = Vec::new();
    let mut erroneous_posts = false;
    
    //TODO: indicatif logger
    
    /* Generate posts */
    for path in PostIterator::read(&args.input) {
        let content = map_file(&path);
        let post = match Post::new(&content) {
            Ok(post) => post,
            Err(err) => {
                eprintln!("[{}] {}", path.display(), err);
                erroneous_posts = true;
                continue;
            },
        };
        
        let renderer = HtmlRenderer::new(&args.output, &post);
        
        if args.force || needs_updating(&path, renderer.output_file()) {
            if let Err(err) = renderer.render(&content, &post) {
                eprintln!("[{}] {}", path.display(), err);
                erroneous_posts = true;
                continue;
            }
        }
        
        posts.push(post);
        drop(content);
    }
    
    if erroneous_posts {
        exit(1);
    }
    
    /* Copy static content */
    copy_static_files(
        args.force,
        Path::new(&args.static_folder),
        &args.output,
    );
    
    //TODO: index page, category pages
    
    exit(0);
}
