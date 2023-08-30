use clap::Parser;

mod postiter;

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
    
    for post_file in postiter::PostIterator::read(&args.input) {
        println!("{}", post_file.display());
    }
}
