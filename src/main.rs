use core::panic;
use std::time::Instant;
use clap::{Arg, Parser};
use walkdir::DirEntry;

mod dir;
mod db;


#[derive(Parser, Debug)]
#[clap(author = "Dylan Nandlall", version="0.1.0", about)]
/// A Simple File Searcher written in Rust
struct Cli {
    /// Keyword to search
    #[arg(short, long, group = "file", conflicts_with = "basename")]
    keyword: Option<String>,
    #[arg(short, conflicts_with = "file")]
    mode: Option<String>,
    #[arg(short, long, group = "file")]
    basename: Option<String>,
}



fn initialize() { 
    match db::init_db() {
        Ok(_) => {}
        Err(err) => {
            println!("Cannot initialize database: {}", err);
            panic!();
        }
    }

    let entries: Vec<DirEntry> = dir::get_filepaths(); 

    match db::insert_entries(entries) {
        Ok(_) => {}
        Err(err) => {
            println!("Could not insert entries: {}", err);
        }
    }
}

fn locate_keyword_basename(keyword: String) {
    let entries: Vec<db::PathEntry> = db::retrieve_entries();

    for entry in entries {
        let basename = entry.get_basename(); 
        
        if basename == keyword {
            // We need to split the string on "/" and get a vector of string types
            let directory: Vec<String> = entry.get_path()
                                        .split("/")
                                        .map(str::to_string)
                                        .collect();
            let directory = &directory[..directory.len() - 1].join("/");
            let directory = format!("{directory}/");

            println!("{}\x1b[31m{}\x1b[0m", directory, basename);
        }
    }
}

fn locate_keyword(keyword: String) {
    // let entries: Vec<String> = db::retrieve_entries();
    let entries: Vec<db::PathEntry> = db::retrieve_entries();

    for entry in entries {
        let entry = entry.get_path();
        if entry.contains(&keyword) {
            let start = entry.find(&keyword).unwrap();
            let (left, right) = entry.split_at(start);
            let (middle, right) = right.split_at(keyword.len());

            println!("{}\x1b[31m{}\x1b[0m{}", left, middle, right);
        }
    }
}

fn reset() {
    db::delete_db();
}

fn debug_db() {
    db::print_entries();
}

fn main() {
    let now = Instant::now();

    let args = Cli::parse();

    match args.mode.as_deref() {
        None => {}
        Some(command) => {
            match command {
                "init" => initialize(),
                "debug" => debug_db(),
                "reset" => reset(),
                _ => {
                    println!("Enter a valid mode");
                    return;
                }
            }
            
        }
    }

    if args.keyword.is_some() {
        locate_keyword(args.keyword.unwrap());
    }
    
    if args.basename.is_some() {
        locate_keyword_basename(args.basename.unwrap());
    }


    let elapsed = now.elapsed();
    println!("Program Runtime: {:?}", elapsed);
}
