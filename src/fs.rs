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

fn cpr_helper(force: bool, current_dir: &Path, root: &Path, output: &Path) -> Result<()> {
    for entry in read_dir(current_dir)? {
        let src_path = entry?.path();
        let dst_path = output.join(src_path.strip_prefix(root)?);
        let trans_path = transformer::transform_filename(&dst_path);
        
        if src_path.is_dir() {
            if !dst_path.exists() {
                create_dir(&dst_path)?;
            }
            cpr_helper(force, &src_path, root, output)?;
        } else if force || is_newer(&src_path, &trans_path) {
            transformer::transform_file(src_path, dst_path)?;
        }
    }
    
    Ok(())
}

pub fn copy_dir_recursive<P1: AsRef<Path>, P2: AsRef<Path>>(force: bool, input: P1, output: P2) -> Result<()> {
    let output = output.as_ref();
    
    if !output.exists() {
        create_dir(output)?;
    }
    
    cpr_helper(force, input.as_ref(), input.as_ref(), output)
}
