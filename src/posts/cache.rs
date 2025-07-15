use std::collections::{HashMap, hash_map::Values};
use std::path::{PathBuf, Path};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::{posts::{PostMetadata, Post}, engine::Renderer, transformer};

#[derive(Serialize, Deserialize, PartialEq ,Eq)]
pub struct Dependency {
    input: PathBuf,
    output: PathBuf,
}

impl Dependency {
    pub fn input(&self) -> &Path {
        &self.input
    }
    
    pub fn output(&self) -> &Path {
        &self.output
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheEntry {
    dependencies: Vec<Dependency>,
    metadata: PostMetadata,
    url: String,
}

impl CacheEntry {
    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }
    
    pub fn metadata(&self) -> &PostMetadata {
        &self.metadata
    }
    
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct PostCache {
    resources: HashMap<PathBuf, CacheEntry>,
}

impl PostCache {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let data = std::fs::read(path)?;
        let cache = bitcode::deserialize(&data)?;
        
        Ok(cache)
    }
    
    pub fn clear(&mut self) {
        self.resources.clear();
    }
    
    pub fn get(&mut self, path: &PathBuf) -> Option<&CacheEntry> {
        self.resources.get(path)
    }
    
    pub fn insert(&mut self, input_basedir: &Path, input_file: &Path, output_basedir: &Path, output_file: &Path, post: &Post, renderer: &Renderer) -> bool {
        let mut dependencies = vec![
            Dependency {
                input: input_file.to_owned(),
                output: output_file.to_owned(),
            }
        ];
        
        for path in renderer.file_mentions() {
            let input = input_basedir.join(path);
            let output = transformer::transform_filename(output_basedir.join(path));
            
            dependencies.push(Dependency {
                input,
                output,
            });
        }
        
        let entry = CacheEntry {
            dependencies,
            metadata: post.metadata().clone(),
            url: post.url().to_owned(),
        };
        
        let input_file = input_file.to_owned();
        
        let changed = if let Some(value) = self.resources.get(&input_file) {
            &entry != value
        } else {
            true
        };
        
        if changed {
            self.resources.insert(input_file, entry);
        }
        
        changed
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let buffer = bitcode::serialize(self)?;
        std::fs::write(path, &buffer)?;
        Ok(())
    }
    
    pub fn resources(&self) -> Values<PathBuf, CacheEntry> {
        self.resources.values()
    }
}
