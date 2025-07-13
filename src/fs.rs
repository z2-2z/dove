use std::path::Path;
use std::fs::{File, read_dir, create_dir};
use memmap2::Mmap;
use anyhow::Result;

use crate::transformer;

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

pub fn copy_dir_recursive<P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path>>(force: bool, current_dir: P1, root: P2, output: P3) -> Result<()> {
    let output = output.as_ref();
    let root = root.as_ref();
    
    for entry in read_dir(current_dir)? {
        let src_path = entry?.path();
        let dst_path = output.join(src_path.strip_prefix(root)?);
        
        if src_path.is_dir() {
            if !dst_path.exists() {
                create_dir(&dst_path)?;
            }
            copy_dir_recursive(force, &src_path, root, output)?;
        } else if force || is_newer(&src_path, &dst_path) {
            transformer::transform_file(src_path, dst_path)?;
        }
    }
    
    Ok(())
}
