extern crate dotenv;
mod iron_ssg;

use dotenv::dotenv;
use iron_ssg::IronSSG;
use std::env;

fn main() {
    dotenv().ok(); // This line loads the .env file
    let config_path: String = env::var("CONFIG").unwrap_or("iron_ssg.toml".to_string());

    // Should allow some command line arguments to override config.toml
    // e.g. output directory, clean, verbose, etc.

    let mut iron_ssg = match IronSSG::new(&config_path) {
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
