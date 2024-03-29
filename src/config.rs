use std::env;
use std::fs::File;
use std::io::Read;
#[cfg(test)]
use std::path::PathBuf;

use serde::Deserialize;

use once_cell::sync::Lazy;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::new);

#[derive(Deserialize)]
pub struct Config {
    pub service: Service,
    pub database: Database,
}

#[derive(Deserialize)]
pub struct Service {
    pub address: Option<String>,
    pub port: Option<u32>,
    pub safeword: Option<String>,
}

#[derive(Deserialize)]
pub struct Database {
    pub url: String,
}

impl Config {
    fn new() -> Self {
        let args: Vec<String> = env::args().collect();

        //config file can be the first argument
        let config_file = if args.len() > 1 {
            args.get(1).expect("Cannot retrieve config path").into()
        } else {
            #[cfg(test)]
            let mut path = PathBuf::from("pokifications");
            #[cfg(not(test))]
            let mut path = env::current_exe().expect("Cannot find executable path");
            // format!("{}.toml", path.file_stem().expect("Cannot find executable name").to_str().expect("Cannot parse executable name"))
            if !path.set_extension("toml") {
                panic!("Canno parse executable name: {}", path.display());
            }
            path
        };
        let mut toml = File::open(&config_file).unwrap_or_else(|_| panic!("File {} not found", config_file.display()));
        let mut s = String::new();
        toml.read_to_string(&mut s).expect("Unable to read Toml file");
        //read config file in toml format
        toml::from_str(&s).expect("Syntax error on Tolm file")
    }
}
