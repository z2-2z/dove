use std::path::{Path, PathBuf};

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

#[inline]
pub fn transform_image_filename<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut path = path.as_ref().to_owned();
    path.set_extension("webp");
    path
}

//TODO: is_css, is_js, is_html
//TODO: transform_file(in, out)
//TODO: transform_buffer(buf, out)
