#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

impl From<Option<String>> for DeviceName {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(name) => Self::Name(name),
            None => Self::None,
        }
    }
}

impl From<DeviceName> for Option<String> {
    fn from(value: DeviceName) -> Self {
        match value {
            DeviceName::Name(name) => Some(name),
            _ => None,
        }
    }
}
