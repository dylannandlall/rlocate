use std::{fs, path::{Path, PathBuf}};
use dirs;
use walkdir::DirEntry;
use rusqlite::{self, params, Connection, Result, Transaction};

// const TABLE_NAME: &str = "entries";

fn get_db_path() -> PathBuf {
    let db_path = dirs::data_dir().unwrap().join("rlocate/db.sql"); 
    return db_path;
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

pub fn init_db() -> Result<()> {
    let database_path: PathBuf = get_db_path();
    let parent_database_path: &Path = database_path.parent().unwrap(); 


    if std::path::Path::is_dir(&parent_database_path) == false {
        match std::fs::create_dir(database_path.parent().unwrap()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[Error] Could not create database folder at {}: {}", parent_database_path.display(), e);
                std::process::exit(1);
            }
        }
    }

    let conn: Connection = Connection::open(get_db_path())?;

    if check_if_table_exists().unwrap() == false {
        conn.execute(
            "CREATE TABLE entries (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    path        TEXT NOT NULL,
                    basename    TEXT NOT NULL
                )"
            , (),
        )?;
    }
    Ok(())
}

pub fn check_if_table_exists() -> Result<bool> {
    let conn: Connection = Connection::open(get_db_path())?; 

    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name= ?1")?; 
    stmt.exists(["entries"])
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

pub fn retrieve_entries() -> Vec<PathEntry> {
    let mut entries: Vec<PathEntry> = Vec::new();
    let conn: Connection = Connection::open(get_db_path()).unwrap();

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
    return entries;
}

pub fn insert_entries(entries: Vec<DirEntry>) -> Result<()> {
    let mut conn: Connection = Connection::open(get_db_path())?;
    let tx = conn.transaction()?; 

    insert_batch(entries, &tx)?;
    tx.commit()?;

    Ok(())
}

pub fn print_entries() -> Result<()> {
    let conn: Connection = Connection::open(get_db_path())?;

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

pub fn delete_db() -> std::io::Result<()> {
    fs::remove_file(get_db_path())?;
    Ok(())
}