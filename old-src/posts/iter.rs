use std::path::{Path, PathBuf};
use std::iter::Iterator;
use std::fs::{read_dir, ReadDir};
use std::ffi::OsStr;

pub const POST_EXTENSION: &str = "md";

pub struct PostIterator {
    dir_stack: Vec<PathBuf>,
    next_item: Option<PathBuf>,
    current_iter: Option<ReadDir>,
}

impl PostIterator {
    pub fn read<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let mut current_iter = None;
        let mut next_item = None;
        
        if path.is_dir() {
            current_iter = Some(read_dir(path).unwrap());
        } else {
            next_item = Some(path.to_owned());
        }
        
        let mut ret = Self {
            dir_stack: Vec::with_capacity(64),
            next_item,
            current_iter,
        };
        ret.find_next_item();
        ret
    }
    
    fn find_next_item(&mut self) {
        while let Some(current_iter) = &mut self.current_iter {
            /* Find next item in current iterator */
            for item in current_iter {
                let item = item.unwrap().path();
                
                if item.is_dir() {
                    self.dir_stack.push(item);
                } else if item.extension() == Some(OsStr::new(POST_EXTENSION)) {
                    self.next_item = Some(item);
                    return;
                } 
            }
            
            /* Create a new iterator */
            self.current_iter = self.dir_stack.pop().map(|x| read_dir(x).unwrap());
        }
    }
}

impl Iterator for PostIterator {
    type Item = PathBuf;
    
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.next_item.take();
        self.find_next_item();
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_post_iterator() {
        for post in PostIterator::read("test-data/postiter") {
            println!("{}", post.display());
        }
        
        for post in PostIterator::read("test-data/postiter/root.md") {
            println!("{}", post.display());
        }
    }
}
