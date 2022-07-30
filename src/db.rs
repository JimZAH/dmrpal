use sled;

pub fn init() -> sled::Db{
    let db = match sled::open("db/db.dpal"){
        Ok(db) => db,
        Err(e) => {eprintln!("There was an error with the database file: {}", e);
        std::process::exit(-1);
        }
    };
    db
}