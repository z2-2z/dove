use std::path::Path;
use std::fs::File;
use std::io::Write;
use css_minify::optimizations::{Minifier, Level};

pub fn minimize_css(src: &Path, dst: &Path) {
    let buffer = std::fs::read_to_string(src).unwrap();
    let minified = Minifier::default().minify(&buffer, Level::One).unwrap();
    let mut output_file = File::create(dst).unwrap();
    output_file.write_all(minified.as_bytes()).unwrap();
    output_file.flush().unwrap();
}
