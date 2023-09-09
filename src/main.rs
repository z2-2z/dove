#![feature(path_file_prefix)]

mod posts;
mod renderer;

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
}

/*
fn copy_static_files(src_dir: &Path, output: &str) {
    for entry in std::fs::read_dir(src_dir).unwrap() {
        let src_path = entry.unwrap().path();
        
        if src_path.is_dir() {
            copy_static_files(&src_path, output);
        } else {
            let mut dst_path = PathBuf::from(output);
            let part: PathBuf = src_path.iter().skip(1).collect();
            dst_path.push(part);
            
            let src_str = src_path.to_str().unwrap();
            
            if src_str.ends_with(".css") && !src_str.ends_with(".min.css") {
                
            } else if src_str.ends_with(".js") && !src_str.ends_with(".min.js") {
                todo!("Minifying js currently not supported");
            } else {
                
            }
            
            println!("{} -> {}", src_path.display(), dst_path.display());
        }
    }
}
*/

fn main() {
    let args = Args::parse();
    let mut posts = Vec::new();
    let mut erroneous_posts = false;
    
    //TODO: indicatif logger
    
    /* Generate individual posts */
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
        
        if args.force || renderer.needs_updating(&path) {
            renderer.render(&content, &post);
        }
        
        posts.push(post);
        drop(content);
    }
    
    if erroneous_posts {
        exit(1);
    }
    
    /* Copy static content */
    /*copy_static_files(
        Path::new("static"),
        &args.output,
    );*/
    
    
    //TODO: index page, category pages, author pages
    
    exit(0);
}
