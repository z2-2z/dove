use crate::posts::post::Post;
use crate::mini::html::HtmlMinimizer;
use crate::renderer::templates::Archive;

pub fn render_archive(output_dir: &str, posts: &[Post]) {
    let mut minimizer = HtmlMinimizer::new();
    minimizer.append_template(Archive {
        posts,
    });
    minimizer.minimize(format!("{}/archive.html", output_dir));
}
