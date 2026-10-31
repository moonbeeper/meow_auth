#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditField {
    Name,
    Email,
    Flags,
    DisplayName,
    Enabled,
    Scopes,
    RedirectUri,
    Public,
    Count,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuditChange {
    /// The field that was changed
    pub field: AuditField,
    /// The value before the change
    pub before: serde_json::Value,
    /// The value after the change
    pub after: serde_json::Value,
}

impl AuditChange {
    /// Used when some data is changed (that implies there's a "before" and "after")
    pub fn change(
        field: AuditField,
        before: impl serde::Serialize,
        after: impl serde::Serialize,
    ) -> Self {
        Self {
            field,
            before: serde_json::to_value(before).unwrap_or(serde_json::Value::Null),
            after: serde_json::to_value(after).unwrap_or(serde_json::Value::Null),
        }
    }

    /// Used when some data is created (that implies there's no "before")
    pub fn created(field: AuditField, after: impl serde::Serialize) -> Self {
        Self::change(field, serde_json::Value::Null, after)
    }

    /// Used when some data is delete (that implies there's no "after")
    pub fn deleted(field: AuditField, before: impl serde::Serialize) -> Self {
        Self::change(field, before, serde_json::Value::Null)
    }
}

/// A helper to compare two instances of a struct and return a list of audit changes
pub trait AuditDiff {
    /// Compare the current instance with a previous instance to return a list of changes
    fn audit_changes(&self, before: &Self) -> Vec<AuditChange>;
}
