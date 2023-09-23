mod ironssg;
use ironssg::{IronSSG, IronSSGConfig};

fn main() {
    let config = Some(IronSSGConfig {
        dev: true,
        verbose: true,
    });

    let mut iron_ssg = match IronSSG::new(config) {
        Ok(ssg) => ssg,
        Err(e) => {
            eprintln!("Failed to initialize IronSSG: {:?}", e);
            std::process::exit(1);
        }
    };

    // Add the first page
    if let Err(e) = iron_ssg.page(
        r#"{
        "view": "views/index.hbs",
        "model": "index.json",
        "controller": "index.rs",
        "title": "My Page Title"
    }"#,
    ) {
        eprintln!("Failed to create first page: {:?}", e);
    }

    // Add a second page
    if let Err(e) = iron_ssg.page(
        r#"{
        "view": "views/about.hbs",
        "model": "about.json",
        "controller": "about.rs",
        "title": "About Us",
        "slug": "about"
    }"#,
    ) {
        eprintln!("Failed to create second page: {:?}", e);
    }

    // Generate all pages
    if let Err(e) = iron_ssg.generate() {
        eprintln!("Failed to generate pages: {:?}", e);
    }
}
