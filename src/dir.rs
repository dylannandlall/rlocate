use lazy_static::lazy_static;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};



lazy_static! {
    static ref EXCLUDED_PATHS: Vec<PathBuf> = vec![
        // PathBuf::from("/boot"),
        // PathBuf::from("/dev"),
        // PathBuf::from("/root"),
        // PathBuf::from("/sys"),
        PathBuf::from("/mnt"),
        PathBuf::from("/")
    ];
}

// Returns a vector of the folders in the root directory and removes certain folders that will cause a massive performance penalty
fn get_root_dir() -> Vec<DirEntry> {
    let mut root_vec: Vec<DirEntry> = Vec::new();

    for entry in WalkDir::new(Path::new("/")).max_depth(1) {
        match entry {
            Ok(entry) => {
                if !EXCLUDED_PATHS.contains(&entry.path().to_path_buf()) {
                    root_vec.push(entry);
                }
            }
            Err(err) => {
                println!("{}", err);
                continue;
            }
        }
    }

    println!("{:?}", root_vec);
    return root_vec;
}

// Returns a vector of all the paths found in the specified root_dir vector
pub fn get_filepaths() -> Vec<DirEntry> {
    let mut path_vec: Vec<DirEntry> = Vec::new();
    let root_dir = get_root_dir(); 

    for root in root_dir {
        for entry in WalkDir::new(root.path()).follow_links(false).follow_root_links(false) {
            match entry {
                Ok(entry) => {
                    path_vec.push(entry);
                }
                Err(error) => {
                    if let Some(inner) = error.io_error() {
                        match inner.kind() {
                            io::ErrorKind::PermissionDenied => continue,
                            _ => {
                                println!("Unexpected error occured: {}", inner);
                                // panic!("Exiting Program")
                            }
                        }
                    }
                }
            }
        }
    }
    return path_vec;
}