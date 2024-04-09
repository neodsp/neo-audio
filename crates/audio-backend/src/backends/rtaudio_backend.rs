use std::{
    collections::HashSet,
    sync::mpsc::{self, Receiver},
};

use rt_tools::interleaved_audio::{InterleavedAudio, InterleavedAudioMut};
use rtaudio::{DeviceParams, Host};

use crate::{
    audio_backend_error::AudioBackendError, device_config::DeviceConfig, device_name::DeviceName,
    AudioBackend, DEFAULT_NUM_FRAMES, DEFAULT_SAMPLE_RATE,
};

pub struct RtAudioBackend {
    host: Option<Host>,
    apis: Vec<rtaudio::Api>,
    input_devices: Vec<rtaudio::DeviceInfo>,
    output_devices: Vec<rtaudio::DeviceInfo>,
    sample_rates: Vec<u32>,
    selected_api: rtaudio::Api,
    selected_input_device: Option<rtaudio::DeviceInfo>,
    selected_num_input_channels: u32,
    selected_output_device: Option<rtaudio::DeviceInfo>,
    selected_num_output_channels: u32,
    selected_sample_rate: u32,
    selected_num_frames: u32,
    stream_handle: Option<rtaudio::StreamHandle>,
    error_receiver: Option<Receiver<AudioBackendError>>,
}

impl std::fmt::Debug for RtAudioBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RtAudioImpl")
            .field("apis", &self.apis)
            .field("input_devices", &self.input_devices)
            .field("output_devices", &self.output_devices)
            .field("sample rates", &self.sample_rates)
            .field("selected_api", &self.selected_api.get_display_name())
            .field(
                "Selected Input",
                &self.selected_input_device.as_ref().map(|d| d.name.clone()),
            )
            .field(
                "Selected Output",
                &self.selected_output_device.as_ref().map(|d| d.name.clone()),
            )
            .field("Selected Samplerate", &self.selected_sample_rate)
            .field("Selected Number of Frames", &self.selected_num_frames)
            .finish()
    }
}

impl AudioBackend for RtAudioBackend {
    fn default() -> Result<Self, AudioBackendError> {
        let host = rtaudio::Host::new(rtaudio::Api::Unspecified)?;
        let selected_api = host.api();
        let mut neo_audio = Self {
            host: Some(host),
            apis: rtaudio::compiled_apis(),
            output_devices: vec![],
            input_devices: vec![],
            sample_rates: vec![],
            selected_api,
            selected_output_device: None,
            selected_input_device: None,
            selected_num_output_channels: 0,
            selected_num_input_channels: 0,
            selected_sample_rate: DEFAULT_SAMPLE_RATE,
            selected_num_frames: DEFAULT_NUM_FRAMES,
            stream_handle: None,
            error_receiver: None,
        };
        neo_audio.update_devices()?;
        neo_audio.set_output_device(DeviceName::Default)?;
        neo_audio.set_input_device(DeviceName::Default)?;
        Ok(neo_audio)
    }

    fn update_devices(&mut self) -> Result<(), AudioBackendError> {
        if let Some(host) = self.host.as_mut() {
            self.input_devices = host.iter_input_devices().collect();
            self.output_devices = host.iter_output_devices().collect();

            // use default device when selected device is not present anymore
            if let Some(input_device) = self.selected_input_device.as_ref() {
                if !self
                    .input_devices
                    .iter()
                    .any(|device| device.id == input_device.id)
                {
                    self.set_output_device(DeviceName::Default)?;
                }
            }

            // use default device when selected device is not present anymore
            if let Some(output_device) = self.selected_output_device.as_ref() {
                if !self
                    .output_devices
                    .iter()
                    .any(|device| device.id == output_device.id)
                {
                    self.set_input_device(DeviceName::Default)?;
                }
            }

            self.update_sample_rates();

            Ok(())
        } else {
            Err(AudioBackendError::StreamRunning)
        }
    }

