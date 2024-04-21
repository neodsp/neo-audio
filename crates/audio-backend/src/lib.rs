use rt_tools::interleaved_audio::{InterleavedAudio, InterleavedAudioMut};

use self::{
    audio_backend_error::AudioBackendError, device_config::DeviceConfig, device_name::AudioDevice,
};

const DEFAULT_SAMPLE_RATE: u32 = 48000;
const DEFAULT_NUM_FRAMES: u32 = 512;

pub mod audio_backend_error;
pub mod backends;
pub mod device_config;
pub mod device_name;

pub trait AudioBackend {
    fn default() -> Result<Self, AudioBackendError>
    where
        Self: Sized;

    /// updates the available devices for the currently set api and updates the
    /// sample rates as well
    fn update_devices(&mut self) -> Result<(), AudioBackendError>;
    /// this will just update the available sample rates for the currently set devices, usually
    /// this is not necessary to be called by the user
    fn update_sample_rates(&mut self);

    /// this will return all apis that are compiled into the program, this should not change during
    /// runtime of the program
    fn available_apis(&self) -> Vec<String>;
    /// you can set the api by name, the name can also be just a part of the name, it will use the
    /// first match. returns an error if the api is not present.
    fn set_api(&mut self, api_name: &str) -> Result<(), AudioBackendError>;
    fn api(&self) -> String;

    fn available_output_devices(&self) -> Vec<String>;
    /// Sets the output devie by name and updates the available sample rates.
    /// The name can be just a fragment of the name, first match is used.
    /// Retruns an error if the device is not available.
    fn set_output_device(&mut self, device: AudioDevice) -> Result<(), AudioBackendError>;
    fn output_device(&self) -> Option<String>;

    fn available_input_devices(&self) -> Vec<String>;
    /// Sets the input device by name and updates the available sample rates.
    /// The name can be just a fragment of the name, first match is used.
    /// Returns an error if the device is not available.
    fn set_input_device(&mut self, device: AudioDevice) -> Result<(), AudioBackendError>;
    fn input_device(&self) -> Option<String>;

    fn available_num_output_channels(&self) -> u32;
    fn set_num_output_channels(&mut self, ch: u32) -> Result<(), AudioBackendError>;
    fn num_output_channels(&self) -> u32;

    fn available_num_input_channels(&self) -> u32;
    fn set_num_input_channels(&mut self, ch: u32) -> Result<(), AudioBackendError>;
    fn num_input_channels(&self) -> u32;

    fn available_sample_rates(&self) -> Vec<u32>;
    fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), AudioBackendError>;
    fn sample_rate(&self) -> u32;

    fn available_num_frames(&self) -> Vec<u32>;
    fn set_num_frames(&mut self, num_frames: u32) -> Result<(), AudioBackendError>;
    fn num_frames(&self) -> u32;

    /// set config all at once, good for loading application state at the start of the application
    fn set_config(&mut self, config: &DeviceConfig) -> Result<DeviceConfig, AudioBackendError>;
    /// get the selected config all at once, for saving state
    fn config(&self) -> DeviceConfig;

    /// starts the audio stream with the currently selected options
    fn start_stream(
        &mut self,
        process_fn: impl FnMut(InterleavedAudioMut<'_, f32>, InterleavedAudio<'_, f32>) + Send + 'static,
    ) -> Result<(), AudioBackendError>;
    /// call this to stop the audio stream
    fn stop_stream(&mut self) -> Result<(), AudioBackendError>;
    /// if an error happens during the audio stream it will be returned by this function
    fn stream_error(&self) -> Result<(), AudioBackendError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_sine() {
        use backends::rtaudio_backend::RtAudioBackend as AudioEngine;

        let mut audio_engine = AudioEngine::default().unwrap();
        dbg!(audio_engine.config());

        let mut phasor = 0.0;
        let phasor_inc = 440.0 / audio_engine.sample_rate() as f32;

        audio_engine
            .start_stream(move |mut output, _| {
                for frame in output.frames_iter_mut() {
                    // Generate a sine wave at 440 Hz at 50% volume.
                    let val = (phasor * std::f32::consts::TAU).sin() * 0.5;
                    phasor = (phasor + phasor_inc).fract();
                    frame[0] = val;
                    frame[1] = val;
                }
            })
            .unwrap();

        std::thread::sleep(std::time::Duration::from_secs(3));

        // assert no error happing
        audio_engine.stream_error().unwrap();

        audio_engine.stop_stream().unwrap();
    }
}
