use portaudio::{
    DeviceIndex, Duplex, DuplexStreamCallbackArgs, DuplexStreamSettings, HostApiIndex, Input,
    NonBlocking, Output, OutputStreamSettings, PortAudio, Stream, StreamParameters,
};
use realtime_tools::interleaved_audio::{InterleavedAudio, InterleavedAudioMut};

use crate::{
    backends::{AudioBackend, DEFAULT_NUM_FRAMES, DEFAULT_SAMPLE_RATE},
    device_name::Device,
    prelude::NeoAudioError,
};

use super::{COMMON_FRAMES_PER_BUFFER, COMMON_SAMPLE_RATES};

// TODO: Test only output / only input streams
// TODO: Find a solution to check if sample rates and frame sizes are working upfront
// TODO: Stream Errors?

enum StreamType {
    Duplex(Stream<NonBlocking, Duplex<f32, f32>>),
    Output(Stream<NonBlocking, Output<f32>>),
    Input(Stream<NonBlocking, Input<f32>>),
}

pub struct PortAudioBackend {
    pa: PortAudio,
    host_apis: Vec<HostApiIndex>,
    input_devices: Vec<DeviceIndex>,
    output_devices: Vec<DeviceIndex>,
    sample_rates: Vec<u32>,
    selected_host: Option<i32>,
    selected_output_device: Option<DeviceIndex>,
    selected_input_device: Option<DeviceIndex>,
    selected_num_input_channels: i32,
    selected_num_output_channels: i32,
    selected_sample_rate: f64,
    selected_num_frames: u32,
    stream: Option<StreamType>,
}

impl AudioBackend for PortAudioBackend {
    fn default() -> Result<Self, NeoAudioError>
    where
        Self: Sized,
    {
        let pa = PortAudio::new().map_err(|e| NeoAudioError::Backend(e.to_string()))?;
        let mut neo_audio = Self {
            host_apis: pa.host_apis().map(|(index, _)| index).collect(),
            output_devices: Vec::new(),
            input_devices: Vec::new(),
            sample_rates: Vec::new(),
            selected_host: Some(pa.default_host_api()?),
            selected_output_device: None,
            selected_input_device: None,
            selected_num_input_channels: 0,
            selected_num_output_channels: 0,
            selected_sample_rate: DEFAULT_SAMPLE_RATE as f64,
            selected_num_frames: DEFAULT_NUM_FRAMES,
            stream: None,
            pa,
        };

        neo_audio.update_devices()?;
        neo_audio.set_output_device(Device::Default)?;
        neo_audio.set_input_device(Device::Default)?;

        Ok(neo_audio)
    }

    fn update_devices(&mut self) -> Result<(), NeoAudioError> {
        // device update should only work when stream is stopped
        if self.stream.is_some() {
            return Err(NeoAudioError::StreamRunning);
        }

        // ensure a host is selected
        if self.selected_host.is_none() {
            self.selected_host = Some(self.pa.default_host_api()?);
        }
        if let Some(host) = self.selected_host {
            self.output_devices.clear();
            self.input_devices.clear();
            for (index, info) in self.pa.devices()?.flatten() {
                if info.host_api == host {
                    if info.max_output_channels > 0 {
                        self.output_devices.push(index);
                    }
                    if info.max_input_channels > 0 {
                        self.input_devices.push(index);
                    }
                }
            }

            // use default device when selected device is not present anymore
            if let Some(output_device) = self.selected_output_device.as_ref() {
                if !self
                    .output_devices
                    .iter()
                    .any(|device| device == output_device)
                {
                    self.set_output_device(Device::Default)?;
                }
            }

            // use default device when selected device is not present anymore
            if let Some(input_device) = self.selected_input_device.as_ref() {
                if !self
                    .input_devices
                    .iter()
                    .any(|device| device == input_device)
                {
                    self.set_input_device(Device::Default)?;
                }
            }

            self.update_sample_rates();
        }

        Ok(())
    }

    fn update_sample_rates(&mut self) {
        self.sample_rates = COMMON_SAMPLE_RATES.to_vec();
        if !self
            .sample_rates
            .contains(&(self.selected_sample_rate as u32))
        {
            self.selected_sample_rate = COMMON_SAMPLE_RATES[0] as f64;
        }
    }

    fn available_apis(&self) -> Vec<String> {
        self.host_apis
            .iter()
            .map(|api| {
                self.pa
                    .host_api_info(*api)
                    .map(|info| info.name.to_string())
                    .unwrap_or_default()
            })
            .collect()
    }

