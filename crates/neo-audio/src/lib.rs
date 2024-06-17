use audio_processor::AudioProcessor;
use backends::AudioBackend;
use crossbeam_channel::Sender;
use error::Error;

pub mod audio_processor;
pub mod backends;
pub mod device_config;
pub mod device_name;
pub mod error;
pub mod prelude;
#[cfg(feature = "processors")]
pub mod processors;

pub struct NeoAudio<B>
where
    B: AudioBackend,
{
    backend: B,
}

unsafe impl<B> Sync for NeoAudio<B> where B: AudioBackend {}
unsafe impl<B> Send for NeoAudio<B> where B: AudioBackend {}

impl<B> NeoAudio<B>
where
    B: AudioBackend,
{
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            backend: B::default()?,
        })
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn start_audio<P>(&mut self, mut processor: P) -> Result<Sender<P::Message>, Error>
    where
        P: AudioProcessor + Send + 'static,
        <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
    {
        let (sender, receiver) = crossbeam_channel::bounded(1024);
        processor.prepare(self.backend.config());
        self.backend.start_stream(move |output, input| {
            // receive all messages
            for _ in 0..receiver.len() {
                match receiver.try_recv() {
                    Ok(message) => {
                        processor.message_process(message);
                    }
                    _ => break,
                }
            }

            processor.process(output, input);
        })?;
        Ok(sender)
    }

    pub fn stop_audio(&mut self) -> Result<(), Error> {
        self.backend.stop_stream()?;
        self.backend.stream_error()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::backends::portaudio_backend::PortAudioBackend;

    use crate::prelude::*;
    use crate::{processors::feedback::FeedbackProcessor, NeoAudio};

    #[test]
    #[ignore = "manual test"]
    fn feedback() {
        let mut audio = NeoAudio::<PortAudioBackend>::new().unwrap();

        audio.start_audio(FeedbackProcessor::default()).unwrap();

        sleep(Duration::from_secs(5));

        audio.stop_audio().unwrap();
    }

    #[test]
    fn readme_code_test() -> Result<(), Error> {
        enum MyMessage {
            Gain(f32),
        }

        struct MyProcessor {
            gain: f32,
        }

        impl Default for MyProcessor {
            fn default() -> Self {
                Self { gain: 1.0 }
            }
        }

        impl AudioProcessor for MyProcessor {
            type Message = MyMessage;

            fn prepare(&mut self, config: DeviceConfig) {
                println!("Prepare is called with {:?}", config);
            }

            fn message_process(&mut self, message: Self::Message) {
                match message {
                    MyMessage::Gain(gain) => self.gain = gain,
                }
            }

            /// This is a simple feedback with gain
            fn process(
                &mut self,
                mut output: InterleavedAudioMut<'_, f32>,
                input: InterleavedAudio<'_, f32>,
            ) {
                for (out_frame, in_frame) in output.frames_iter_mut().zip(input.frames_iter()) {
                    out_frame
                        .iter_mut()
                        .zip(in_frame.iter())
                        .for_each(|(o, i)| *o = *i * self.gain);
                }
            }
        }

        let mut neo_audio = NeoAudio::<PortAudioBackend>::new()?;
        
        let _output_devices = neo_audio.backend().available_output_devices();

        // don't start an output stream
        neo_audio.backend_mut().set_output_device(Device::None)?;
        // use the system default device
        neo_audio.backend_mut().set_output_device(Device::Default)?;
        // specify a device by name
        neo_audio
            .backend_mut()
            .set_output_device(Device::Name("My Soundcard Name".into()))?;

        let _selected_output_device = neo_audio.backend().output_device();

        let sender = neo_audio.start_audio(MyProcessor::default())?;

        sender.send(MyMessage::Gain(0.5))?;

        neo_audio.stop_audio()?;

        Ok(())
    }
}
