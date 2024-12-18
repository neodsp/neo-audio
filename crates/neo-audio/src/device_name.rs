#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Device {
    None,
    Default,
    Name(String),
}

impl From<&str> for Device {
    fn from(value: &str) -> Self {
        Self::Name(value.to_string())
    }
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
