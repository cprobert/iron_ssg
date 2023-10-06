use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct PageManifest {
    pub title: String,
    pub view_file_path: String,
    pub model_file_path: String,
    pub dist_path: String,
    pub dist_file_path: String,
    pub model: serde_json::Value,
}
