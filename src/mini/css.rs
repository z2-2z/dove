use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use css_minify::optimizations::{Minifier, Level};

pub fn minimize_css(buffer: &mut String, src: &Path, dst: &Path) {
    buffer.clear();
    File::open(src).unwrap().read_to_string(buffer).unwrap();
    let minified = Minifier::default().minify(buffer, Level::One).unwrap();
    let mut output_file = File::create(dst).unwrap();
    output_file.write_all(minified.as_bytes()).unwrap();
    output_file.flush().unwrap();
}
