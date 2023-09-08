#![feature(path_file_prefix)]

mod posts;
mod renderer;

use std::path::Path;
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
    output: String
}

fn main() {
    let args = Args::parse();
    let mut posts = Vec::new();
    let mut erroneous_posts = false;
    
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
        
        if renderer.needs_updating(&path) {
            renderer.render(&content, &post);
        }
        
        posts.push(post);
        drop(content);
    }
    
    if erroneous_posts {
        exit(1);
    }
    
    exit(0);
}