    fn set_api(&mut self, api_name: &str) -> Result<(), NeoAudioError> {
        if self.stream.is_some() {
            return Err(NeoAudioError::StreamRunning);
        }
        let host = self
            .host_apis
            .iter()
            .find(|&api| {
                self.pa
                    .host_api_info(*api)
                    .map(|info| info.name.contains(api_name))
                    .unwrap_or(false)
            })
            .ok_or(NeoAudioError::ApiNotFound)?;

        self.selected_host = Some(*host);

        self.update_devices()?;

        Ok(())
    }

    fn api(&self) -> String {
        if let Some(host) = self.selected_host {
            self.pa
                .host_api_info(host)
                .map(|info| info.name.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }

    fn available_output_devices(&self) -> Vec<String> {
        self.output_devices
            .iter()
            .map(|d| {
                self.pa
                    .device_info(*d)
                    .map(|info| info.name.to_string())
                    .unwrap_or_default()
            })
            .collect()
    }

    fn set_output_device(
        &mut self,
        device: crate::device_name::Device,
    ) -> Result<(), NeoAudioError> {
        if self.stream.is_some() {
            return Err(NeoAudioError::StreamRunning);
        }

        self.selected_output_device = match device {
            Device::None => None,
            Device::Default => self.pa.default_output_device().ok(),
            Device::Name(name) => Some(
                *self
                    .output_devices
                    .iter()
                    .find(|&device| {
                        self.pa
                            .device_info(*device)
                            .map(|info| info.name.contains(&name))
                            .unwrap_or(false)
                    })
                    .ok_or(NeoAudioError::OutputDeviceNotFound)?,
            ),
        };

        self.selected_num_output_channels = self.available_num_output_channels() as i32;
        self.update_sample_rates();

        Ok(())
    }

    fn output_device(&self) -> Option<String> {
        if let Some(output_device) = self.selected_output_device {
            let info = self.pa.device_info(output_device).ok();
            if let Some(info) = info {
                return Some(info.name.to_string());
            }
        }

        None
    }

    fn available_input_devices(&self) -> Vec<String> {
        self.input_devices
            .iter()
            .map(|d| {
                self.pa
                    .device_info(*d)
                    .map(|info| info.name.to_string())
                    .unwrap_or_default()
            })
            .collect()
    }

    fn set_input_device(
        &mut self,
        device: crate::device_name::Device,
    ) -> Result<(), NeoAudioError> {
        if self.stream.is_some() {
            return Err(NeoAudioError::StreamRunning);
        }

        self.selected_input_device = match device {
            Device::None => None,
            Device::Default => self.pa.default_input_device().ok(),
            Device::Name(name) => Some(
                *self
                    .input_devices
                    .iter()
                    .find(|&&device| {
                        if let Ok(info) = self.pa.device_info(device) {
                            info.name.contains(&name)
                        } else {
                            false
                        }
                    })
                    .ok_or(NeoAudioError::InputDeviceNotFound)?,
            ),
        };

        self.selected_num_input_channels = self.available_num_input_channels() as i32;
        self.update_sample_rates();

        Ok(())
    }

    fn input_device(&self) -> Option<String> {
        if let Some(input_device) = self.selected_input_device {
            let info = self.pa.device_info(input_device).ok();
            if let Some(info) = info {
                return Some(info.name.to_string());
            }
        }

        None
    }

    fn available_num_output_channels(&self) -> u16 {
        self.selected_output_device
            .as_ref()
            .map(|d| {
                self.pa
                    .device_info(*d)
                    .map(|d| d.max_output_channels as u16)
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    fn set_num_output_channels(&mut self, ch: u16) -> Result<(), NeoAudioError> {
        self.selected_num_output_channels = ch.min(self.available_num_output_channels()) as i32;
        Ok(())
    }

    fn num_output_channels(&self) -> u16 {
        self.selected_num_output_channels as u16
    }

    fn available_num_input_channels(&self) -> u16 {
        self.selected_input_device
            .as_ref()
            .map(|d| {
                self.pa
                    .device_info(*d)
                    .map(|d| d.max_input_channels as u16)
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    fn set_num_input_channels(&mut self, ch: u16) -> Result<(), NeoAudioError> {
        self.selected_num_input_channels = ch.min(self.available_num_input_channels()) as i32;
        Ok(())
    }

    fn num_input_channels(&self) -> u16 {
        self.selected_num_input_channels as u16
    }

    fn available_sample_rates(&self) -> Vec<u32> {
        self.sample_rates.clone()
    }

    fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), NeoAudioError> {
        if self.sample_rates.contains(&sample_rate) {
            self.selected_sample_rate = sample_rate as f64;
        } else {
            self.selected_sample_rate = DEFAULT_SAMPLE_RATE as f64;
        }
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.selected_sample_rate as u32
    }

    fn available_num_frames(&self) -> Vec<u32> {
        COMMON_FRAMES_PER_BUFFER.to_vec()
    }

    fn set_num_frames(&mut self, num_frames: u32) -> Result<(), NeoAudioError> {
        if COMMON_FRAMES_PER_BUFFER.contains(&num_frames) {
            self.selected_num_frames = num_frames;
        } else {
            self.selected_num_frames = self.available_num_frames()[0];
        }
        Ok(())
    }

    fn num_frames(&self) -> u32 {
        self.selected_num_frames
    }

    fn start_stream(
        &mut self,
        mut process_fn: impl FnMut(
                realtime_tools::interleaved_audio::InterleavedAudioMut<'_, f32>,
                realtime_tools::interleaved_audio::InterleavedAudio<'_, f32>,
            ) + Send
            + 'static,
    ) -> Result<(), NeoAudioError> {
        let output_params = if let Some(output_device) = self.selected_output_device {
            let info = self.pa.device_info(output_device)?;
            let latency = info.default_low_output_latency;
            Some(StreamParameters::<f32>::new(
                output_device,
                self.selected_num_output_channels,
                true,
                latency,
            ))
        } else {
            None
        };

        let input_params = if let Some(input_device) = self.selected_input_device {
            let latency = if let Some(output_params) = output_params {
                output_params.suggested_latency
            } else {
                let info = self.pa.device_info(input_device)?;
                info.default_low_output_latency
            };
            Some(StreamParameters::<f32>::new(
                input_device,
                self.selected_num_input_channels,
                true,
                latency,
            ))
        } else {
            None
        };

        match (output_params, input_params) {
            (Some(output_params), Some(input_params)) => {
                let settings = DuplexStreamSettings::new(
                    input_params,
                    output_params,
                    self.selected_sample_rate,
                    self.selected_num_frames,
                );
                let callback = move |DuplexStreamCallbackArgs::<f32, f32> {
                                         in_buffer,
                                         out_buffer,
                                         frames,
                                         ..
                                     }| {
                    let num_output_channels = out_buffer.len() / frames;
                    let num_input_channels = in_buffer.len() / frames;
                    process_fn(
                        InterleavedAudioMut::from_slice(out_buffer, num_output_channels),
                        InterleavedAudio::from_slice(in_buffer, num_input_channels),
                    );
                    portaudio::Continue
                };
                let mut stream = self.pa.open_non_blocking_stream(settings, callback)?;
                stream.start()?;
                self.stream = Some(StreamType::Duplex(stream));
            }
            (Some(output_params), None) => {
                let settings = OutputStreamSettings::new(
                    output_params,
                    self.selected_sample_rate,
                    self.selected_num_frames,
                );
                let callback =
                    move |portaudio::OutputStreamCallbackArgs::<f32> { buffer, frames, .. }| {
                        let num_channels = buffer.len() / frames;
                        process_fn(
                            InterleavedAudioMut::from_slice(buffer, num_channels),
                            InterleavedAudio::from_slice(&[], 0),
                        );
                        portaudio::Continue
                    };
                let mut stream = self.pa.open_non_blocking_stream(settings, callback)?;
                stream.start()?;
                self.stream = Some(StreamType::Output(stream));
            }
            (None, Some(input_params)) => {
                let settings = portaudio::InputStreamSettings::new(
                    input_params,
                    self.selected_sample_rate,
                    self.selected_num_frames,
                );
                let callback =
                    move |portaudio::InputStreamCallbackArgs::<f32> { buffer, frames, .. }| {
                        let num_channels = buffer.len() / frames;
                        process_fn(
                            InterleavedAudioMut::from_slice(&mut [], 0),
                            InterleavedAudio::from_slice(buffer, num_channels),
                        );
                        portaudio::Continue
                    };
                let mut stream = self.pa.open_non_blocking_stream(settings, callback)?;
                stream.start()?;
                self.stream = Some(StreamType::Input(stream));
            }
            _ => {
                self.stream = None;
            }
        }
        Ok(())
    }

    fn stop_stream(&mut self) -> Result<(), NeoAudioError> {
        if let Some(stream) = self.stream.as_mut() {
            match stream {
                StreamType::Duplex(stream) => stream.stop()?,
                StreamType::Output(stream) => stream.stop()?,
                StreamType::Input(stream) => stream.stop()?,
            }
        }
        self.stream = None;
        Ok(())
    }

    fn stream_error(&self) -> Result<(), NeoAudioError> {
        Ok(())
    }
}

impl From<portaudio::Error> for NeoAudioError {
    fn from(value: portaudio::Error) -> Self {
        Self::Backend(value.to_string())
    }
}
