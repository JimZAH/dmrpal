use sled;

pub fn init(ver: u64) -> sled::Db {
    let db = match sled::open("db/db.dpal") {
        Ok(db) => db,
        Err(e) => {
            eprintln!("There was an error with the database file: {}", e);
            std::process::exit(-1);
        }
    };
    let b = ver.to_be_bytes();
    match db.insert("swv", &b) {
        Ok(_) => {
            println!("Software version: OK")
        }
        Err(e) => {
            eprintln!("Error with writing software version: {}", e)
        }
    }
    db
}
