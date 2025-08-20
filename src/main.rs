use std::time::Instant;
use clap::{Args, Parser, Subcommand};

use crate::db::database_search;

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

fn main() {
    let now = Instant::now();

    let args = Cli::parse();

    if let Some(command) = args.command {
        match command {
            Commands::Updatedb => {
                println!("Updating database...");
                let entries = dir::get_filepaths(); 
                db::database_handler(Some(entries), "updatedb").expect("Could not update database"); 
            }

            Commands::Debug => {
                db::database_handler(None, "debug").expect("Could not debug database");
            }

            Commands::Reset => {
                db::database_handler(None, "reset").expect("Could not delete database");
            }
        }
    }
    
    if args.search.keyword.is_some() {
        match database_search(args.search.keyword.unwrap(), "keyword") {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "[Error] Search for keyword has failed: {}", e);
                std::process::exit(1) 
            }
        }
    }
    
    if args.search.basename.is_some() {
        match database_search(args.search.basename.unwrap(), "basename") {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "[Error] Search for basename has failed: {}", e);
                std::process::exit(1) 
            }
        }
    }


    let elapsed = now.elapsed();
    println!("Program Runtime: {:?}", elapsed);
}
