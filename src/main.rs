mod ironssg;
use ironssg::{IronSSG, IronSSGConfig};
use serde_json::{self, Value};
use std::fs::File;
use std::io::Read;

fn main() {
    let config = Some(IronSSGConfig {
        dev: true,
        verbose: true,
        clean: true,
    });

    let mut iron_ssg = match IronSSG::new(config) {
        Ok(ssg) => ssg,
        Err(e) => {
            eprintln!("Failed to initialize IronSSG: {:?}", e);
            std::process::exit(1);
        }
    };

    // Read and parse router.json
    let mut file = File::open("router.json").expect("Unable to open router.json");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to read router.json");
    let v: Value = serde_json::from_str(&data).expect("Error parsing JSON");

    // Loop through the Pages array and register each page
    if let Some(pages) = v["pages"].as_array() {
        for page in pages {
            if let Err(e) = iron_ssg.page(page) {
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
