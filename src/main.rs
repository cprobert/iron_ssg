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
use std::io::Write;
use std::path::Path;

fn main() {
    dotenv().ok(); // This line loads the .env file

    let config_path: String = env::var("CONFIG").unwrap_or("iron_ssg.toml".to_string());
    println!("Config: {}", config_path);

    let mut file = File::open(&config_path).expect("Unable to open config");
    let mut data = String::new();

    file.read_to_string(&mut data)
        .expect("Unable to read config.toml");

    let config: Result<IronSSGConfig, toml::de::Error> = toml::from_str(&data);
    // dbg!(&config);
    if let Err(e) = &config {
        eprintln!("Deserialization error: {}", e);
    }

    match config {
        Ok(config) => {
            if let Err(e) = log_config(&config_path, &config) {
                eprintln!("Failed to log config: {:?}", e);
            }

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

fn log_config(
    config_path: &String,
    config: &IronSSGConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize the config to a JSON string
    let config_json = serde_json::to_string_pretty(config)?;

    // Ensure the _logs directory exists
    if !Path::new("_logs").exists() {
        std::fs::create_dir("_logs")?;
    }

    // Open the file for writing
    let file_path = format!("_logs/{}.json", config_path);
    let mut file = File::create(file_path)?;
    // Write the JSON string to the file
    file.write_all(config_json.as_bytes())?;

    Ok(())
}
