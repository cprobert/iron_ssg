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

    // Your code to call .page() and .generate()
    if let Err(e) = iron_ssg.page(
        r#"{
            "view": "views/index.hbs",
            "model": "index.json",
            "controller": "index.rs",
            "title": "My Page Title"
        }"#,
    ) {
        eprintln!("Failed to create page: {:?}", e);
    }

    if let Err(e) = iron_ssg.generate() {
        eprintln!("Failed to generate pages: {:?}", e);
    }
}
