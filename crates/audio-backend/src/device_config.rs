use super::device_name::AudioDevice;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceConfig {
    pub api: String,
    pub output_device: AudioDevice,
    pub input_device: AudioDevice,
    pub num_output_ch: u32,
    pub num_input_ch: u32,
    pub sample_rate: u32,
    pub num_frames: u32,
}
