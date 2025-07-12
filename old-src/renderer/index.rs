use crate::posts::post::Post;
use crate::mini::html::HtmlMinimizer;
use crate::renderer::templates::Index;

pub fn render_index(output_dir: &str, posts: &[Post]) {
    let mut minimizer = HtmlMinimizer::new();
    minimizer.append_template(Index {
        posts,
    });
    minimizer.minimize(format!("{}/index.html", output_dir));
}
