use std::path::Path;
use std::fs::File;
use memmap2::Mmap;
use anyhow::Result;

pub fn mmap_file<P: AsRef<Path>>(path: P) -> Result<Mmap> {
    let file = File::open(path)?;
    let ret = unsafe { Mmap::map(&file) }?;
    Ok(ret)
}
