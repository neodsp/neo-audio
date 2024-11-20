use std::{
    collections::HashSet,
    sync::mpsc::{self, Receiver},
};

use realtime_tools::interleaved_audio::{InterleavedAudio, InterleavedAudioMut};
use rtaudio::{DeviceParams, Host};

use crate::{device_name::Device, error::NeoAudioError, AudioBackend};

use super::{DEFAULT_NUM_FRAMES, DEFAULT_SAMPLE_RATE};

pub struct RtAudioBackend {
    host: Option<Host>,
    apis: Vec<rtaudio::Api>,
    input_devices: Vec<rtaudio::DeviceInfo>,
    output_devices: Vec<rtaudio::DeviceInfo>,
    sample_rates: Vec<u32>,
    selected_api: rtaudio::Api,
    selected_input_device: Option<rtaudio::DeviceInfo>,
    selected_num_input_channels: u16,
    selected_output_device: Option<rtaudio::DeviceInfo>,
    selected_num_output_channels: u16,
    selected_sample_rate: u32,
    selected_num_frames: u32,
    stream_handle: Option<rtaudio::StreamHandle>,
    error_receiver: Option<Receiver<NeoAudioError>>,
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
    fn default() -> Result<Self, NeoAudioError> {
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
        neo_audio.set_output_device(Device::Default)?;
        neo_audio.set_input_device(Device::Default)?;
        Ok(neo_audio)
    }

