use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryEntry {
    pub id:          String,
    pub title:       String,
    pub description: Option<String>,
}
