use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseMode {
    ReadOnly,
    ReadWrite,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenLocalDatabaseRequest {
    pub path: String,
    pub mode: DatabaseMode,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaItem {
    pub name: String,
    pub table_name: String,
    pub sql: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSchema {
    pub tables: Vec<SchemaItem>,
    pub views: Vec<SchemaItem>,
    pub indexes: Vec<SchemaItem>,
    pub triggers: Vec<SchemaItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDatabaseResponse {
    pub id: u64,
    pub path: String,
    pub display_name: String,
    pub mode: DatabaseMode,
    pub schema: DatabaseSchema,
}