    fn update_devices(&mut self) -> Result<(), NeoAudioError> {
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
                    self.set_output_device(Device::Default)?;
                }
            }

            // use default device when selected device is not present anymore
            if let Some(output_device) = self.selected_output_device.as_ref() {
                if !self
                    .output_devices
                    .iter()
                    .any(|device| device.id == output_device.id)
                {
                    self.set_input_device(Device::Default)?;
                }
            }

            self.update_sample_rates();

            Ok(())
        } else {
            Err(NeoAudioError::StreamRunning)
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

        self.sample_rates.sort();

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

    fn set_api(&mut self, api_name: &str) -> Result<(), NeoAudioError> {
        if let Some(host) = self.host.as_mut() {
            self.selected_api = *self
                .apis
                .iter()
                .find(|api| api.get_display_name().contains(api_name))
                .ok_or(NeoAudioError::ApiNotFound)?;
            *host = Host::new(self.selected_api)?;
            self.update_devices()?;
            Ok(())
        } else {
            Err(NeoAudioError::StreamRunning)
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

    fn set_output_device(&mut self, device: Device) -> Result<(), NeoAudioError> {
        if let Some(host) = self.host.as_ref() {
            self.selected_output_device = match device {
                Device::None => None,
                Device::Default => host.default_output_device().ok(),
                Device::Name(name) => Some(
                    self.output_devices
                        .iter()
                        .find(|device| device.name.contains(&name))
                        .ok_or(NeoAudioError::OutputDeviceNotFound)?
                        .clone(),
                ),
            };
            self.selected_num_output_channels = self.available_num_output_channels();
            self.update_sample_rates();
            Ok(())
        } else {
            Err(NeoAudioError::StreamRunning)
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

    fn set_input_device(&mut self, device: Device) -> Result<(), NeoAudioError> {
        if let Some(host) = self.host.as_ref() {
            self.selected_input_device = match device {
                Device::None => None,
                Device::Default => host.default_input_device().ok(),
                Device::Name(name) => Some(
                    self.input_devices
                        .iter()
                        .find(|device| device.name.contains(&name))
                        .ok_or(NeoAudioError::InputDeviceNotFound)?
                        .clone(),
                ),
            };
            self.selected_num_input_channels = self.available_num_input_channels();
            self.update_sample_rates();
            Ok(())
        } else {
            Err(NeoAudioError::StreamRunning)
        }
    }

    fn input_device(&self) -> Option<String> {
        self.selected_input_device.as_ref().map(|d| d.name.clone())
    }

    fn available_num_output_channels(&self) -> u16 {
        self.selected_output_device
            .as_ref()
            .map(|d| d.output_channels as u16)
            .unwrap_or(0)
    }

    fn set_num_output_channels(&mut self, ch: u16) -> Result<(), NeoAudioError> {
        if ch > self.available_num_output_channels() {
            self.selected_num_output_channels = self.available_num_output_channels();
        } else {
            self.selected_num_output_channels = ch;
        }
        Ok(())
    }

    fn num_output_channels(&self) -> u16 {
        self.selected_num_output_channels
    }

    fn available_num_input_channels(&self) -> u16 {
        self.selected_input_device
            .as_ref()
            .map(|d| d.input_channels as u16)
            .unwrap_or(0)
    }

    fn set_num_input_channels(&mut self, ch: u16) -> Result<(), NeoAudioError> {
        if ch > self.available_num_input_channels() {
            self.selected_num_input_channels = self.available_num_input_channels();
        } else {
            self.selected_num_input_channels = ch;
        }
        Ok(())
    }

    fn num_input_channels(&self) -> u16 {
        self.selected_num_input_channels
    }

    fn available_sample_rates(&self) -> Vec<u32> {
        self.sample_rates.clone()
    }

    fn sample_rate(&self) -> u32 {
        self.selected_sample_rate
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), NeoAudioError> {
        if self.sample_rates.contains(&sample_rate) {
            self.selected_sample_rate = sample_rate;
        } else {
            self.selected_sample_rate = self.available_sample_rates()[0];
        }
        Ok(())
    }

    fn set_num_frames(&mut self, num_frames: u32) -> Result<(), NeoAudioError> {
        if [16, 32, 64, 128, 256, 512, 1024, 2048].contains(&num_frames) {
            self.selected_num_frames = num_frames;
        } else {
            self.selected_num_frames = self.available_num_frames()[0];
        }
        Ok(())
    }

    fn available_num_frames(&self) -> Vec<u32> {
        // TODO: get rid of hardcoded values
        vec![16, 32, 64, 128, 256, 512, 1024, 2048]
    }

    fn num_frames(&self) -> u32 {
        self.selected_num_frames
    }

    fn start_stream(
        &mut self,
        mut process_fn: impl FnMut(InterleavedAudioMut<'_, f32>, InterleavedAudio<'_, f32>)
            + Send
            + 'static,
    ) -> Result<(), NeoAudioError> {
        if let Some(host) = self.host.take() {
            let (sender, receiver) = mpsc::channel();
            self.error_receiver = Some(receiver);
            self.stream_handle = Some(
                host.open_stream(
                    self.selected_output_device
                        .as_ref()
                        .map(|device| DeviceParams {
                            device_id: device.id,
                            num_channels: self.selected_num_output_channels as u32,
                            first_channel: 0,
                        }),
                    self.selected_input_device
                        .as_ref()
                        .map(|device| DeviceParams {
                            device_id: device.id,
                            num_channels: self.selected_num_input_channels as u32,
                            first_channel: 0,
                        }),
                    rtaudio::SampleFormat::Float32,
                    self.selected_sample_rate,
                    self.selected_num_frames,
                    rtaudio::StreamOptions::default(),
                    move |error| {
                        sender
                            .send(NeoAudioError::from(error))
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
            Err(NeoAudioError::Unspecified)
        }
    }

    fn stop_stream(&mut self) -> Result<(), NeoAudioError> {
        if let Some(mut stream_handle) = self.stream_handle.take() {
            stream_handle.stop();
        }
        // reload host if it's gone (it was eaten by the stream)
        if self.host.is_none() {
            self.host = Some(Host::new(self.selected_api)?)
        }
        Ok(())
    }

    fn stream_error(&self) -> Result<(), NeoAudioError> {
        if let Some(receiver) = self.error_receiver.as_ref() {
            if let Ok(error) = receiver.try_recv() {
                return Err(error);
            }
        }
        Ok(())
    }
}

impl From<rtaudio::RtAudioError> for NeoAudioError {
    fn from(value: rtaudio::RtAudioError) -> Self {
        Self::Backend(value.to_string())
    }
}
