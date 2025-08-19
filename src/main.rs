use core::panic;
use std::time::Instant;
use clap::{Args, Parser, Subcommand};
use rusqlite::Result;
use walkdir::DirEntry;

mod dir;
mod db;


#[derive(Parser, Debug)]
#[command(arg_required_else_help(true))]
#[clap(author = "Dylan Nandlall", version="0.1.0", about)]
/// A Simple File Searcher written in Rust
struct Cli {
    /// Keyword to search
    #[command(subcommand)]
    command: Option<Commands>,
    #[command(flatten)]
    search: SearchArgs,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Updatedb,
    Init,
    Debug,
    Reset,
}

#[derive(Args, Debug)]
struct SearchArgs {
    #[arg(short, long, group = "file", conflicts_with = "basename")]
    keyword: Option<String>,
    #[arg(short, conflicts_with = "file")]
    mode: Option<String>,
    #[arg(short, long, group = "file")]
    basename: Option<String>,
}

/// Initializes the file database and adds entries to it
fn initialize() { 
    if db::check_if_table_exists().unwrap() == false  {
        match db::init_db() {
            Ok(_) => {}
            Err(err) => {
                println!("Cannot initialize database: {}", err);
                std::process::exit(1);
            }
        }

        let entries: Vec<DirEntry> = dir::get_filepaths(); 

        match db::insert_entries(entries) {
            Ok(_) => {}
            Err(err) => {
                println!("Could not insert entries: {}", err);
                std::process::exit(1);
            }
        }
    }
}

fn update_db() {
    let entries: Vec<DirEntry> = dir::get_filepaths(); 

    match db::insert_entries(entries) {
        Ok(_) => {}
        Err(err) => {
            println!("[Error] Could not update database: {}", err);
            std::process::exit(1);
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

    if let Some(command) = args.command {
        match command {
            Commands::Updatedb => {
                println!("Running the updatedb logic...");
                update_db(); 
                println!("Scanning filesystem and refreshing database... Done.");
            },

            Commands::Init => {
                println!("Initializing the database...");
                initialize();
                println!("Database initialized");
            }

            Commands::Debug => {
                debug_db();
            }

            Commands::Reset => {
                reset();
            }
        }
    }

    // match args.search.mode.as_deref() {
    //     None => {}
    //     Some(command) => {
    //         match command {
    //             "init" => initialize(),
    //             "debug" => debug_db(),
    //             "reset" => reset(),
    //             _ => {
    //                 println!("Enter a valid mode");
    //                 return;
    //             }
    //         }
            
    //     }
    // }

    if args.search.keyword.is_some() {
        locate_keyword(args.search.keyword.unwrap());
    }
    
    if args.search.basename.is_some() {
        locate_keyword_basename(args.search.basename.unwrap());
    }


    let elapsed = now.elapsed();
    println!("Program Runtime: {:?}", elapsed);
}
