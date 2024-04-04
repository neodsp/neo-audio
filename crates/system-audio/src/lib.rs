use rt_tools::audio_buffers::{InputBuffer, OutputBuffer};

use self::{
    device_config::DeviceConfig, device_name::DeviceName, system_audio_error::SystemAudioError,
};

const DEFAULT_SAMPLE_RATE: u32 = 48000;
const DEFAULT_NUM_FRAMES: u32 = 512;

pub mod device_config;
pub mod device_name;
pub mod implementations;
pub mod system_audio_error;

pub trait SystemAudio {
    fn default() -> Result<Self, SystemAudioError>
    where
        Self: Sized;

    /// updates the available devices for the currently set api and updates the
    /// sample rates as well
    fn update_devices(&mut self) -> Result<(), SystemAudioError>;
    /// this will just update the available sample rates for the currently set devices, usually
    /// this is not necessary to be called by the user
    fn update_sample_rates(&mut self);

    /// this will return all apis that are compiled into the program, this should not change during
    /// runtime of the program
    fn available_apis(&self) -> Vec<String>;
    /// you can set the api by name, the name can also be just a part of the name, it will use the
    /// first match. returns an error if the api is not present.
    fn set_api(&mut self, api_name: &str) -> Result<(), SystemAudioError>;
    fn api(&self) -> String;

    fn available_output_devices(&self) -> Vec<String>;
    /// Sets the output devie by name and updates the available sample rates.
    /// The name can be just a fragment of the name, first match is used.
    /// Retruns an error if the device is not available.
    fn set_output_device(&mut self, device: DeviceName) -> Result<(), SystemAudioError>;
    fn output_device(&self) -> Option<String>;

    fn available_input_devices(&self) -> Vec<String>;
    /// Sets the input device by name and updates the available sample rates.
    /// The name can be just a fragment of the name, first match is used.
    /// Returns an error if the device is not available.
    fn set_input_device(&mut self, device: DeviceName) -> Result<(), SystemAudioError>;
    fn input_device(&self) -> Option<String>;

    fn available_num_output_channels(&self) -> u32;
    fn set_num_output_channels(&mut self, ch: u32) -> Result<(), SystemAudioError>;
    fn num_output_channels(&self) -> u32;

    fn available_num_input_channels(&self) -> u32;
    fn set_num_input_channels(&mut self, ch: u32) -> Result<(), SystemAudioError>;
    fn num_input_channels(&self) -> u32;

    fn available_sample_rates(&self) -> Vec<u32>;
    fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), SystemAudioError>;
    fn sample_rate(&self) -> u32;

    fn available_num_frames(&self) -> Vec<u32>;
    fn set_num_frames(&mut self, num_frames: u32) -> Result<(), SystemAudioError>;
    fn num_frames(&self) -> u32;

    /// set config all at once, good for loading application state at the start of the application
    fn set_config(&mut self, config: &DeviceConfig) -> Result<(), SystemAudioError>;
    /// get the selected config all at once, for saving state
    fn config(&self) -> DeviceConfig;

    /// starts the audio stream with the currently selected options
    fn start_stream(
        &mut self,
        process_fn: impl FnMut(OutputBuffer<'_, f32>, InputBuffer<'_, f32>) + Send + 'static,
    ) -> Result<(), SystemAudioError>;
    /// call this to stop the audio stream
    fn stop_stream(&mut self) -> Result<(), SystemAudioError>;
    /// if an error happens during the audio stream it will be returned by this function
    fn stream_error(&self) -> Result<(), SystemAudioError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_sine() {
        use implementations::system_rtaudio::SystemRtAudio as AudioEngine;

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
