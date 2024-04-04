#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DeviceName {
    None,
    Default,
    Name(String),
}

impl From<Option<&String>> for DeviceName {
    fn from(value: Option<&String>) -> Self {
        match value {
            Some(name) => Self::Name(name.to_string()),
            None => Self::None,
        }
    }
}
