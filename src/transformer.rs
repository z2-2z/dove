use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use anyhow::Result;
use minify_html_onepass as minify_html;

pub fn is_image<P: AsRef<Path>>(path: P) -> bool {
    matches!(
        path.as_ref().extension().map(|x| x.as_encoded_bytes()),
        Some(b"jpg") | Some(b"JPG") |
        Some(b"jpeg") | Some(b"JPEG") |
        Some(b"png") | Some(b"PNG") |
        Some(b"bmp") | Some(b"BMP") |
        Some(b"gif") | Some(b"GIF") |
        Some(b"ico") | Some(b"ICO") |
        Some(b"tiff") | Some(b"TIFF")
    )
}

pub fn is_css<P: AsRef<Path>>(path: P) -> bool {
    matches!(
        path.as_ref().extension().map(|x| x.as_encoded_bytes()),
        Some(b"css") | Some(b"CSS")
    )
}

pub fn is_js<P: AsRef<Path>>(path: P) -> bool {
    matches!(
        path.as_ref().extension().map(|x| x.as_encoded_bytes()),
        Some(b"js") | Some(b"JS")
    )
}

pub fn is_html<P: AsRef<Path>>(path: P) -> bool {
    matches!(
        path.as_ref().extension().map(|x| x.as_encoded_bytes()),
        Some(b"html") | Some(b"HTML")
    )
}

#[inline]
pub fn transform_css_filename<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = path.as_ref().to_owned();
    path.set_extension("min.css");
    path
}

#[inline]
pub fn transform_js_filename<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = path.as_ref().to_owned();
    path.set_extension("min.js");
    path
}

#[inline]
pub fn transform_image_filename<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = path.as_ref().to_owned();
    path.set_extension("webp");
    path
}

pub fn transform_file<P1: AsRef<Path>, P2: AsRef<Path>>(infile: P1, outfile: P2) -> Result<()> {
    let mut buffer = std::fs::read(infile)?;
    transform_buffer(&mut buffer, outfile, true)
}

pub fn transform_buffer<P: AsRef<Path>>(buffer: &mut [u8], outfile: P, overwrite: bool) -> Result<()> {
    let open = |outfile: P| -> Result<File> {
        let ret = if overwrite {
            File::create(outfile)?
        } else {
            File::options().append(true).create(true).open(outfile)?
        };
        Ok(ret)
    };
    
    if is_image(&outfile) {
        let outfile = transform_image_filename(outfile);
        todo!()
    } else if is_css(&outfile) {
        let outfile = transform_css_filename(outfile);
        todo!()
    } else if is_js(&outfile) {
        let outfile = transform_js_filename(outfile);
        todo!()
    } else if is_html(&outfile) {
        let cfg = minify_html::Cfg {
            minify_js: true,
            minify_css: true,
        };
        let new_len = minify_html::in_place(buffer, &cfg)?;
        let mut file = open(outfile)?;
        file.write_all(&buffer[..new_len])?;
        file.flush()?;
    } else {
        let mut file = open(outfile)?;
        file.write_all(buffer)?;
        file.flush()?;
    }
    
    Ok(())
}
