use core::panic;
use std::time::Instant;
use clap::{Arg, Parser};
use walkdir::DirEntry;

mod dir;
mod db;


#[derive(Parser, Debug)]
#[clap(author = "Dylan Nandlall", version="0.1.0", about)]
/// A Simple File Searcher written in Rust
struct Args {
    /// Keyword to search
    keyword: String,
    mode: i32,
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

    let args = Args::parse();

    match args.mode {
        0 => {
            initialize();
            // locate_keyword(args.keyword);
        },
        1 => locate_keyword(args.keyword),
        2 => locate_keyword_basename(args.keyword),
        -1 => debug_db(),
        -2 => reset(),
        _ => println!("Enter a valid mode")
    };

    let elapsed = now.elapsed();
    println!("Program Runtime: {:?}", elapsed);
}
