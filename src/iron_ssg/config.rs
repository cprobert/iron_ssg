use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct IronSSGConfig {
    pub logging: Option<bool>,
    pub verbose: Option<bool>,
    pub clean: Option<bool>,
    pub output: Option<String>,
    #[serde(rename = "static_assets")]
    pub static_assets: Option<Vec<String>>,
    pub authors: Vec<String>,
    pub name: String,
    pub version: String,
    pub page: Vec<IronSSGPage>,
}

impl Default for IronSSGConfig {
    fn default() -> Self {
        Self {
            logging: Some(false),
            verbose: Some(true),
            clean: Some(true),
            output: Some("dist".to_string()),
            static_assets: None,
            authors: Vec::new(),
            name: "IronSSG Website".to_string(),
            version: "0.1.0".to_string(),
            page: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IronSSGPage {
    pub controller: Option<String>,
    pub path: Option<String>,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub view: String,
    pub components: Option<Vec<String>>,
    pub model: Option<String>,
}

impl Default for IronSSGPage {
    fn default() -> Self {
        Self {
            controller: None,
            path: None,
            slug: "index".to_string(),
            title: "".to_string(),
            description: None,
            view: "".to_string(),
            components: None,
            model: None,
        }
    }
}
