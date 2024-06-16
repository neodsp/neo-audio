use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, Stream, StreamConfig, SupportedStreamConfigRange,
};
use ringbuf::{traits::*, HeapRb};
use rt_tools::interleaved_audio::{InterleavedAudio, InterleavedAudioMut};

use crate::{
    audio_backend_error::AudioBackendError, backends::COMMON_SAMPLE_RATES, device_name::Device,
    AudioBackend, DEFAULT_NUM_FRAMES, DEFAULT_SAMPLE_RATE,
};

use super::COMMON_FRAMES_PER_BUFFER;

pub struct CpalBackend {
    apis: Vec<cpal::HostId>,
    output_devices: Vec<cpal::Device>,
    input_devices: Vec<cpal::Device>,
    sample_rates: Vec<u32>,
    selected_api: cpal::Host,
    selected_output_device: Option<cpal::Device>,
    selected_input_device: Option<cpal::Device>,
    selected_num_output_channels: u32,
    selected_num_input_channels: u32,
    selected_sample_rate: u32,
    selected_num_frames: u32,
    output_stream: Option<Stream>,
    input_stream: Option<Stream>,
}

impl CpalBackend {
    fn output_config_f32(&self) -> Option<SupportedStreamConfigRange> {
        if let Some(device) = self.selected_output_device.as_ref() {
            let formats = match device.supported_output_configs() {
                Ok(formats) => formats,
                Err(err) => {
                    eprintln!("Error while querying formats: {}", err);
                    return None;
                }
            };
            formats
                .filter(|f| f.sample_format() == SampleFormat::F32)
                .last()
        } else {
            None
        }
    }

    fn input_config_f32(&self) -> Option<SupportedStreamConfigRange> {
        if let Some(device) = self.selected_input_device.as_ref() {
            let formats = match device.supported_input_configs() {
                Ok(formats) => formats,
                Err(err) => {
                    eprintln!("Error while querying formats: {}", err);
                    return None;
                }
            };
            formats
                .filter(|f| f.sample_format() == SampleFormat::F32)
                .last()
        } else {
            None
        }
    }
}

impl AudioBackend for CpalBackend {
    fn default() -> Result<Self, crate::audio_backend_error::AudioBackendError>
    where
        Self: Sized,
    {
        unimplemented!("This is still WIP!");
        let apis = cpal::available_hosts().iter().cloned().collect::<Vec<_>>();
        let mut neo_audio = Self {
            apis,
            selected_api: cpal::default_host(),
            output_devices: Vec::new(),
            input_devices: Vec::new(),
            sample_rates: Vec::new(),
            selected_output_device: None,
            selected_input_device: None,
            selected_num_output_channels: 0,
            selected_num_input_channels: 0,
            selected_sample_rate: DEFAULT_SAMPLE_RATE,
            selected_num_frames: DEFAULT_NUM_FRAMES,
            output_stream: None,
            input_stream: None,
        };

        neo_audio.update_devices()?;
        neo_audio.set_output_device(Device::Default)?;
        neo_audio.set_input_device(Device::Default)?;

        Ok(neo_audio)
    }

    fn update_devices(&mut self) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        // update availabe devices
        self.output_devices = self.selected_api.output_devices()?.into_iter().collect();
        self.input_devices = self.selected_api.input_devices()?.into_iter().collect();

        // use default device when selected output device is not present anymore
        if let Some(output_device) = self.selected_output_device.as_ref() {
            let mut device_still_present = false;
            for device in self.output_devices.iter() {
                if device.name()? == output_device.name()? {
                    device_still_present = true;
                }
            }
            if !device_still_present {
                self.set_output_device(Device::Default)?;
            }
        }

        // use default device when selected input device is not present anymore
        if let Some(input_device) = self.selected_input_device.as_ref() {
            let mut device_still_present = false;
            for device in self.output_devices.iter() {
                if device.name()? == input_device.name()? {
                    device_still_present = true;
                }
            }
            if !device_still_present {
                self.set_input_device(Device::Default)?;
            }
        }

        self.update_sample_rates();

