#![feature(path_file_prefix)]

mod posts;

use clap::Parser;
use posts::{
    iter::PostIterator,
    post::Post,
};

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
    
    for path in PostIterator::read(&args.input) {
        println!("{}", path.display());
        let _post = Post::from_file(&path);
    }
}
