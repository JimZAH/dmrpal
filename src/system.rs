use serde_derive::{Deserialize, Serialize};
use std::{fs, str, time};

/* system.rs
    Data related to the DMRPal System.
*/

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub my_id: u32,
    pub master_ip: String,
    pub verbose: u8,
}

pub struct System {
    pub master_reconnects: usize,
    pub total_timeouts: usize,
    pub uptime: time::SystemTime,
}

impl Config {
    pub fn load() -> Self {
        if let Ok(file) = fs::read("dmrpal.toml") {
            let file_data = str::from_utf8(&file).unwrap();
            toml::from_str(file_data).unwrap()
        } else {
            panic!("Configuration file not found!");
        }
    }
}

impl System {
    pub fn init() -> Self {
        Self {
            master_reconnects: 0,
            total_timeouts: 0,
            uptime: time::SystemTime::now(),
        }
    }
}
