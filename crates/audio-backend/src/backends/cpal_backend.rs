use cpal::{
    traits::{DeviceTrait, HostTrait},
    SampleFormat, SampleRate, SupportedStreamConfigRange,
};

use crate::{
    audio_backend_error::AudioBackendError, backends::COMMON_SAMPLE_RATES, device_name::Device,
    AudioBackend, DEFAULT_SAMPLE_RATE,
};

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
        let apis = cpal::available_hosts().iter().cloned().collect::<Vec<_>>();
        Ok(Self {
            apis,
            selected_api: cpal::default_host(),
        })
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
        self.sample_rates = COMMON_SAMPLE_RATES.to_vec();
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
        
        match (self.input_config_f32(), self.output_config_f32()) {
            (Some(inp), Some(out)) => {
                
            }
            _ => {}
        }
    }

    fn set_sample_rate(
        &mut self,
        sample_rate: u32,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        todo!()
    }

    fn sample_rate(&self) -> u32 {
        todo!()
    }

    fn available_num_frames(&self) -> Vec<u32> {
        todo!()
    }

    fn set_num_frames(
        &mut self,
        num_frames: u32,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        todo!()
    }

    fn num_frames(&self) -> u32 {
        todo!()
    }

    fn start_stream(
        &mut self,
        process_fn: impl FnMut(
                rt_tools::interleaved_audio::InterleavedAudioMut<'_, f32>,
                rt_tools::interleaved_audio::InterleavedAudio<'_, f32>,
            ) + Send
            + 'static,
    ) -> Result<(), crate::audio_backend_error::AudioBackendError> {
    }

    fn stop_stream(&mut self) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        todo!()
    }

    fn stream_error(&self) -> Result<(), crate::audio_backend_error::AudioBackendError> {
        todo!()
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
