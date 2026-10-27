use crate::error::{BackendError, RESULT_PAYLOAD_TOO_LARGE, SCHEMA_LIMIT_EXCEEDED};
use crate::model::{SchemaItem, SqlValue};

pub const MAX_SQL_BYTES: usize = 1024 * 1024;
pub const MAX_RESULT_COLUMNS: usize = 512;
pub const MAX_TRANSPORTED_CELL_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_RESULT_PAYLOAD_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_SCHEMA_OBJECTS: usize = 10_000;
pub const MAX_SCHEMA_FIELD_BYTES: usize = 1024 * 1024;
pub const MAX_SCHEMA_PAYLOAD_BYTES: usize = 32 * 1024 * 1024;

#[derive(Default)]
pub struct PayloadBudget {
    used: usize,
}

impl PayloadBudget {
    pub fn add_column(&mut self, name: &str) -> Result<(), BackendError> {
        self.add(name.len())
    }

    pub fn add_value(&mut self, value: &SqlValue) -> Result<(), BackendError> {
        let bytes = match value {
            SqlValue::Null => 4,
            SqlValue::Integer { value } | SqlValue::Text { value } | SqlValue::Blob { value } => value.len(),
            SqlValue::Real { .. } => 24,
        };
        self.add(bytes)
    }

    pub fn add(&mut self, bytes: usize) -> Result<(), BackendError> {
        self.used = self.used.checked_add(bytes).ok_or_else(payload_error)?;
        if self.used > MAX_RESULT_PAYLOAD_BYTES {
            return Err(payload_error());
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct SchemaBudget {
    objects: usize,
    bytes: usize,
}

impl SchemaBudget {
    pub fn add(&mut self, item: &SchemaItem) -> Result<(), BackendError> {
        self.objects = self.objects.checked_add(1).ok_or_else(schema_error)?;
        if self.objects > MAX_SCHEMA_OBJECTS {
            return Err(schema_error());
        }
        let fields = [item.name.as_str(), item.table_name.as_str()].into_iter().chain(item.sql.as_deref());
        for field in fields {
            if field.len() > MAX_SCHEMA_FIELD_BYTES {
                return Err(schema_error());
            }
            self.bytes = self.bytes.checked_add(field.len()).ok_or_else(schema_error)?;
            if self.bytes > MAX_SCHEMA_PAYLOAD_BYTES {
                return Err(schema_error());
            }
        }
        Ok(())
    }
}

fn payload_error() -> BackendError {
    BackendError::new(RESULT_PAYLOAD_TOO_LARGE, "Result payload exceeds 32 MiB")
}

fn schema_error() -> BackendError {
    BackendError::new(SCHEMA_LIMIT_EXCEEDED, "Schema metadata exceeds configured limits")
}

#[cfg(test)]
mod tests {
    use super::{SchemaBudget, MAX_SCHEMA_FIELD_BYTES, MAX_SCHEMA_OBJECTS};
    use crate::model::SchemaItem;

    fn item(name: String, sql: Option<String>) -> SchemaItem {
        SchemaItem { name, table_name: "table".to_string(), sql }
    }

    #[test]
    fn schema_budget_rejects_per_item_object_and_aggregate_limits() {
        let mut budget = SchemaBudget::default();
        assert!(budget.add(&item("x".repeat(MAX_SCHEMA_FIELD_BYTES + 1), None)).is_err());

        let mut budget = SchemaBudget::default();
        for index in 0..MAX_SCHEMA_OBJECTS {
            assert!(budget.add(&item(index.to_string(), None)).is_ok());
        }
        assert!(budget.add(&item("one-too-many".to_string(), None)).is_err());

        let mut budget = SchemaBudget::default();
        assert!(budget.add(&item("a".repeat(1024 * 1024), Some("b".repeat(1024 * 1024)))).is_ok());
        for _ in 0..14 {
            assert!(budget.add(&item("a".repeat(1024 * 1024), Some("b".repeat(1024 * 1024)))).is_ok());
        }
        assert!(budget.add(&item("a".repeat(1024 * 1024), Some("b".repeat(1024 * 1024)))).is_err());
    }
}
