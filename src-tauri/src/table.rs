use rusqlite::{Connection, Row};

use crate::error::{BackendError, INVALID_IDENTITY, INVALID_PAGINATION, QUERY_FAILED, RESULT_COLUMN_LIMIT_EXCEEDED};
use crate::limits::{PayloadBudget, MAX_RESULT_COLUMNS};
use crate::model::{BrowseRow, BrowseTableResponse, RowIdentity, SqlValue, TableColumn, TableMetadata};
use crate::value::from_ref;

pub fn metadata(connection: &Connection, table_name: &str) -> Result<TableMetadata, BackendError> {
    let object_type: String = connection
        .query_row("SELECT type FROM sqlite_schema WHERE name = ?1", [table_name], |row| row.get(0))
        .map_err(|error| table_error(table_name, &error))?;
    if object_type != "table" {
        return Err(BackendError::new("not_a_table", format!("'{table_name}' is not a table")));
    }
    let mut statement = connection
        .prepare(
            "SELECT name, type, \"notnull\", dflt_value, pk FROM pragma_table_xinfo(?1) WHERE hidden = 0 ORDER BY cid",
        )
        .map_err(|error| BackendError::database(QUERY_FAILED, &error))?;
    let columns = statement
        .query_map([table_name], column_from_row)
        .map_err(|error| BackendError::database(QUERY_FAILED, &error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| BackendError::database(QUERY_FAILED, &error))?;
    if columns.len() > MAX_RESULT_COLUMNS {
        return Err(BackendError::new(RESULT_COLUMN_LIMIT_EXCEEDED, "Table exceeds 512 columns"));
    }
    let mut primary_key = columns.iter().filter(|column| column.primary_key_position > 0).collect::<Vec<_>>();
    primary_key.sort_by_key(|column| column.primary_key_position);
    let primary_key = primary_key.into_iter().map(|column| column.name.clone()).collect::<Vec<_>>();
    let without_rowid: i64 = connection
        .query_row("SELECT wr FROM pragma_table_list WHERE schema = 'main' AND name = ?1", [table_name], |row| {
            row.get(0)
        })
        .map_err(|error| BackendError::database(QUERY_FAILED, &error))?;
    let table_type: String = connection
        .query_row("SELECT type FROM pragma_table_list WHERE schema = 'main' AND name = ?1", [table_name], |row| {
            row.get(0)
        })
        .map_err(|error| BackendError::database(QUERY_FAILED, &error))?;
    let virtual_table = table_type == "virtual";
    let rowid_alias = (!without_rowid.eq(&1) && primary_key.is_empty()).then(|| usable_rowid_alias(&columns)).flatten();
    let (identity_kind, reason) = if virtual_table {
        ("none", Some("Virtual tables are browse-only".to_string()))
    } else if !primary_key.is_empty() {
        ("primaryKey", None)
    } else if rowid_alias.is_some() {
        ("rowid", None)
    } else {
        ("none", Some("Table has no primary key or usable rowid".to_string()))
    };
    Ok(TableMetadata {
        name: table_name.to_string(),
        columns,
        primary_key,
        identity_kind: identity_kind.to_string(),
        editable: reason.is_none(),
        non_editable_reason: reason,
    })
}

pub fn browse(
    connection: &Connection,
    table_name: &str,
    offset: u64,
    limit: u32,
) -> Result<BrowseTableResponse, BackendError> {
    if offset > i64::MAX as u64 {
        return Err(BackendError::new(INVALID_PAGINATION, "Pagination offset exceeds SQLite's supported range"));
    }
    let metadata = metadata(connection, table_name)?;
    let limit = limit.clamp(1, 500);
    let names = metadata.columns.iter().map(|column| column.name.clone()).collect::<Vec<_>>();
    let mut budget = PayloadBudget::default();
    budget.add(metadata.name.len())?;
    for column in &metadata.columns {
        budget.add_column(&column.name)?;
        if let Some(declared_type) = &column.declared_type {
            budget.add(declared_type.len())?;
        }
        if let Some(default_sql) = &column.default_sql {
            budget.add(default_sql.len())?;
        }
    }
    let identity_column = rowid_alias(&metadata);
    let selected = names
        .iter()
        .map(|name| quote(name))
        .chain(identity_column.iter().map(|name| format!("{} AS \"__litestation_identity\"", quote(name))))
        .collect::<Vec<_>>()
        .join(", ");
    let order = if metadata.primary_key.is_empty() {
        identity_column.as_ref().map_or_else(|| "1".to_string(), Clone::clone)
    } else {
        metadata.primary_key.iter().map(|name| quote(name)).collect::<Vec<_>>().join(", ")
    };
    let sql = format!("SELECT {selected} FROM {} ORDER BY {order} LIMIT ?1 OFFSET ?2", quote(table_name));
    let mut statement = connection.prepare(&sql).map_err(|error| BackendError::database(QUERY_FAILED, &error))?;
    let capped = u64::from(limit) + 1;
    let mut rows = statement.query([capped, offset]).map_err(|error| BackendError::database(QUERY_FAILED, &error))?;
    let mut result = Vec::new();
    let mut has_more = false;
    while let Some(row) = rows.next().map_err(|error| BackendError::database(QUERY_FAILED, &error))? {
        if result.len() == usize::try_from(limit).unwrap_or(500) {
            has_more = true;
            break;
        }
        let values = row_values(row, names.len(), &mut budget)?;
        let identity = identity_for_row(&metadata, &values, row, names.len())?;
        account_identity(&identity, &mut budget)?;
        result.push(BrowseRow { values, identity });
    }
    Ok(BrowseTableResponse { metadata, columns: names, rows: result, offset, limit, has_more })
}

fn row_values(row: &Row<'_>, column_count: usize, budget: &mut PayloadBudget) -> Result<Vec<SqlValue>, BackendError> {
    (0..column_count)
        .map(|index| {
            let value =
                row.get_ref(index).map_err(|error| BackendError::database(QUERY_FAILED, &error)).and_then(from_ref)?;
            budget.add_value(&value)?;
            Ok(value)
        })
        .collect()
}

pub fn quote(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

pub fn rowid_alias(metadata: &TableMetadata) -> Option<String> {
    (metadata.identity_kind == "rowid").then(|| usable_rowid_alias(&metadata.columns)).flatten()
}

fn column_from_row(row: &Row<'_>) -> rusqlite::Result<TableColumn> {
    Ok(TableColumn {
        name: row.get(0)?,
        declared_type: row.get(1)?,
        not_null: row.get::<_, i64>(2)? != 0,
        default_sql: row.get(3)?,
        primary_key_position: row.get(4)?,
    })
}

fn table_error(table_name: &str, error: &rusqlite::Error) -> BackendError {
    if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
        BackendError::new("table_not_found", format!("Table '{table_name}' was not found"))
    } else {
        BackendError::database(QUERY_FAILED, &error)
    }
}

fn usable_rowid_alias(columns: &[TableColumn]) -> Option<String> {
    ["rowid", "_rowid_", "oid"]
        .into_iter()
        .find(|candidate| !columns.iter().any(|column| column.name.eq_ignore_ascii_case(candidate)))
        .map(str::to_string)
}

fn identity_for_row(
    metadata: &TableMetadata,
    values: &[SqlValue],
    row: &Row<'_>,
    identity_index: usize,
) -> Result<Option<RowIdentity>, BackendError> {
    if metadata.identity_kind == "primaryKey" {
        let values = metadata
            .primary_key
            .iter()
            .map(|name| {
                values
                    .iter()
                    .zip(&metadata.columns)
                    .find(|(_, column)| column.name == *name)
                    .map(|(value, _)| value.clone())
                    .ok_or_else(|| BackendError::new(INVALID_IDENTITY, "Primary key column was not returned"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(RowIdentity::PrimaryKey { values }))
    } else if metadata.identity_kind == "rowid" {
        let value: i64 = row.get(identity_index).map_err(|error| BackendError::database(QUERY_FAILED, &error))?;
        Ok(Some(RowIdentity::Rowid { value: value.to_string() }))
    } else {
        Ok(None)
    }
}

fn account_identity(identity: &Option<RowIdentity>, budget: &mut PayloadBudget) -> Result<(), BackendError> {
    match identity {
        Some(RowIdentity::PrimaryKey { values }) => {
            for value in values {
                budget.add_value(value)?;
            }
        }
        Some(RowIdentity::Rowid { value }) => budget.add(value.len())?,
        None => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rusqlite::Connection;

    use super::{browse, metadata};
    use crate::model::SqlValue;

    #[test]
    fn describes_primary_keys_rowids_without_rowid_and_paginates() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch("CREATE TABLE keyed (a INTEGER, b INTEGER, PRIMARY KEY (a, b)); CREATE TABLE rowids (value TEXT); CREATE TABLE compact (a INTEGER PRIMARY KEY, b TEXT) WITHOUT ROWID; INSERT INTO keyed VALUES (2, 1), (1, 2); INSERT INTO rowids VALUES ('a'), ('b');")?;
        assert_eq!(metadata(&connection, "keyed")?.identity_kind, "primaryKey");
        assert_eq!(metadata(&connection, "rowids")?.identity_kind, "rowid");
        assert_eq!(metadata(&connection, "compact")?.identity_kind, "primaryKey");
        let page = browse(&connection, "keyed", 0, 1)?;
        assert!(page.has_more);
        assert_eq!(page.rows[0].values[0], SqlValue::Integer { value: "1".to_string() });
        Ok(())
    }

    #[test]
    fn rejects_offsets_that_cannot_be_bound_as_sqlite_integers() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch("CREATE TABLE items (value TEXT)")?;
        let error = browse(&connection, "items", i64::MAX as u64 + 1, 10).expect_err("offset must be rejected");
        assert_eq!(error.code, "invalid_pagination");
        Ok(())
    }

    #[test]
    fn uses_table_list_for_virtual_tables_and_all_rowid_alias_shadowing() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch("CREATE TABLE shadowed (rowid TEXT, _rowid_ TEXT, oid TEXT)")?;
        assert_eq!(metadata(&connection, "shadowed")?.identity_kind, "none");
        connection.execute_batch("/* comment */ CREATE VIRTUAL TABLE searchable USING fts5(content)")?;
        let virtual_table = metadata(&connection, "searchable")?;
        assert!(!virtual_table.editable);
        assert_eq!(virtual_table.non_editable_reason.as_deref(), Some("Virtual tables are browse-only"));
        Ok(())
    }
}
