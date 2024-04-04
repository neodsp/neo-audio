use super::device_name::DeviceName;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceConfig {
    pub api: String,
    pub output_device: DeviceName,
    pub input_device: DeviceName,
    pub num_output_ch: u32,
    pub num_input_ch: u32,
    pub sample_rate: u32,
    pub num_frames: u32,
}
