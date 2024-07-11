#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InputDevice {
    pub name: String,
    pub num_ch: u16,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OutputDevice {
    pub name: String,
    pub num_ch: u16,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Driver {
    pub name: String,
    pub input_devices: Vec<InputDevice>,
    pub output_devices: Vec<OutputDevice>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AvailableDevices {
    pub apis: Vec<Driver>,
}
