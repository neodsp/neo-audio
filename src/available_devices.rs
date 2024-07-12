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

impl Driver {
    pub fn input_device(&self, name: &str) -> Option<&InputDevice> {
        self.input_devices.iter().find(|d| d.name.contains(name))
    }

    pub fn output_device(&self, name: &str) -> Option<&OutputDevice> {
        self.output_devices.iter().find(|d| d.name.contains(name))
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AvailableDevices {
    pub drivers: Vec<Driver>,
}

impl AvailableDevices {
    pub fn driver(&self, name: &str) -> Option<&Driver> {
        self.drivers.iter().find(|d| d.name.contains(name))
    }
}
