#![feature(path_file_prefix)]

mod posts;

use clap::Parser;
use posts::{
    iter::PostIterator,
    post::Post,
};
use std::process::exit;

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
        let post = match Post::from_file(&path) {
            Ok(post) => post,
            Err(err) => {
                eprintln!("[{}] {}", path.display(), err);
                erroneous_posts = true;
                continue;
            },
        };
        posts.push(post);
    }
    
    if erroneous_posts {
        exit(1);
    }
    
    exit(0);
}
