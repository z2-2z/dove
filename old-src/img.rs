use std::path::Path;
use image::ImageReader;

pub fn is_image<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    
    matches!(
        path.extension().map(|x| x.as_encoded_bytes()),
        Some(b"jpg") | Some(b"JPG") |
        Some(b"jpeg") | Some(b"JPEG") |
        Some(b"png") | Some(b"PNG") |
        Some(b"bmp") | Some(b"BMP") |
        Some(b"gif") | Some(b"GIF") |
        Some(b"ico") | Some(b"ICO") |
        Some(b"tiff") | Some(b"TIFF")
    )
}

pub fn convert_to_webp(input: &Path, output: &Path) {
    let output = output.with_extension("webp");
    let img = ImageReader::open(input).unwrap().decode().unwrap();
    let encoder = webp::Encoder::from_image(&img).unwrap();
    let webp_data = encoder.encode(100.0);
    std::fs::write(output, &*webp_data).unwrap();
}
