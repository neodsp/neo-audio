use super::device_name::Device;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceConfig {
    pub api: String,
    pub output_device: Device,
    pub input_device: Device,
    pub num_output_ch: u32,
    pub num_input_ch: u32,
    pub sample_rate: u32,
    pub num_frames: u32,
}
