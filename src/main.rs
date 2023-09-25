extern crate dotenv;
extern crate json;

mod iron_ssg;

use dotenv::dotenv;
use iron_ssg::{IronSSG, IronSSGConfig};
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    dotenv().ok(); // This line loads the .env file

    let router = env::var("ROUTER").unwrap_or("router.json".to_string());
    println!("Router: {}", router);

    let config = Some(IronSSGConfig {
        dev: true,
        verbose: true,
        clean: true,
        dist: "dist".to_string(),
        public: "./public".to_string(),
    });

    let mut iron_ssg = match IronSSG::new(config) {
        Ok(ssg) => ssg,
        Err(e) => {
            eprintln!("Failed to initialize IronSSG: {:?}", e);
            std::process::exit(1);
        }
    };

    // Read and parse router.json
    let mut file = File::open(router).expect("Unable to open router");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to read router.json");
    let v = match json::parse(&data) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Error parsing JSON: {:?}", err);
            std::process::exit(1);
        }
    };

    // Loop through the Pages array and register each page
    if let json::JsonValue::Array(pages) = &v["pages"] {
        for page in pages {
            if let Err(e) = iron_ssg.page(&page.clone()) {
                eprintln!("Failed to create page: {:?}", e);
            }
        }
    } else {
        eprintln!("No pages found in router.json");
    }

    // Generate the pages
    if let Err(e) = iron_ssg.generate() {
        eprintln!("Failed to generate pages: {:?}", e);
    }
}
