#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioDevice {
    None,
    Default,
    Name(String),
}

impl From<Option<&String>> for AudioDevice {
    fn from(value: Option<&String>) -> Self {
        match value {
            Some(name) => Self::Name(name.to_string()),
            None => Self::None,
        }
    }
}

impl From<Option<String>> for AudioDevice {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(name) => Self::Name(name),
            None => Self::None,
        }
    }
}

impl From<AudioDevice> for Option<String> {
    fn from(value: AudioDevice) -> Self {
        match value {
            AudioDevice::Name(name) => Some(name),
            _ => None,
        }
    }
}