        Ok(())
    }

    fn update_sample_rates(&mut self) {
        self.sample_rates = self.available_sample_rates();
        // set default sample rate if sample_rate is not available
        if !self
            .sample_rates
            .contains(&(self.selected_sample_rate as u32))
        {
            self.selected_sample_rate = DEFAULT_SAMPLE_RATE;
        }
    }

    fn available_apis(&self) -> Vec<String> {
        self.apis.iter().map(|api| api.name().to_string()).collect()
    }

    fn set_api(
        &mut self,
        api_name: &str,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        self.selected_api = cpal::host_from_id(
            *self
                .apis
                .iter()
                .find(|api| api.name().contains(api_name))
                .ok_or(AudioBackendError::ApiNotFound)?,
        )?;
        self.update_devices()?;
        Ok(())
    }

    fn api(&self) -> String {
        self.selected_api.id().name().to_string()
    }

    fn available_output_devices(&self) -> Vec<String> {
        self.output_devices
            .iter()
            .map(|d| d.name().unwrap_or_default())
            .collect()
    }

    fn set_output_device(
        &mut self,
        device: crate::device_name::Device,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        self.selected_output_device = match device {
            Device::None => None,
            Device::Default => self.selected_api.default_output_device(),
            Device::Name(name) => Some(
                self.output_devices
                    .iter()
                    .find(|d| d.name().unwrap_or_default().contains(&name))
                    .ok_or(AudioBackendError::OutputDeviceNotFound)?
                    .clone(),
            ),
        };
        self.selected_num_output_channels = self.available_num_output_channels();
        self.update_sample_rates();
        Ok(())
    }

    fn output_device(&self) -> Option<String> {
        self.selected_output_device
            .as_ref()
            .map(|d| d.name().unwrap_or_default())
    }

    fn available_input_devices(&self) -> Vec<String> {
        self.input_devices
            .iter()
            .map(|d| d.name().unwrap_or_default())
            .collect()
    }

    fn set_input_device(
        &mut self,
        device: crate::device_name::Device,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        self.selected_input_device = match device {
            Device::None => None,
            Device::Default => self.selected_api.default_input_device(),
            Device::Name(name) => Some(
                self.input_devices
                    .iter()
                    .find(|d| d.name().unwrap_or_default().contains(&name))
                    .ok_or(AudioBackendError::InputDeviceNotFound)?
                    .clone(),
            ),
        };
        self.selected_num_input_channels = self.available_num_input_channels();
        self.update_sample_rates();
        Ok(())
    }

    fn input_device(&self) -> Option<String> {
        self.selected_input_device
            .as_ref()
            .map(|d| d.name().unwrap_or_default())
    }

    fn available_num_output_channels(&self) -> u32 {
        self.output_config_f32()
            .map(|c| c.channels() as u32)
            .unwrap_or(0)
    }

    fn set_num_output_channels(
        &mut self,
        ch: u32,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        self.selected_num_output_channels = ch.max(self.available_num_output_channels());
        Ok(())
    }

    fn num_output_channels(&self) -> u32 {
        self.selected_num_output_channels
    }

    fn available_num_input_channels(&self) -> u32 {
        self.input_config_f32()
            .map(|c| c.channels() as u32)
            .unwrap_or(0)
    }

    fn set_num_input_channels(
        &mut self,
        ch: u32,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        self.selected_num_input_channels = ch.max(self.available_num_input_channels());
        Ok(())
    }

    fn num_input_channels(&self) -> u32 {
        self.selected_num_input_channels
    }

    fn available_sample_rates(&self) -> Vec<u32> {
        let (min, max) = match (self.input_config_f32(), self.output_config_f32()) {
            (Some(inp), Some(out)) => (
                inp.min_sample_rate().0.max(out.max_sample_rate().0),
                inp.max_sample_rate().0.min(out.max_sample_rate().0),
            ),
            (None, Some(out)) => (out.min_sample_rate().0, out.max_sample_rate().0),
            (Some(inp), None) => (inp.min_sample_rate().0, inp.max_sample_rate().0),
            _ => {
                assert!(false);
                (0, 0)
            }
        };
        COMMON_SAMPLE_RATES
            .iter()
            .copied()
            .filter(|&sr| sr >= min && sr <= max)
            .collect()
    }

    fn set_sample_rate(
        &mut self,
        sample_rate: u32,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        if self.available_sample_rates().contains(&sample_rate) {
            self.selected_sample_rate = sample_rate;
            Ok(())
        } else {
            Err(AudioBackendError::SampleRate)
        }
    }

    fn sample_rate(&self) -> u32 {
        self.selected_sample_rate
    }

    fn available_num_frames(&self) -> Vec<u32> {
        COMMON_FRAMES_PER_BUFFER.to_vec()
    }

    fn set_num_frames(
        &mut self,
        num_frames: u32,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        if self.available_num_frames().contains(&num_frames) {
            self.selected_num_frames = num_frames;
            Ok(())
        } else {
            Err(AudioBackendError::NumFrames)
        }
    }

    fn num_frames(&self) -> u32 {
        self.selected_num_frames
    }

    fn start_stream(
        &mut self,
        mut process_fn: impl FnMut(
                rt_tools::interleaved_audio::InterleavedAudioMut<'_, f32>,
                rt_tools::interleaved_audio::InterleavedAudio<'_, f32>,
            ) + Send
            + 'static,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        match (
            self.selected_output_device.as_ref(),
            self.selected_input_device.as_ref(),
        ) {
            (None, None) => (),
            (None, Some(input)) => (),
            (Some(output), None) => (),
            (Some(output), Some(input)) => {
                let latency_samples = self.selected_num_frames as usize * 2;
                let ring_buffer =
                    HeapRb::<f32>::new(self.selected_num_frames as usize * 2 + latency_samples * 2);
                let (mut producer, mut consumer) = ring_buffer.split();
                for _ in 0..latency_samples {
                    producer.try_push(0.0).unwrap();
                }
                let mut input_buffer = vec![
                    0.0;
                    self.selected_num_frames as usize
                        * self.selected_num_input_channels as usize
                ];

                let config = cpal::StreamConfig {
                    channels: self.selected_num_output_channels as u16,
                    sample_rate: SampleRate(self.selected_sample_rate),
                    buffer_size: cpal::BufferSize::Fixed(self.selected_num_frames),
                };

                let num_output_ch = self.selected_num_output_channels;
                let num_input_ch = self.selected_num_input_channels;

                let output_stream = output.build_output_stream(
                    &config,
                    move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                        let _consumed = consumer.pop_slice(&mut input_buffer);
                        process_fn(
                            InterleavedAudioMut::from_slice(data, num_output_ch as usize),
                            InterleavedAudio::from_slice(&input_buffer, num_input_ch as usize),
                        );
                    },
                    move |err| eprintln!("Error in output stream: {:?}", err),
                    None,
                )?;

                let input_stream = input.build_input_stream(
                    &config,
                    move |data: &[f32], _info: &cpal::InputCallbackInfo| {
                        producer.push_slice(data);
                    },
                    move |err| eprintln!("Error in input stream: {:?}", err),
                    None,
                )?;

                input_stream.play();
                output_stream.play();

                self.output_stream = Some(output_stream);
                self.input_stream = Some(input_stream);
            }
        }

        Ok(())
    }

    fn stop_stream(&mut self) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        self.output_stream.as_mut().map(|s| s.pause());
        self.input_stream.as_mut().map(|s| s.pause());
        self.output_stream = None;
        self.input_stream = None;
        Ok(())
    }

    fn stream_error(&self) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        Ok(())
    }
}