    fn update_sample_rates(&mut self) {
        let input_sample_rates = self
            .selected_input_device
            .as_ref()
            .map(|device| device.sample_rates.clone());
        let output_sample_rates = self
            .selected_output_device
            .as_ref()
            .map(|device| device.sample_rates.clone());
        self.sample_rates = match (input_sample_rates, output_sample_rates) {
            (None, None) => vec![],
            (None, Some(out_sr)) => out_sr,
            (Some(in_sr), None) => in_sr,
            (Some(in_sr), Some(out_sr)) => {
                // collect sample rates that are supported by both devices
                let in_sr: HashSet<_> = in_sr.into_iter().collect();
                let out_sr: HashSet<_> = out_sr.into_iter().collect();
                in_sr.intersection(&out_sr).cloned().collect()
            }
        };

        // change the sample rate if the currently set is not available
        if !self.sample_rates.contains(&self.selected_sample_rate) {
            // select closest avaiblabe samplerate to default samplerate
            self.selected_sample_rate = self
                .sample_rates
                .iter()
                .min_by_key(|&&num| num.abs_diff(DEFAULT_SAMPLE_RATE))
                .copied()
                .unwrap_or(0);
        }
    }

    fn available_apis(&self) -> Vec<String> {
        self.apis.iter().map(|api| api.get_display_name()).collect()
    }

    fn set_api(&mut self, api_name: &str) -> Result<(), AudioBackendError> {
        if let Some(host) = self.host.as_mut() {
            self.selected_api = *self
                .apis
                .iter()
                .find(|api| api.get_display_name().contains(api_name))
                .ok_or(AudioBackendError::ApiNotFound)?;
            *host = Host::new(self.selected_api)?;
            self.update_devices()?;
            Ok(())
        } else {
            Err(AudioBackendError::StreamRunning)
        }
    }

    fn api(&self) -> String {
        self.selected_api.get_display_name()
    }

    fn available_output_devices(&self) -> Vec<String> {
        self.output_devices
            .iter()
            .map(|device| device.name.clone())
            .collect()
    }

    fn set_output_device(&mut self, device: DeviceName) -> Result<(), AudioBackendError> {
        if let Some(host) = self.host.as_ref() {
            self.selected_output_device = match device {
                DeviceName::None => None,
                DeviceName::Default => host.default_output_device().ok(),
                DeviceName::Name(name) => Some(
                    self.output_devices
                        .iter()
                        .find(|device| device.name.contains(&name))
                        .ok_or(AudioBackendError::OutputDeviceNotFound)?
                        .clone(),
                ),
            };
            self.selected_num_output_channels = self.available_num_output_channels();
            self.update_sample_rates();
            Ok(())
        } else {
            Err(AudioBackendError::StreamRunning)
        }
    }

    fn output_device(&self) -> Option<String> {
        self.selected_output_device.as_ref().map(|d| d.name.clone())
    }

    fn available_input_devices(&self) -> Vec<String> {
        self.input_devices
            .iter()
            .map(|device| device.name.clone())
            .collect()
    }

    fn set_input_device(&mut self, device: DeviceName) -> Result<(), AudioBackendError> {
        if let Some(host) = self.host.as_ref() {
            self.selected_input_device = match device {
                DeviceName::None => None,
                DeviceName::Default => host.default_input_device().ok(),
                DeviceName::Name(name) => Some(
                    self.input_devices
                        .iter()
                        .find(|device| device.name.contains(&name))
                        .ok_or(AudioBackendError::InputDeviceNotFound)?
                        .clone(),
                ),
            };
            self.selected_num_input_channels = self.available_num_input_channels();
            self.update_sample_rates();
            Ok(())
        } else {
            Err(AudioBackendError::StreamRunning)
        }
    }

    fn input_device(&self) -> Option<String> {
        self.selected_input_device.as_ref().map(|d| d.name.clone())
    }

    fn available_num_output_channels(&self) -> u32 {
        self.selected_output_device
            .as_ref()
            .map(|d| d.output_channels)
            .unwrap_or(0)
    }

    fn set_num_output_channels(&mut self, ch: u32) -> Result<(), AudioBackendError> {
        if ch > self.available_num_output_channels() {
            return Err(AudioBackendError::NumOutputChannels);
        }
        self.selected_num_output_channels = ch;
        Ok(())
    }

    fn num_output_channels(&self) -> u32 {
        self.selected_num_output_channels
    }

    fn available_num_input_channels(&self) -> u32 {
        self.selected_input_device
            .as_ref()
            .map(|d| d.input_channels)
            .unwrap_or(0)
    }

