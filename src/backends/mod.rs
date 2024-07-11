use std::error::Error;

use crate::{
    audio_block::{AudioBlock, AudioBlockMut},
    available_devices::AvailableDevices,
    device_config::DeviceConfig,
    error::NeoAudioError,
};

pub mod portaudio_backend;

pub trait AudioBackend {
    fn new() -> Result<Self, NeoAudioError>
    where
        Self: Sized;

    // Devices
    fn available_devices(&self) -> Result<AvailableDevices, NeoAudioError>;
    fn default_config(&self) -> Result<DeviceConfig, NeoAudioError>;

    // Audio Stream
    fn start_stream<F>(
        &mut self,
        device_config: &DeviceConfig,
        process_fn: F,
    ) -> Result<(), NeoAudioError>
    where
        F: FnMut(AudioBlock, AudioBlockMut) -> Result<(), Box<dyn Error>> + 'static;
    fn stop_stream(&mut self) -> Result<(), NeoAudioError>;
    fn stream_error(&self) -> Result<(), NeoAudioError>;
}
