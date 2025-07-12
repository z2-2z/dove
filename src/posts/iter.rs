use std::path::{Path, PathBuf};
use std::iter::Iterator;
use std::fs::{read_dir, ReadDir};
use std::ffi::OsStr;
use anyhow::Result;
use std::collections::VecDeque;

const POST_EXTENSION: &str = "md";

pub struct PostIterator {
    dir_queue: VecDeque<ReadDir>,
}

impl PostIterator {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.is_dir() {
            anyhow::bail!("Expected a directory: {}", path.display());
        }
        
        let mut dir_queue = VecDeque::with_capacity(512);
        dir_queue.push_back(read_dir(path)?);
        
        Ok(Self {
            dir_queue,
        })
    }
}

impl Iterator for PostIterator {
    type Item = PathBuf;
    
    fn next(&mut self) -> Option<Self::Item> {
        'restart:
        loop {
            if let Some(dir) = self.dir_queue.front_mut() {
                for entry in dir {
                    let path = entry.unwrap().path();
                    
                    if path.is_dir() {
                        let new_iter = read_dir(&path).unwrap();
                        self.dir_queue.push_back(new_iter);
                        continue 'restart;
                    } else if path.extension() == Some(OsStr::new(POST_EXTENSION)) {
                        return Some(path);
                    }
                }
                
                self.dir_queue.pop_front();
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_post_iterator() {
        for post in  PostIterator::new("test-data/postiter").unwrap() {
            println!("{}", post.display());
        }
    }
}