impl From<cpal::StreamError> for AudioBackendError {
    fn from(value: cpal::StreamError) -> Self {
        Self::Backend(value.to_string())
    }
}

impl From<cpal::DevicesError> for AudioBackendError {
    fn from(value: cpal::DevicesError) -> Self {
        Self::Backend(value.to_string())
    }
}

impl From<cpal::DeviceNameError> for AudioBackendError {
    fn from(value: cpal::DeviceNameError) -> Self {
        Self::Backend(value.to_string())
    }
}

impl From<cpal::PlayStreamError> for AudioBackendError {
    fn from(value: cpal::PlayStreamError) -> Self {
        Self::Backend(value.to_string())
    }
}

impl From<cpal::BuildStreamError> for AudioBackendError {
    fn from(value: cpal::BuildStreamError) -> Self {
        Self::Backend(value.to_string())
    }
}

impl From<cpal::PauseStreamError> for AudioBackendError {
    fn from(value: cpal::PauseStreamError) -> Self {
        Self::Backend(value.to_string())
    }
}

impl From<cpal::BackendSpecificError> for AudioBackendError {
    fn from(value: cpal::BackendSpecificError) -> Self {
        Self::Backend(value.to_string())
    }
}

impl From<cpal::SupportedStreamConfigsError> for AudioBackendError {
    fn from(value: cpal::SupportedStreamConfigsError) -> Self {
        Self::Backend(value.to_string())
    }
}

impl From<cpal::DefaultStreamConfigError> for AudioBackendError {
    fn from(value: cpal::DefaultStreamConfigError) -> Self {
        Self::Backend(value.to_string())
    }
}

impl From<cpal::HostUnavailable> for AudioBackendError {
    fn from(value: cpal::HostUnavailable) -> Self {
        Self::Backend(value.to_string())
    }
}
