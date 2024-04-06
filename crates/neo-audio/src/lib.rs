use audio_processor::AudioProcessor;
use crossbeam_channel::{Receiver, Sender};
use error::NeoAudioError;
use system_audio::{system_audio_error::SystemAudioError, SystemAudio};

pub use system_audio::implementations::system_rtaudio::SystemRtAudio;

pub mod audio_processor;
pub mod error;

pub struct NeoAudio<S: SystemAudio, Message: Send + 'static> {
    system_audio: S,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

unsafe impl<S: SystemAudio, M: Send + 'static> Sync for NeoAudio<S, M> {}
unsafe impl<S: SystemAudio, M: Send + 'static> Send for NeoAudio<S, M> {}

impl<S: SystemAudio, M: Send + 'static> NeoAudio<S, M> {
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

    pub fn sender(&self) -> &Sender<M> {
        &self.sender
    }

    pub fn send_message(&self, message: M) -> Result<(), NeoAudioError> {
        self.sender
            .send(message)
            .map_err(|_| NeoAudioError::SendFailed)?;
        Ok(())
    }

    pub fn start_audio<P: AudioProcessor<M> + Send + 'static>(
        &mut self,
        mut processor: P,
    ) -> Result<(), SystemAudioError> {
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
