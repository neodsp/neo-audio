use super::device_name::Device;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceConfig {
    pub api: String,
    pub output_device: Device,
    pub input_device: Device,
    pub num_output_ch: u16,
    pub num_input_ch: u16,
    pub sample_rate: u32,
    pub num_frames: u32,
}
