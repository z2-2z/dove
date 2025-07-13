use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Write, Cursor};
use anyhow::Result;
use minify_html_onepass as minify_html;
use image::ImageReader;
use css_minify::optimizations as minify_css;

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
    let path = path.as_ref();
    let path = path.file_name().unwrap().to_str().unwrap();
    
    if path.ends_with(".min.css") {
        return false;
    }
    
    path.ends_with(".css")
}

pub fn is_js<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    let path = path.file_name().unwrap().to_str().unwrap();
    
    if path.ends_with(".min.js") {
        return false;
    }
    
    path.ends_with(".js")
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

pub fn transform_filename<P: AsRef<Path>>(filename: P) -> PathBuf {
    if is_image(&filename) {
        transform_image_filename(filename)
    } else if is_css(&filename) {
        transform_css_filename(filename)
    } else if is_js(&filename) {
        transform_js_filename(filename)
    } else {
        filename.as_ref().to_owned()
    }
}

pub fn transform_file<P1: AsRef<Path>, P2: AsRef<Path>>(infile: P1, outfile: P2) -> Result<()> {
    let mut buffer = std::fs::read(infile)?;
    transform_buffer(&mut buffer, outfile)
}

pub fn transform_buffer<P: AsRef<Path>>(buffer: &mut [u8], outfile: P) -> Result<()> {
    let write_buffer = |outfile: &Path, buffer: &[u8]| -> Result<()> {
        let mut file = File::create(outfile)?;
        file.write_all(buffer)?;
        file.flush()?;
        Ok(())
    };
    
    if is_image(&outfile) {
        let outfile = transform_image_filename(outfile);
        let image = ImageReader::new(Cursor::new(buffer)).with_guessed_format()?.decode()?;
        let encoder = match webp::Encoder::from_image(&image) {
            Ok(e) => e,
            Err(msg) => anyhow::bail!("{msg}"),
        };
        let webp_data = encoder.encode(85.0);
        write_buffer(&outfile, &webp_data)?;
        
    } else if is_css(&outfile) {
        let outfile = transform_css_filename(outfile);
        let str = std::str::from_utf8(buffer)?;
        let minified = minify_css::Minifier::default().minify(str, minify_css::Level::One).unwrap();
        write_buffer(&outfile, minified.as_bytes())?;
        
    } else if is_js(&outfile) {
        let outfile = transform_js_filename(outfile);
        let session = minify_js::Session::new();
        let mut out = Vec::new();
        if minify_js::minify(&session, minify_js::TopLevelMode::Global, buffer, &mut out).is_err() {
            anyhow::bail!("Minifying js file failed");
        }
        write_buffer(&outfile, &out)?;
        
    } else if is_html(&outfile) {
        let cfg = minify_html::Cfg {
            minify_js: true,
            minify_css: true,
        };
        let new_len = minify_html::in_place(buffer, &cfg)?;
        write_buffer(outfile.as_ref(), &buffer[..new_len])?;
        
    } else {
        write_buffer(outfile.as_ref(), buffer)?;
    }
    
    Ok(())
}
