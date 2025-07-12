use crate::mini::html::HtmlMinimizer;
use crate::renderer::templates::Status404;

pub fn render_404_page(output_dir: &str) {
    let mut minimizer = HtmlMinimizer::new();
    minimizer.append_template(Status404 {});
    minimizer.minimize(format!("{}/404.html", output_dir));
}
