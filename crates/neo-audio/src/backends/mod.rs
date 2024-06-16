pub const COMMON_SAMPLE_RATES: &[u32] = &[44100, 48000, 88200, 96000, 192000];
pub const COMMON_FRAMES_PER_BUFFER: &[u32] = &[16, 32, 64, 128, 256, 512, 1024, 2048];
pub const DEFAULT_SAMPLE_RATE: u32 = 48000;
pub const DEFAULT_NUM_FRAMES: u32 = 512;

#[cfg(feature = "cpal-backend")]
pub mod cpal_backend;
#[cfg(feature = "portaudio-backend")]
pub mod portaudio_backend;
#[cfg(feature = "rtaudio-backend")]
pub mod rtaudio_backend;

use realtime_tools::interleaved_audio::{InterleavedAudio, InterleavedAudioMut};

use crate::prelude::{Device, DeviceConfig, Error};

pub trait AudioBackend {
    fn default() -> Result<Self, Error>
    where
        Self: Sized;

    /// updates the available devices for the currently set api and updates the
    /// sample rates as well
    fn update_devices(&mut self) -> Result<(), Error>;
    /// this will just update the available sample rates for the currently set devices, usually
    /// this is not necessary to be called by the user
    fn update_sample_rates(&mut self);

    /// this will return all apis that are compiled into the program, this should not change during
    /// runtime of the program
    fn available_apis(&self) -> Vec<String>;
    /// you can set the api by name, the name can also be just a part of the name, it will use the
    /// first match. returns an error if the api is not present.
    fn set_api(&mut self, api_name: &str) -> Result<(), Error>;
    fn api(&self) -> String;

    fn available_output_devices(&self) -> Vec<String>;
    /// Sets the output devie by name and updates the available sample rates.
    /// The name can be just a fragment of the name, first match is used.
    /// Retruns an error if the device is not available.
    fn set_output_device(&mut self, device: Device) -> Result<(), Error>;
    fn output_device(&self) -> Option<String>;

    fn available_input_devices(&self) -> Vec<String>;
    /// Sets the input device by name and updates the available sample rates.
    /// The name can be just a fragment of the name, first match is used.
    /// Returns an error if the device is not available.
    fn set_input_device(&mut self, device: Device) -> Result<(), Error>;
    fn input_device(&self) -> Option<String>;

    fn available_num_output_channels(&self) -> u16;
    fn set_num_output_channels(&mut self, ch: u16) -> Result<(), Error>;
    fn num_output_channels(&self) -> u16;

    fn available_num_input_channels(&self) -> u16;
    fn set_num_input_channels(&mut self, ch: u16) -> Result<(), Error>;
    fn num_input_channels(&self) -> u16;

    fn available_sample_rates(&self) -> Vec<u32>;
    fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), Error>;
    fn sample_rate(&self) -> u32;

    fn available_num_frames(&self) -> Vec<u32>;
    fn set_num_frames(&mut self, num_frames: u32) -> Result<(), Error>;
    fn num_frames(&self) -> u32;

    /// set config all at once, good for loading application state at the start of the application
    fn set_config(&mut self, config: &DeviceConfig) -> Result<DeviceConfig, Error> {
        if config.api != self.api() {
            self.set_api(&config.api)?;
            self.set_output_device(Device::Default)?;
            self.set_input_device(Device::Default)?;
            self.update_devices()?;
        } else {
            self.set_output_device(config.output_device.clone())?;
            self.set_input_device(config.input_device.clone())?;
            self.set_num_output_channels(config.num_output_ch)?;
            self.set_num_input_channels(config.num_input_ch)?;
            self.set_sample_rate(config.sample_rate)?;
            self.set_num_frames(config.num_frames)?;
        }
        Ok(self.config())
    }
    /// get the selected config all at once, for saving state

    fn config(&self) -> DeviceConfig {
        DeviceConfig {
            api: self.api(),
            output_device: self.output_device().as_ref().into(),
            input_device: self.input_device().as_ref().into(),
            num_output_ch: self.num_output_channels(),
            num_input_ch: self.num_input_channels(),
            sample_rate: self.sample_rate(),
            num_frames: self.num_frames(),
        }
    }

    /// starts the audio stream with the currently selected options
    fn start_stream(
        &mut self,
        process_fn: impl FnMut(InterleavedAudioMut<'_, f32>, InterleavedAudio<'_, f32>) + Send + 'static,
    ) -> Result<(), Error>;
    /// call this to stop the audio stream
    fn stop_stream(&mut self) -> Result<(), Error>;
    /// if an error happens during the audio stream it will be returned by this function
    fn stream_error(&self) -> Result<(), Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_sine() {
        pub use portaudio_backend::PortAudioBackend as Backend;

        let mut audio_engine = Backend::default().unwrap();
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

        // // assert no error happing
        // // audio_engine.stream_error().unwrap();

        audio_engine.stop_stream().unwrap();
    }

    #[test]
    fn feedback() {
        pub use portaudio_backend::PortAudioBackend as Backend;

        let mut audio_engine = Backend::default().unwrap();
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

        // // assert no error happing
        // // audio_engine.stream_error().unwrap();

        audio_engine.stop_stream().unwrap();
    }
}
