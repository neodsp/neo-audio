#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Device {
    None,
    Default,
    Name(String),
}

impl From<Option<&String>> for Device {
    fn from(value: Option<&String>) -> Self {
        match value {
            Some(name) => Self::Name(name.to_string()),
            None => Self::None,
        }
    }
}

impl From<Option<String>> for Device {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(name) => Self::Name(name),
            None => Self::None,
        }
    }
}

impl From<Device> for Option<String> {
    fn from(value: Device) -> Self {
        match value {
            Device::Name(name) => Some(name),
            _ => None,
        }
    }
}
