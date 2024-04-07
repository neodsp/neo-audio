use audio_processor::AudioProcessor;
use crossbeam_channel::{Receiver, Sender};
use error::NeoAudioError;
use system_audio::{system_audio_error::SystemAudioError, SystemAudio};

pub mod audio_processor;
pub mod error;
pub mod prelude;
#[cfg(feature = "processors")]
pub mod processors;

pub struct NeoAudio<S, P>
where
    S: SystemAudio,
    P: AudioProcessor + Send + 'static,
    <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
{
    system_audio: S,
    sender: Sender<P::Message>,
    receiver: Receiver<P::Message>,
}

unsafe impl<S, P> Sync for NeoAudio<S, P>
where
    S: SystemAudio,
    P: AudioProcessor + Send + 'static,
    <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
{
}
unsafe impl<S, P> Send for NeoAudio<S, P>
where
    S: SystemAudio,
    P: AudioProcessor + Send + 'static,
    <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
{
}

impl<S, P> NeoAudio<S, P>
where
    S: SystemAudio,
    P: AudioProcessor + Send + 'static,
    <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
{
    pub fn new() -> Result<Self, NeoAudioError> {
        let (sender, receiver) = crossbeam_channel::bounded(1024);
        Ok(Self {
            system_audio: S::default()?,
            sender,
            receiver,
        })
    }

    pub fn system_audio(&self) -> &S {
        &self.system_audio
    }

    pub fn system_audio_mut(&mut self) -> &mut S {
        &mut self.system_audio
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

    pub fn start_audio(&mut self, mut processor: P) -> Result<(), SystemAudioError>
    where
        <P as audio_processor::AudioProcessor>::Message: std::marker::Send,
    {
        processor.prepare(self.system_audio.config());
        let rcv = self.receiver.clone();
        self.system_audio.start_stream(move |output, input| {
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
        self.system_audio.stop_stream()?;
        self.system_audio.stream_error()?;
        Ok(())
    }
}
