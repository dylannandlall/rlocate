use std::{fs, path::{PathBuf}, process};
use dirs;
use walkdir::DirEntry;
use rusqlite::{self, params, Connection, Result, Transaction};
use lazy_static::lazy_static;

const TABLE_NAME: &str = "entries";

lazy_static! {
    static ref DATABASE_FILE_PATH: PathBuf = dirs::data_dir().unwrap().join("rlocate/db.sql");
}

#[derive(Debug)]
pub struct PathEntry {
    path: String,
    basename: String,
}

impl PathEntry {
    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_basename(&self) -> &str {
        &self.basename
    }
}

pub fn database_handler(entries_opt: Option<Vec<DirEntry>>, command: &str) -> Result<()> {
    let mut conn: Connection = Connection::open(DATABASE_FILE_PATH.as_os_str())?;

    match command {
        "updatedb" => {
            if let Some(entries) = entries_opt {
                update_database(entries, &mut conn)?; 
            } else {
                eprintln!(
                    "[Error] Entries are null cannot update database");
                process::exit(1) 
            }
        }

        "debug" => {
            print_entries(&conn)?;
        }

        "reset" => {
            match delete_db() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!(
                        "[Error] Could not delete database at {}: {}", DATABASE_FILE_PATH.display(), e);
                    process::exit(1) 
                }
            }
        }

        _ => {
            eprintln!(
                "[Error] Database command not recognized {}", command);
            process::exit(1) 
        }
    }
    Ok(())
}

pub fn database_search(pattern: String, pattern_type: &str) -> Result<()> {
    let conn: Connection = Connection::open(DATABASE_FILE_PATH.as_os_str()).expect("Could not open up database connection");
    let database_entries: Vec<PathEntry> = retrieve_entries(&conn)?;

    match pattern_type {
        "keyword" => {
            for entry in database_entries {
                let entry = entry.get_path();
                if entry.contains(&pattern) {
                    let start = entry.find(&pattern).unwrap();
                    let (left, right) = entry.split_at(start);
                    let (middle, right) = right.split_at(pattern.len());

                    println!("{}\x1b[31m{}\x1b[0m{}", left, middle, right);
                }
            }
        }

        "basename" => {
            for entry in database_entries {
                let basename = entry.get_basename(); 
                
                if basename == pattern {
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

        _ => {
            eprintln!(
                "[Error] Search command not recognized {}", pattern_type);
            process::exit(1) 
        }
    }

    Ok(())
}

fn update_database(entries: Vec<DirEntry>, conn: &mut Connection) -> Result<()> {

    if let Some(database_parent_path) = DATABASE_FILE_PATH.parent() {
        // this is technology
        // Matches against a single pattern of the Result enum returned by fs::create_dir_all()
        // If the function call returns an Err variant of the Result enum, execute the code following, 
        // else skip it and continue program execution
        if let Err(e) = fs::create_dir_all(database_parent_path) {
            eprintln!(
                "[Error] Could not create database folder at {}: {}", database_parent_path.display(), e);
            process::exit(1)
        }
    } else {
        eprintln!(
            "[Error] Could not determine parent directory for {}",
            DATABASE_FILE_PATH.display()
        );
        process::exit(1);
    }

    // let mut conn: Connection = Connection::open(DATABASE_FILE_PATH.as_os_str())?;
    // Helps limit the scope of stmt to allow conn to be released from being borrowed after it is used in stmt 
    // and be moved to Ok() to be return in a Result enum
    {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name= ?1")?; 
        match stmt.exists(["entries"]) {
            Ok(bool) => {
                if bool == false {
                    conn.execute(
                    "CREATE TABLE entries (
                            id          INTEGER PRIMARY KEY AUTOINCREMENT,
                            path        TEXT NOT NULL,
                            basename    TEXT NOT NULL
                        )"
                    , (),
                    )?;
                }
            }
            Err(e) => {
                eprintln!(
                    "[Error] Could not create database table named entries: {}", e);
                process::exit(1)
            }
        };
    }

    match insert_entries(entries, conn) {
        Ok(_) => println!("Database updated"),
        Err(e) => {
            eprintln!(
                    "[Error] Could not update filepath database: {}", e);
                process::exit(1)
        }
    }

    Ok(())
}

fn insert_batch(entries: Vec<DirEntry>, tx: &Transaction) -> Result<()> {
    let mut stmt = tx.prepare("INSERT INTO entries (path, basename) VALUES (?1, ?2)")?;

    for entry in entries {
        let e = PathEntry {
            path: entry.path()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string(),
            basename: entry.path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
        };

        match stmt.execute(params![&e.path, &e.basename]) {
            Ok(_) => {}
            Err(err) => {
                println!("Failure to insert entry: {}", err);
            }
        }
    }
    Ok(())
}

fn retrieve_entries(conn: &Connection) -> Result<Vec<PathEntry>> {
    let mut entries: Vec<PathEntry> = Vec::new();
    let mut stmt = conn.prepare("SELECT path, basename from entries").unwrap();
    let entry_iter = stmt.query_map([], |row| {
        Ok(PathEntry {
            path: row.get(0)?,
            basename: row.get(1)?,
        })
    }).unwrap();


    for entry in entry_iter {
        entries.push(entry.unwrap())
    }
    
    Ok(entries)
}

fn insert_entries(entries: Vec<DirEntry>, conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?; 

    insert_batch(entries, &tx)?;
    tx.commit()?;

    Ok(())
}

fn print_entries(conn: &Connection) -> Result<()> {

    let mut stmt = conn.prepare("SELECT path, basename from entries")?;
    let entry_iter = stmt.query_map([], |row| {
        Ok(PathEntry {
            path: row.get(0)?,
            basename: row.get(1)?,
        })
    })?;

    for entry in entry_iter {
        println!("Path: {:?} Basename: {:?}", entry.as_ref().unwrap().path, entry.as_ref().unwrap().basename);
    }

    Ok(())
}

fn delete_db() -> std::io::Result<()> {
    fs::remove_file(DATABASE_FILE_PATH.as_os_str())?;
    Ok(())
}