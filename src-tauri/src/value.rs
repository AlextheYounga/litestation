use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::types::{Value, ValueRef};

use crate::error::{BackendError, INVALID_SQL, INVALID_TEXT_ENCODING, RESULT_CELL_TOO_LARGE};
use crate::limits::MAX_TRANSPORTED_CELL_BYTES;
use crate::model::SqlValue;

pub fn from_ref(value: ValueRef<'_>) -> Result<SqlValue, BackendError> {
    match value {
        ValueRef::Null => Ok(SqlValue::Null),
        ValueRef::Integer(value) => Ok(SqlValue::Integer { value: value.to_string() }),
        ValueRef::Real(value) => Ok(SqlValue::Real { value }),
        ValueRef::Text(value) => {
            ensure_transport_size(value.len())?;
            String::from_utf8(value.to_vec())
                .map(|value| SqlValue::Text { value })
                .map_err(|_| BackendError::new(INVALID_TEXT_ENCODING, "SQLite TEXT contains invalid UTF-8"))
        }
        ValueRef::Blob(value) => {
            let encoded_length = value
                .len()
                .checked_add(2)
                .and_then(|length| length.checked_div(3))
                .and_then(|groups| groups.checked_mul(4))
                .ok_or_else(cell_too_large)?;
            ensure_transport_size(encoded_length)?;
            Ok(SqlValue::Blob { value: STANDARD.encode(value) })
        }
    }
}

fn ensure_transport_size(bytes: usize) -> Result<(), BackendError> {
    if bytes > MAX_TRANSPORTED_CELL_BYTES {
        return Err(cell_too_large());
    }
    Ok(())
}

fn cell_too_large() -> BackendError {
    BackendError::new(RESULT_CELL_TOO_LARGE, "Result cell exceeds 8 MiB transport limit")
}

pub fn to_value(value: &SqlValue) -> Result<Value, BackendError> {
    match value {
        SqlValue::Null => Ok(Value::Null),
        SqlValue::Integer { value } => value
            .parse()
            .map(Value::Integer)
            .map_err(|_| BackendError::new(INVALID_SQL, "Integer values must be valid i64 strings")),
        SqlValue::Real { value } => Ok(Value::Real(*value)),
        SqlValue::Text { value } => Ok(Value::Text(value.clone())),
        SqlValue::Blob { value } => STANDARD
            .decode(value)
            .map(Value::Blob)
            .map_err(|_| BackendError::new(INVALID_SQL, "Blob values must be base64 encoded")),
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::types::ValueRef;

    use super::from_ref;
    use crate::limits::MAX_TRANSPORTED_CELL_BYTES;

    #[test]
    fn rejects_oversized_text_and_blob_before_transport_encoding() {
        let text = vec![b'x'; MAX_TRANSPORTED_CELL_BYTES + 1];
        assert_eq!(from_ref(ValueRef::Text(&text)).err().map(|error| error.code), Some("result_cell_too_large".into()));

        let blob = vec![0; (MAX_TRANSPORTED_CELL_BYTES / 4) * 3 + 1];
        assert_eq!(from_ref(ValueRef::Blob(&blob)).err().map(|error| error.code), Some("result_cell_too_large".into()));
    }
}
