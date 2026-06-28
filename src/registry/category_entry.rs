use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryEntry {
    pub id:          String,
    pub title:       String,
    pub description: Option<String>,
}
