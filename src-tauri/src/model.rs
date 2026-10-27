use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SqlValue {
    Null,
    Integer { value: String },
    Real { value: f64 },
    Text { value: String },
    Blob { value: String },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSqlRequest {
    pub connection_id: u64,
    pub sql: String,
    pub max_rows: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSqlResponse {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<SqlValue>>,
    pub affected_rows: Option<usize>,
    pub truncated: bool,
    pub duration_ms: u128,
    pub schema_changed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RowIdentity {
    PrimaryKey { values: Vec<SqlValue> },
    Rowid { value: String },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRequest {
    pub connection_id: u64,
    pub table_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableColumn {
    pub name: String,
    pub declared_type: Option<String>,
    pub not_null: bool,
    pub default_sql: Option<String>,
    pub primary_key_position: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableMetadata {
    pub name: String,
    pub columns: Vec<TableColumn>,
    pub primary_key: Vec<String>,
    pub identity_kind: String,
    pub editable: bool,
    pub non_editable_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseTableRequest {
    pub connection_id: u64,
    pub table_name: String,
    pub offset: u64,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseRow {
    pub values: Vec<SqlValue>,
    pub identity: Option<RowIdentity>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseTableResponse {
    pub metadata: TableMetadata,
    pub columns: Vec<String>,
    pub rows: Vec<BrowseRow>,
    pub offset: u64,
    pub limit: u32,
    pub has_more: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnValue {
    pub column: String,
    pub value: SqlValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertRowRequest {
    pub connection_id: u64,
    pub table_name: String,
    pub values: Vec<ColumnValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRowRequest {
    pub connection_id: u64,
    pub table_name: String,
    pub identity: RowIdentity,
    pub expected: Vec<ColumnValue>,
    pub changes: Vec<ColumnValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRowRequest {
    pub connection_id: u64,
    pub table_name: String,
    pub identity: RowIdentity,
    pub expected: Vec<ColumnValue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationResult {
    pub affected_rows: usize,
    pub identity: Option<RowIdentity>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSqlDumpRequest {
    pub connection_id: u64,
    pub source_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSqlDumpResponse {
    pub schema: DatabaseSchema,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSqliteFileRequest {
    pub source_path: String,
    pub destination_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSqliteFileResponse {
    pub path: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRowsRequest {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<SqlValue>>,
    pub format: String,
    pub destination_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRowsResponse {
    pub path: String,
    pub row_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSqlDumpRequest {
    pub connection_id: u64,
    pub destination_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSqlDumpResponse {
    pub path: String,
    pub row_count: usize,
}
