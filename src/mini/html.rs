use std::path::Path;
use std::fs::File;
use std::io::Write;
use askama::Template;
use minify_html_onepass as minify;

pub struct HtmlMinimizer<'a> {
    html: &'a mut String,
}

impl<'a> HtmlMinimizer<'a> {
    pub fn new(html: &'a mut String) -> Self {
        Self {
            html,
        }
    }
    
    pub fn append_template<T: Template>(&mut self, template: T) {
        template.render_into(self.html).unwrap();
    }
    
    pub fn minimize<P: AsRef<Path>>(self, path: P) {
        let config = minify::Cfg {
            minify_js: true,
            minify_css: true,
        };
        let minified = minify::in_place_str(self.html, &config).unwrap();
        let mut output_file = File::create(path.as_ref()).unwrap();
        output_file.write_all(minified.as_bytes()).unwrap();
        output_file.flush().unwrap();
    }
}
