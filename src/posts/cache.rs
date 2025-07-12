use std::collections::HashMap;
use std::path::{PathBuf, Path};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::posts::PostMetadata;

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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
        
        let data = std::fs::read_to_string(path)?;
        let cache = serde_json::from_str(&data)?;
        
        Ok(cache)
    }
    
    pub fn clear(&mut self) {
        self.resources.clear();
    }
    
    pub fn get(&mut self, path: &PathBuf) -> Option<&CacheEntry> {
        self.resources.get(path)
    }
    
    //TODO: insert
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let output = std::fs::File::create(path)?;
        serde_json::to_writer(output, self)?;
        Ok(())
    }
}
