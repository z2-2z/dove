use std::path::Path;
use std::fs::File;
use std::io::Write;
use minify_js::{Session, TopLevelMode, minify};

pub fn minimize_js(src: &Path, dst: &Path) {
    let code = std::fs::read(src).unwrap();
    let session = Session::new();
    let mut out = Vec::new();
    minify(&session, TopLevelMode::Global, &code, &mut out).unwrap();
    let mut output_file = File::create(dst).unwrap();
    output_file.write_all(&out).unwrap();
    output_file.flush().unwrap();
}
