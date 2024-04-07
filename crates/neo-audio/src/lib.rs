use audio_backend::{audio_backend_error::AudioBackendError, AudioBackend};
use audio_processor::AudioProcessor;
use crossbeam_channel::{Receiver, Sender};
use error::NeoAudioError;

pub mod audio_processor;
pub mod error;
pub mod prelude;
#[cfg(feature = "processors")]
pub mod processors;

pub struct NeoAudio<B, P>
where
    B: AudioBackend,
    P: AudioProcessor + Send + 'static,
    <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
{
    backend: B,
    sender: Sender<P::Message>,
    receiver: Receiver<P::Message>,
}

unsafe impl<B, P> Sync for NeoAudio<B, P>
where
    B: AudioBackend,
    P: AudioProcessor + Send + 'static,
    <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
{
}
unsafe impl<B, P> Send for NeoAudio<B, P>
where
    B: AudioBackend,
    P: AudioProcessor + Send + 'static,
    <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
{
}

impl<B, P> NeoAudio<B, P>
where
    B: AudioBackend,
    P: AudioProcessor + Send + 'static,
    <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
{
    pub fn new() -> Result<Self, NeoAudioError> {
        let (sender, receiver) = crossbeam_channel::bounded(1024);
        Ok(Self {
            backend: B::default()?,
            sender,
            receiver,
        })
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn sender(&self) -> &Sender<P::Message> {
        &self.sender
    }

    pub fn send_message(&self, message: P::Message) -> Result<(), NeoAudioError> {
        self.sender
            .send(message)
            .map_err(|_| NeoAudioError::SendFailed)?;
        Ok(())
    }

    pub fn start_audio(&mut self, mut processor: P) -> Result<(), AudioBackendError>
    where
        <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
    {
        processor.prepare(self.backend.config());
        let rcv = self.receiver.clone();
        self.backend.start_stream(move |output, input| {
            // receive all messages
            for _ in 0..rcv.len() {
                match rcv.try_recv() {
                    Ok(message) => {
                        processor.message_process(message);
                    }
                    _ => break,
                }
            }

            processor.process(output, input);
        })
    }

    pub fn stop_audio(&mut self) -> Result<(), NeoAudioError> {
        self.backend.stop_stream()?;
        self.backend.stream_error()?;
        Ok(())
    }
}
