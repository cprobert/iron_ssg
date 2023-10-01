extern crate dotenv;
extern crate serde;
extern crate serde_json;
extern crate toml;

mod iron_ssg;

use dotenv::dotenv;
use iron_ssg::{IronSSG, IronSSGConfig};
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    dotenv().ok(); // This line loads the .env file

    let config_path = env::var("CONFIG").unwrap_or("iron_ssg.toml".to_string());
    println!("Config: {}", config_path);

    let mut file = File::open(config_path).expect("Unable to open config");
    let mut data = String::new();

    file.read_to_string(&mut data)
        .expect("Unable to read config.toml");

    let config: Result<IronSSGConfig, toml::de::Error> = toml::from_str(&data);

    match config {
        Ok(config) => {
            let iron_ssg = match IronSSG::new(config) {
                Ok(ssg) => ssg,
                Err(e) => {
                    eprintln!("Failed to initialise IronSSG: {:?}", e);
                    std::process::exit(1);
                }
            };

            // Generate the pages
            if let Err(e) = iron_ssg.generate() {
                eprintln!("Failed to generate pages: {:?}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to parse TOML: {}", e);
            std::process::exit(1);
        }
    }
}
