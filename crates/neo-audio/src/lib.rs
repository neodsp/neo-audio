use audio_backend::AudioBackend;
use audio_processor::AudioProcessor;
use crossbeam_channel::Sender;
use error::NeoAudioError;

pub mod audio_processor;
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
    pub fn new() -> Result<Self, NeoAudioError> {
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

    pub fn start_audio<P>(&mut self, mut processor: P) -> Result<Sender<P::Message>, NeoAudioError>
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

    pub fn stop_audio(&mut self) -> Result<(), NeoAudioError> {
        self.backend.stop_stream()?;
        self.backend.stream_error()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use audio_backend::backends::portaudio_backend::PortAudioBackend;

    use crate::{processors::feedback::FeedbackProcessor, NeoAudio};

    #[test]
    fn feedback() {
        let mut audio = NeoAudio::<PortAudioBackend>::new().unwrap();

        audio.start_audio(FeedbackProcessor::default()).unwrap();

        sleep(Duration::from_secs(5));

        audio.stop_audio().unwrap();
    }
}
