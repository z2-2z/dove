use std::path::Path;
use std::fs::File;
use memmap2::Mmap;
use anyhow::Result;

pub fn mmap_file<P: AsRef<Path>>(path: P) -> Result<Mmap> {
    let file = File::open(path)?;
    let ret = unsafe { Mmap::map(&file) }?;
    Ok(ret)
}

pub fn is_newer<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2) -> bool {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    !dst.exists() ||
    src.metadata().unwrap().modified().unwrap() > dst.metadata().unwrap().modified().unwrap()
}