    fn set_num_input_channels(&mut self, ch: u32) -> Result<(), AudioBackendError> {
        if ch > self.available_num_input_channels() {
            return Err(AudioBackendError::NumInputChannels);
        }
        self.selected_num_input_channels = ch;
        Ok(())
    }

    fn num_input_channels(&self) -> u32 {
        self.selected_num_input_channels
    }

    fn available_sample_rates(&self) -> Vec<u32> {
        self.sample_rates.clone()
    }

    fn sample_rate(&self) -> u32 {
        self.selected_sample_rate
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), AudioBackendError> {
        if self.sample_rates.contains(&sample_rate) {
            self.selected_sample_rate = sample_rate;
            Ok(())
        } else {
            Err(AudioBackendError::SampleRate)
        }
    }

    fn set_num_frames(&mut self, num_frames: u32) -> Result<(), AudioBackendError> {
        self.selected_num_frames = num_frames;
        Ok(())
    }

    fn available_num_frames(&self) -> Vec<u32> {
        // TODO: get rid of hardcoded values
        vec![16, 32, 64, 128, 256, 512, 1024, 2048]
    }

    fn num_frames(&self) -> u32 {
        self.selected_num_frames
    }

    fn set_config(&mut self, config: &DeviceConfig) -> Result<DeviceConfig, AudioBackendError> {
        if config.api != self.api() {
            self.set_api(&config.api)?;
            self.set_output_device(DeviceName::Default)?;
            self.set_input_device(DeviceName::Default)?;
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

    fn start_stream(
        &mut self,
        mut process_fn: impl FnMut(InterleavedAudioMut<'_, f32>, InterleavedAudio<'_, f32>)
            + Send
            + 'static,
    ) -> Result<(), AudioBackendError> {
        if let Some(host) = self.host.take() {
            let (sender, receiver) = mpsc::channel();
            self.error_receiver = Some(receiver);
            self.stream_handle = Some(
                host.open_stream(
                    self.selected_output_device
                        .as_ref()
                        .map(|device| DeviceParams {
                            device_id: device.id,
                            num_channels: self.selected_num_output_channels,
                            first_channel: 0,
                        }),
                    self.selected_input_device
                        .as_ref()
                        .map(|device| DeviceParams {
                            device_id: device.id,
                            num_channels: self.selected_num_input_channels,
                            first_channel: 0,
                        }),
                    rtaudio::SampleFormat::Float32,
                    self.selected_sample_rate,
                    self.selected_num_frames,
                    rtaudio::StreamOptions::default(),
                    move |error| {
                        sender
                            .send(AudioBackendError::from(error))
                            .expect("sending error should work")
                    },
                )
                .map_err(|(host, err)| {
                    // if there was an error, we can save the host again in teh struct
                    self.host = Some(host);
                    err
                })?,
            );

            self.stream_handle
                .as_mut()
                .map(|handle| {
                    handle.start(
                        move |buffers: rtaudio::Buffers<'_>,
                              info: &rtaudio::StreamInfo,
                              _status: rtaudio::StreamStatus| {
                            if let rtaudio::Buffers::Float32 { output, input } = buffers {
                                process_fn(
                                    InterleavedAudioMut::from_slice(output, info.out_channels),
                                    InterleavedAudio::from_slice(input, info.in_channels),
                                );
                            }
                        },
                    )
                })
                .transpose()?;

            Ok(())
        } else {
            Err(AudioBackendError::Unspecified)
        }
    }

    fn stop_stream(&mut self) -> Result<(), AudioBackendError> {
        if let Some(mut stream_handle) = self.stream_handle.take() {
            stream_handle.stop();
        }
        // reload host if it's gone (it was eaten by the stream)
        if self.host.is_none() {
            self.host = Some(Host::new(self.selected_api)?)
        }
        Ok(())
    }

    fn stream_error(&self) -> Result<(), AudioBackendError> {
        if let Some(receiver) = self.error_receiver.as_ref() {
            if let Ok(error) = receiver.try_recv() {
                return Err(error);
            }
        }
        Ok(())
    }
}

impl From<rtaudio::RtAudioError> for AudioBackendError {
    fn from(value: rtaudio::RtAudioError) -> Self {
        Self::Backend(value.to_string())
    }
}
