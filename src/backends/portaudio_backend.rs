use std::error::Error;

use crossbeam_channel::Receiver;
use weresocool_portaudio::*;

use crate::{
    audio_block::{AudioBlock, AudioBlockMut},
    available_devices::{AvailableDevices, Driver, InputDevice, OutputDevice},
    device_config::DeviceConfig,
    error::NeoAudioError,
};

use super::AudioBackend;

impl From<weresocool_portaudio::Error> for NeoAudioError {
    fn from(value: weresocool_portaudio::Error) -> Self {
        NeoAudioError::UnknownBackendError(value.to_string())
    }
}

pub struct PortaudioBackend {
    pa: PortAudio,
    stream: Option<Stream<NonBlocking, Duplex<f32, f32>>>,
    error_receiver: Option<Receiver<Box<dyn Error>>>,
}

impl AudioBackend for PortaudioBackend {
    fn new() -> Result<Self, crate::error::NeoAudioError>
    where
        Self: Sized,
    {
        Ok(Self {
            pa: PortAudio::new()?,
            stream: None,
            error_receiver: None,
        })
    }

    fn available_devices(&self) -> Result<AvailableDevices, crate::error::NeoAudioError> {
        Ok(AvailableDevices {
            apis: self
                .pa
                .host_apis()
                .map(|(host_index, host_info)| {
                    let mut input_devices = Vec::new();
                    let mut output_devices = Vec::new();
                    for (_, device_info) in self.pa.devices().unwrap().flatten() {
                        if host_index == device_info.host_api {
                            if device_info.max_input_channels > 0 {
                                input_devices.push(InputDevice {
                                    name: device_info.name.to_string(),
                                    num_ch: device_info.max_input_channels as u16,
                                })
                            }
                            if device_info.max_output_channels > 0 {
                                output_devices.push(OutputDevice {
                                    name: device_info.name.to_string(),
                                    num_ch: device_info.max_output_channels as u16,
                                })
                            }
                        }
                    }
                    Driver {
                        name: host_info.name.to_string(),
                        input_devices,
                        output_devices,
                    }
                })
                .collect(),
        })
    }

    fn default_config(&self) -> Result<DeviceConfig, NeoAudioError> {
        let input_info = self.pa.device_info(self.pa.default_input_device()?)?;
        let output_info = self.pa.device_info(self.pa.default_output_device()?)?;
        Ok(DeviceConfig {
            driver: self
                .pa
                .host_api_info(self.pa.default_host_api()?)
                .unwrap()
                .name
                .to_string(),
            input_device: input_info.name.to_string(),
            output_device: output_info.name.to_string(),
            num_input_ch: input_info.max_input_channels as u16,
            num_output_ch: output_info.max_output_channels as u16,
            sample_rate: input_info.default_sample_rate,
            num_frames: 512,
        })
    }

    fn start_stream<F>(
        &mut self,
        device_config: DeviceConfig,
        mut process_fn: F,
    ) -> Result<(), crate::error::NeoAudioError>
    where
        F: FnMut(AudioBlock, AudioBlockMut) -> Result<(), Box<dyn Error>> + 'static,
    {
        let (error_sender, error_receiver) = crossbeam_channel::bounded(1024);
        self.error_receiver = Some(error_receiver);
        let settings = DuplexStreamSettings::new(
            self.input_params(&device_config.input_device, device_config.num_input_ch)?,
            self.output_params(&device_config.output_device, device_config.num_output_ch)?,
            device_config.sample_rate,
            device_config.num_frames,
        );

        let callback = move |DuplexStreamCallbackArgs::<f32, f32> {
                                 in_buffer,
                                 out_buffer,
                                 frames,
                                 ..
                             }| {
            match process_fn(
                AudioBlock {
                    data: in_buffer,
                    num_channels: (in_buffer.len() / frames) as u16,
                },
                AudioBlockMut {
                    num_channels: (out_buffer.len() / frames) as u16,
                    data: out_buffer,
                },
            ) {
                Ok(()) => weresocool_portaudio::Continue,
                Err(e) => {
                    let _ = error_sender.try_send(e);
                    weresocool_portaudio::Abort
                }
            }
        };

        let mut stream = self.pa.open_non_blocking_stream(settings, callback)?;
        stream.start()?;
        self.stream = Some(stream);
        Ok(())
    }

    fn stop_stream(&mut self) -> Result<(), NeoAudioError> {
        if let Some(mut stream) = self.stream.take() {
            stream.stop()?;
        }

        Ok(())
    }

    fn stream_error(&self) -> Result<(), crate::error::NeoAudioError> {
        if let Some(receiver) = self.error_receiver.as_ref() {
            for _ in 0..receiver.len() {
                if let Ok(error) = receiver.try_recv() {
                    return Err(NeoAudioError::ProcessorError(error.to_string()));
                }
            }
        }
        Ok(())
    }
}

impl PortaudioBackend {
    fn output_params(
        &self,
        name: &str,
        num_ch: u16,
    ) -> Result<StreamParameters<f32>, NeoAudioError> {
        for (index, device_info) in self.pa.devices()?.flatten() {
            if device_info.max_output_channels > 0 && device_info.name.contains(name) {
                return Ok(StreamParameters::<f32>::new(
                    index,
                    num_ch as i32,
                    true,
                    device_info.default_low_input_latency,
                ));
            }
        }
        Err(NeoAudioError::DeviceNotFound(name.to_string()))
    }

    fn input_params(
        &self,
        name: &str,
        num_ch: u16,
    ) -> Result<StreamParameters<f32>, NeoAudioError> {
        for (index, device_info) in self.pa.devices().unwrap().flatten() {
            if device_info.max_input_channels > 0 && device_info.name.contains(name) {
                return Ok(StreamParameters::<f32>::new(
                    index,
                    num_ch as i32,
                    true,
                    device_info.default_low_input_latency,
                ));
            }
        }
        Err(NeoAudioError::DeviceNotFound(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portaudio_backend() -> Result<(), Box<dyn Error>> {
        let mut backend = PortaudioBackend::new()?;
        dbg!(backend.available_devices()?);
        backend.start_stream(
            backend.default_config()?,
            move |input, output| -> Result<(), Box<dyn Error>> {
                output.data.copy_from_slice(input.data);
                Ok(())
            },
        )?;

        std::thread::sleep(std::time::Duration::from_secs(3));

        backend.stop_stream()?;

        backend.stream_error()?;
        Ok(())
    }
}
