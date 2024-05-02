use crate::posts::{
    post::Post,
    metadata::PostDate,
};
use askama::Template;
use std::path::Path;

#[derive(Template)]
#[template(path = "feed/atom_entry.xml")]
struct AtomEntry<'a> {
    title: &'a str,
    url: String,
    published: String,
    categories: &'a [String],
}

#[derive(Template)]
#[template(path = "feed/atom.xml")]
struct AtomFeed<'a> {
    updated: String,
    entries: Vec<AtomEntry<'a>>,
}

fn max_post_date(posts: &[Post]) -> PostDate {
    let mut ret = PostDate::default();
    
    for post in posts {
        if post.metadata().date() > &ret {
            ret.clone_from(post.metadata().date());
        }
    }
    
    ret
}

fn atom_timestamp(date: &PostDate) -> String {
    format!("{:04}-{:02}-{:02}T00:00:00Z", date.year(), date.month(), date.day())
}

pub fn generate_atom_feed<P: AsRef<Path>>(filename: P, posts: &[Post]) {
    let latest_date = max_post_date(posts);
    
    if latest_date.year() == 0 {
        return;
    }
    
    let updated = atom_timestamp(&latest_date);
    let mut entries = Vec::new();
    
    for post in posts {
        let published = atom_timestamp(post.metadata().date());
        let url = if let Some(mirror) = post.metadata().mirror() {
            mirror.clone()
        } else {
            format!("https://z2-2z.github.io{}", post.url())
        };
        let entry = AtomEntry {
            title: post.metadata().title(),
            url,
            published,
            categories: post.metadata().categories(),
        };
        entries.push(entry);
    }
    
    let feed = AtomFeed {
        updated,
        entries,
    };
    
    let mut outfile = std::fs::File::create(filename).expect("Could not create atom feed");
    feed.write_into(&mut outfile).expect("Could not write to atom feed");
}
