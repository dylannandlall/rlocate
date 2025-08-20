use std::{fs, path::{Path, PathBuf}, process};
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

pub fn get_database_connection () -> Result<Connection> {
    let mut conn: Connection = Connection::open(DATABASE_FILE_PATH.as_os_str())?;
    Ok(conn)
}

pub fn update_database(entries: Vec<DirEntry>) -> Result<Connection> {

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

    let mut conn: Connection = Connection::open(DATABASE_FILE_PATH.as_os_str())?;
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

    match insert_entries(entries, &mut conn) {
        Ok(_) => println!("rlocate successfully updated database with current filepath information"),
        Err(e) => {
            eprintln!(
                    "[Error] Could not update filepath database: {}", e);
                process::exit(1)
        }
    }

    Ok(conn)
}




// fn get_db_path() -> PathBuf {
//     let db_path = dirs::data_dir().unwrap().join("rlocate/db.sql"); 
//     return db_path;
// }

// pub fn init_db() -> Result<()> {
//     let database_path: PathBuf = get_db_path();
//     let parent_database_path: &Path = database_path.parent().unwrap(); 


//     if std::path::Path::is_dir(&parent_database_path) == false {
//         match std::fs::create_dir(database_path.parent().unwrap()) {
//             Ok(_) => {}
//             Err(e) => {
//                 eprintln!("[Error] Could not create database folder at {}: {}", parent_database_path.display(), e);
//                 std::process::exit(1);
//             }
//         }
//     }

//     let conn: Connection = Connection::open(get_db_path())?;

//     if check_if_table_exists().unwrap() == false {
//         conn.execute(
//             "CREATE TABLE entries (
//                     id          INTEGER PRIMARY KEY AUTOINCREMENT,
//                     path        TEXT NOT NULL,
//                     basename    TEXT NOT NULL
//                 )"
//             , (),
//         )?;
//     }
//     Ok(())
// }

// pub fn check_if_table_exists(conn: &Connection) -> Result<bool> {
//     // let conn: Connection = Connection::open(get_db_path())?; 

//     let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name= ?1")?; 
//     stmt.exists(["entries"])
// }

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

    let conn: Connection = match get_database_connection() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!(
                    "[Error] Could not retrieve database connection: {}", e);
                process::exit(1)
        }
    };

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

pub fn insert_entries(entries: Vec<DirEntry>, conn: &mut Connection) -> Result<()> {
    // let mut conn: Connection = Connection::open(get_db_path())?;
    let tx = conn.transaction()?; 

    insert_batch(entries, &tx)?;
    tx.commit()?;

    Ok(())
}

pub fn print_entries() -> Result<()> {
    // let conn: Connection = Connection::open(get_db_path())?;
    let conn: Connection = get_database_connection()?; 

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

pub fn delete_db() -> Result<()> {
    fs::remove_file(DATABASE_FILE_PATH.as_os_str());
    Ok(())
}