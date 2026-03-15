use serde::{Deserialize, Serialize};

/// Schema describing user-configurable settings for a source.
/// The host app renders native UI from this schema.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreferenceSchema {
    pub fields: Vec<PreferenceField>,
}

impl PreferenceSchema {
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// A single configurable field in a source's preference schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceField {
    /// Unique key used to read/write this preference value.
    pub key: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub kind: PreferenceFieldKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<PreferenceValue>,
    /// If true, the field is visible in the UI. Defaults to true.
    #[serde(default = "bool_true")]
    pub visible: bool,
}

fn bool_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PreferenceFieldKind {
    Text {
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
    },
    Select {
        options: Vec<SelectOption>,
    },
    MultiSelect {
        options: Vec<SelectOption>,
    },
    Toggle,
    Number {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
    },
    /// A text field whose value is never shown in plaintext in the UI.
    Password,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

/// A concrete preference value, used for defaults and stored settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PreferenceValue {
    Str(String),
    Bool(bool),
    Num(f64),
    StrList(Vec<String>),
}
