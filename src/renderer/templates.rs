use askama::Template;

#[derive(Template)]
#[template(path = "post/header.html")]
pub struct PostHeader;

#[derive(Template)]
#[template(path = "post/footer.html")]
pub struct PostFooter;
