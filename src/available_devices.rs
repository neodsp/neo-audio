#[derive(Debug)]
pub struct InputDevice {
    pub name: String,
    pub num_ch: u16,
}

#[derive(Debug)]
pub struct OutputDevice {
    pub name: String,
    pub num_ch: u16,
}

#[derive(Debug)]
pub struct Driver {
    pub name: String,
    pub input_devices: Vec<InputDevice>,
    pub output_devices: Vec<OutputDevice>,
}

#[derive(Debug)]
pub struct AvailableDevices {
    pub apis: Vec<Driver>,
}
